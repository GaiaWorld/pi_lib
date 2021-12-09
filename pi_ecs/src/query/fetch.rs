use crate::{
    archetype::{Archetype, ArchetypeComponentId, ArchetypeId},
    component::{Component, ComponentId, StorageType},
    entity::Entity,
    query::{Access, FilteredAccess},
    storage::{SlotMap, Keys, Values, LocalVersion, SecondaryMap, SparseSecondaryMap, Local},
    world::World,
};
use pi_ecs_macros::all_tuples;
use std::{
    marker::PhantomData,
    ptr::{self, NonNull},
	mem::MaybeUninit,
};

/// WorldQuery 从world上fetch组件、实体、资源，需要实现该triat
pub trait WorldQuery {
    type Fetch: for<'a> Fetch<'a, State = Self::State>;
    type State: FetchState;
}

pub trait Fetch<'w>: Sized {
    type Item;
    type State: FetchState;

    /// 创建一个新的fetch实例.
    ///
    /// # Safety
    /// `state` must have been initialized (via [FetchState::init]) using the same `world` passed in
    /// to this function.
    unsafe fn init(
        world: &World,
        state: &Self::State,
    ) -> Self;

    /// Adjusts internal state to account for the next [Archetype]. This will always be called on
    /// archetypes that match this [Fetch]
    ///
    /// # Safety
    /// `archetype` and `tables` must be from the [World] [Fetch::init] was called on. `state` must
    /// be the [Self::State] this was initialized with.
    unsafe fn set_archetype(&mut self, state: &Self::State, archetype: &Archetype);

    /// # Safety
    /// Must always be called _after_ [Fetch::set_archetype]. `archetype_index` must be in the range
    /// of the current archetype
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Option<Self::Item>;
}

/// State used to construct a Fetch. This will be cached inside QueryState, so it is best to move as
/// much data / computation here as possible to reduce the cost of constructing Fetch.
/// SAFETY:
/// Implementor must ensure that [FetchState::update_component_access] and
/// [FetchState::update_archetype_component_access] exactly reflects the results of
/// [FetchState::matches_archetype], [FetchState::matches_table], [Fetch::archetype_fetch], and
/// [Fetch::table_fetch]
pub unsafe trait FetchState: Send + Sync + Sized {
	/// 创建FetchState实例
    fn init(world: &mut World) -> Self;
	/// 更新组件
    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>);
    fn matches_archetype(&self, archetype: &Archetype) -> bool;
    // fn matches_table(&self, table: &Table) -> bool;
}

/// A fetch that is read only. This must only be implemented for read-only fetches.
pub unsafe trait ReadOnlyFetch {}

/// 为实例实现WorldQuery
impl WorldQuery for Entity {
    type Fetch = EntityFetch;
    type State = EntityState;
}

pub struct EntityFetch {
    // entities: *const Entity,
	iter: MaybeUninit<Keys<'static, LocalVersion, ()>>,
	archetype_id: ArchetypeId,
}

/// SAFE: access is read only
unsafe impl ReadOnlyFetch for EntityFetch {}

pub struct EntityState;

// SAFE: no component or archetype access
unsafe impl FetchState for EntityState {
	#[inline]
    fn init(_world: &mut World) -> Self {
        Self
    }

	#[inline]
    fn update_component_access(&self, _access: &mut FilteredAccess<ComponentId>) {}

    #[inline]
    fn matches_archetype(&self, _archetype: &Archetype) -> bool {
        true
    }
}

impl<'w> Fetch<'w> for EntityFetch {
    type Item = Entity;
    type State = EntityState;

    unsafe fn init(
        _world: &World,
        _state: &Self::State,
    ) -> Self {
        Self {
			iter: MaybeUninit::uninit(),
			archetype_id: ArchetypeId::default(),
            // entities: std::ptr::null::<Entity>(),
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        _state: &Self::State,
        archetype: &Archetype,
    ) {
		self.iter.write(std::mem::transmute(archetype.entities.keys()));
		self.archetype_id = archetype.id();
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Option<Self::Item> {
		match self.iter.assume_init_mut().next() {
			Some(local) => Some(Entity::new(self.archetype_id, local)),
			None => None,
		} 
    }
}

impl<T: Component> WorldQuery for &T {
    type Fetch = ReadFetch<T>;
    type State = ReadState<T>;
}

pub struct ReadState<T> {
    component_id: ComponentId,
    storage_type: StorageType,
    marker: PhantomData<T>,
}

// SAFE: component access and archetype component access are properly updated to reflect that T is
// read
unsafe impl<T: Component> FetchState for ReadState<T> {
    fn init(world: &mut World) -> Self {
        let component_info = world.components.get_or_insert_info::<T>();
        ReadState {
            component_id: component_info.get_id(),
            storage_type: component_info.get_storage_type(),
            marker: PhantomData,
        }
    }

    fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
        if access.access().has_write(self.component_id) {
            panic!("&{} conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>());
        }
        access.add_read(self.component_id)
    }


    fn matches_archetype(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.component_id)
    }
}

pub struct ReadFetch<T> {
    storage_type: StorageType,
	container: MaybeUninit<NonNull<u8>>,
	mark: PhantomData<T>,
	
    // table_components: NonNull<T>,
    // entity_table_rows: *const usize,
    // entities: *const Entity,
    // sparse_set: *const ComponentSparseSet,
}

/// SAFE: access is read only
unsafe impl<T> ReadOnlyFetch for ReadFetch<T> {}

impl<'w, T: Component> Fetch<'w> for ReadFetch<T> {
    type Item = &'w T;
    type State = ReadState<T>;

    unsafe fn init(
        world: &World,
        state: &Self::State,
    ) -> Self {
        Self {
            storage_type: state.storage_type,
			container: MaybeUninit::uninit(),
			mark: PhantomData,
        }
    }

    #[inline]
    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &Archetype,
    ) {
		self.container.write(archetype.get_component(state.component_id));
    }

    #[inline]
    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Option<Self::Item> {
        match self.storage_type {
            StorageType::Table => {
				unsafe {std::mem::transmute((&mut *(self.container.as_ptr() as usize as *mut SecondaryMap<Local, Self::Item>)).get(Local::new(archetype_index))) }
            }
            StorageType::SparseSet => {
                unsafe {std::mem::transmute((&mut *(self.container.as_ptr() as usize as *mut SparseSecondaryMap<Local, Self::Item>)).get(Local::new(archetype_index)))}
            }
        }
    }
}

// impl<T: Component> WorldQuery for &mut T {
//     type Fetch = WriteFetch<T>;
//     type State = WriteState<T>;
// }

// pub struct WriteFetch<T> {
//     storage_type: StorageType,
//     table_components: NonNull<T>,
//     table_ticks: *mut ComponentTicks,
//     entities: *const Entity,
//     entity_table_rows: *const usize,
//     sparse_set: *const ComponentSparseSet,
//     last_change_tick: u32,
//     change_tick: u32,
// }

// pub struct WriteState<T> {
//     component_id: ComponentId,
//     storage_type: StorageType,
//     marker: PhantomData<T>,
// }

// // SAFE: component access and archetype component access are properly updated to reflect that T is
// // written
// unsafe impl<T: Component> FetchState for WriteState<T> {
//     fn init(world: &mut World) -> Self {
//         let component_info = world.components.get_or_insert_info::<T>();
//         WriteState {
//             component_id: component_info.id(),
//             storage_type: component_info.storage_type(),
//             marker: PhantomData,
//         }
//     }

//     fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
//         if access.access().has_read(self.component_id) {
//             panic!("&mut {} conflicts with a previous access in this query. Mutable component access must be unique.",
//                 std::any::type_name::<T>());
//         }
//         access.add_write(self.component_id);
//     }

//     fn update_archetype_component_access(
//         &self,
//         archetype: &Archetype,
//         access: &mut Access<ArchetypeComponentId>,
//     ) {
//         if let Some(archetype_component_id) =
//             archetype.get_archetype_component_id(self.component_id)
//         {
//             access.add_write(archetype_component_id);
//         }
//     }

//     fn matches_archetype(&self, archetype: &Archetype) -> bool {
//         archetype.contains(self.component_id)
//     }

//     fn matches_table(&self, table: &Table) -> bool {
//         table.has_column(self.component_id)
//     }
// }

// impl<'w, T: Component> Fetch<'w> for WriteFetch<T> {
//     type Item = Mut<'w, T>;
//     type State = WriteState<T>;

//     #[inline]
//     fn is_dense(&self) -> bool {
//         match self.storage_type {
//             StorageType::Table => true,
//             StorageType::SparseSet => false,
//         }
//     }

//     unsafe fn init(
//         world: &World,
//         state: &Self::State,
//         last_change_tick: u32,
//         change_tick: u32,
//     ) -> Self {
//         let mut value = Self {
//             storage_type: state.storage_type,
//             table_components: NonNull::dangling(),
//             entities: ptr::null::<Entity>(),
//             entity_table_rows: ptr::null::<usize>(),
//             sparse_set: ptr::null::<ComponentSparseSet>(),
//             table_ticks: ptr::null_mut::<ComponentTicks>(),
//             last_change_tick,
//             change_tick,
//         };
//         if state.storage_type == StorageType::SparseSet {
//             value.sparse_set = world
//                 .storages()
//                 .sparse_sets
//                 .get(state.component_id)
//                 .unwrap();
//         }
//         value
//     }

//     #[inline]
//     unsafe fn set_archetype(
//         &mut self,
//         state: &Self::State,
//         archetype: &Archetype,
//         tables: &Tables,
//     ) {
//         match state.storage_type {
//             StorageType::Table => {
//                 self.entity_table_rows = archetype.entity_table_rows().as_ptr();
//                 let column = tables[archetype.table_id()]
//                     .get_column(state.component_id)
//                     .unwrap();
//                 self.table_components = column.get_ptr().cast::<T>();
//                 self.table_ticks = column.get_ticks_mut_ptr();
//             }
//             StorageType::SparseSet => self.entities = archetype.entities().as_ptr(),
//         }
//     }

//     #[inline]
//     unsafe fn set_table(&mut self, state: &Self::State, table: &Table) {
//         let column = table.get_column(state.component_id).unwrap();
//         self.table_components = column.get_ptr().cast::<T>();
//         self.table_ticks = column.get_ticks_mut_ptr();
//     }

//     #[inline]
//     unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
//         match self.storage_type {
//             StorageType::Table => {
//                 let table_row = *self.entity_table_rows.add(archetype_index);
//                 Mut {
//                     value: &mut *self.table_components.as_ptr().add(table_row),
//                     component_ticks: &mut *self.table_ticks.add(table_row),
//                     change_tick: self.change_tick,
//                     last_change_tick: self.last_change_tick,
//                 }
//             }
//             StorageType::SparseSet => {
//                 let entity = *self.entities.add(archetype_index);
//                 let (component, component_ticks) =
//                     (*self.sparse_set).get_with_ticks(entity).unwrap();
//                 Mut {
//                     value: &mut *component.cast::<T>(),
//                     component_ticks: &mut *component_ticks,
//                     change_tick: self.change_tick,
//                     last_change_tick: self.last_change_tick,
//                 }
//             }
//         }
//     }

//     #[inline]
//     unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
//         Mut {
//             value: &mut *self.table_components.as_ptr().add(table_row),
//             component_ticks: &mut *self.table_ticks.add(table_row),
//             change_tick: self.change_tick,
//             last_change_tick: self.last_change_tick,
//         }
//     }
// }

// impl<T: WorldQuery> WorldQuery for Option<T> {
//     type Fetch = OptionFetch<T::Fetch>;
//     type State = OptionState<T::State>;
// }

// pub struct OptionFetch<T> {
//     fetch: T,
//     matches: bool,
// }

// /// SAFE: OptionFetch is read only because T is read only
// unsafe impl<T: ReadOnlyFetch> ReadOnlyFetch for OptionFetch<T> {}

// pub struct OptionState<T: FetchState> {
//     state: T,
// }

// // SAFE: component access and archetype component access are properly updated according to the
// // internal Fetch
// unsafe impl<T: FetchState> FetchState for OptionState<T> {
//     fn init(world: &mut World) -> Self {
//         Self {
//             state: T::init(world),
//         }
//     }

//     fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
//         self.state.update_component_access(access);
//     }

//     fn update_archetype_component_access(
//         &self,
//         archetype: &Archetype,
//         access: &mut Access<ArchetypeComponentId>,
//     ) {
//         if self.state.matches_archetype(archetype) {
//             self.state
//                 .update_archetype_component_access(archetype, access)
//         }
//     }

//     fn matches_archetype(&self, _archetype: &Archetype) -> bool {
//         true
//     }

//     fn matches_table(&self, _table: &Table) -> bool {
//         true
//     }
// }

// impl<'w, T: Fetch<'w>> Fetch<'w> for OptionFetch<T> {
//     type Item = Option<T::Item>;
//     type State = OptionState<T::State>;

//     #[inline]
//     fn is_dense(&self) -> bool {
//         self.fetch.is_dense()
//     }

//     unsafe fn init(
//         world: &World,
//         state: &Self::State,
//     ) -> Self {
//         Self {
//             fetch: T::init(world, &state.state),
//             matches: false,
//         }
//     }

//     #[inline]
//     unsafe fn set_archetype(
//         &mut self,
//         state: &Self::State,
//         archetype: &Archetype,
//         tables: &Tables,
//     ) {
//         self.matches = state.state.matches_archetype(archetype);
//         if self.matches {
//             self.fetch.set_archetype(&state.state, archetype, tables);
//         }
//     }

//     #[inline]
//     unsafe fn set_table(&mut self, state: &Self::State, table: &Table) {
//         self.matches = state.state.matches_table(table);
//         if self.matches {
//             self.fetch.set_table(&state.state, table);
//         }
//     }

//     #[inline]
//     unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
//         if self.matches {
//             Some(self.fetch.archetype_fetch(archetype_index))
//         } else {
//             None
//         }
//     }

//     #[inline]
//     unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
//         if self.matches {
//             Some(self.fetch.table_fetch(table_row))
//         } else {
//             None
//         }
//     }
// }

// /// Change trackers for component `T`
// #[derive(Clone)]
// pub struct ChangeTrackers<T: Component> {
//     pub(crate) component_ticks: ComponentTicks,
//     pub(crate) last_change_tick: u32,
//     pub(crate) change_tick: u32,
//     marker: PhantomData<T>,
// }
// impl<T: Component> std::fmt::Debug for ChangeTrackers<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ChangeTrackers")
//             .field("component_ticks", &self.component_ticks)
//             .field("last_change_tick", &self.last_change_tick)
//             .field("change_tick", &self.change_tick)
//             .finish()
//     }
// }

// impl<T: Component> ChangeTrackers<T> {
//     /// Has this component been added since the last execution of this system.
//     pub fn is_added(&self) -> bool {
//         self.component_ticks
//             .is_added(self.last_change_tick, self.change_tick)
//     }

//     /// Has this component been changed since the last execution of this system.
//     pub fn is_changed(&self) -> bool {
//         self.component_ticks
//             .is_changed(self.last_change_tick, self.change_tick)
//     }
// }

// impl<T: Component> WorldQuery for ChangeTrackers<T> {
//     type Fetch = ChangeTrackersFetch<T>;
//     type State = ChangeTrackersState<T>;
// }

// pub struct ChangeTrackersState<T> {
//     component_id: ComponentId,
//     storage_type: StorageType,
//     marker: PhantomData<T>,
// }

// // SAFE: component access and archetype component access are properly updated to reflect that T is
// // read
// unsafe impl<T: Component> FetchState for ChangeTrackersState<T> {
//     fn init(world: &mut World) -> Self {
//         let component_info = world.components.get_or_insert_info::<T>();
//         Self {
//             component_id: component_info.id(),
//             storage_type: component_info.storage_type(),
//             marker: PhantomData,
//         }
//     }

//     fn update_component_access(&self, access: &mut FilteredAccess<ComponentId>) {
//         if access.access().has_write(self.component_id) {
//             panic!("ChangeTrackers<{}> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
//                 std::any::type_name::<T>());
//         }
//         access.add_read(self.component_id)
//     }

//     fn update_archetype_component_access(
//         &self,
//         archetype: &Archetype,
//         access: &mut Access<ArchetypeComponentId>,
//     ) {
//         if let Some(archetype_component_id) =
//             archetype.get_archetype_component_id(self.component_id)
//         {
//             access.add_read(archetype_component_id);
//         }
//     }

//     fn matches_archetype(&self, archetype: &Archetype) -> bool {
//         archetype.contains(self.component_id)
//     }

//     fn matches_table(&self, table: &Table) -> bool {
//         table.has_column(self.component_id)
//     }
// }

// pub struct ChangeTrackersFetch<T> {
//     storage_type: StorageType,
//     table_ticks: *const ComponentTicks,
//     entity_table_rows: *const usize,
//     entities: *const Entity,
//     sparse_set: *const ComponentSparseSet,
//     marker: PhantomData<T>,
//     last_change_tick: u32,
//     change_tick: u32,
// }

// /// SAFE: access is read only  
// unsafe impl<T> ReadOnlyFetch for ChangeTrackersFetch<T> {}

// impl<'w, T: Component> Fetch<'w> for ChangeTrackersFetch<T> {
//     type Item = ChangeTrackers<T>;
//     type State = ChangeTrackersState<T>;

//     #[inline]
//     fn is_dense(&self) -> bool {
//         match self.storage_type {
//             StorageType::Table => true,
//             StorageType::SparseSet => false,
//         }
//     }

//     unsafe fn init(
//         world: &World,
//         state: &Self::State,
//         last_change_tick: u32,
//         change_tick: u32,
//     ) -> Self {
//         let mut value = Self {
//             storage_type: state.storage_type,
//             table_ticks: ptr::null::<ComponentTicks>(),
//             entities: ptr::null::<Entity>(),
//             entity_table_rows: ptr::null::<usize>(),
//             sparse_set: ptr::null::<ComponentSparseSet>(),
//             marker: PhantomData,
//             last_change_tick,
//             change_tick,
//         };
//         if state.storage_type == StorageType::SparseSet {
//             value.sparse_set = world
//                 .storages()
//                 .sparse_sets
//                 .get(state.component_id)
//                 .unwrap();
//         }
//         value
//     }

//     #[inline]
//     unsafe fn set_archetype(
//         &mut self,
//         state: &Self::State,
//         archetype: &Archetype,
//         tables: &Tables,
//     ) {
//         match state.storage_type {
//             StorageType::Table => {
//                 self.entity_table_rows = archetype.entity_table_rows().as_ptr();
//                 let column = tables[archetype.table_id()]
//                     .get_column(state.component_id)
//                     .unwrap();
//                 self.table_ticks = column.get_ticks_mut_ptr().cast::<ComponentTicks>();
//             }
//             StorageType::SparseSet => self.entities = archetype.entities().as_ptr(),
//         }
//     }

//     #[inline]
//     unsafe fn set_table(&mut self, state: &Self::State, table: &Table) {
//         self.table_ticks = table
//             .get_column(state.component_id)
//             .unwrap()
//             .get_ticks_mut_ptr()
//             .cast::<ComponentTicks>();
//     }

//     #[inline]
//     unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
//         match self.storage_type {
//             StorageType::Table => {
//                 let table_row = *self.entity_table_rows.add(archetype_index);
//                 ChangeTrackers {
//                     component_ticks: *self.table_ticks.add(table_row),
//                     marker: PhantomData,
//                     last_change_tick: self.last_change_tick,
//                     change_tick: self.change_tick,
//                 }
//             }
//             StorageType::SparseSet => {
//                 let entity = *self.entities.add(archetype_index);
//                 ChangeTrackers {
//                     component_ticks: *(*self.sparse_set).get_ticks(entity).unwrap(),
//                     marker: PhantomData,
//                     last_change_tick: self.last_change_tick,
//                     change_tick: self.change_tick,
//                 }
//             }
//         }
//     }

//     #[inline]
//     unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
//         ChangeTrackers {
//             component_ticks: *self.table_ticks.add(table_row),
//             marker: PhantomData,
//             last_change_tick: self.last_change_tick,
//             change_tick: self.change_tick,
//         }
//     }
// }

macro_rules! impl_tuple_fetch {
    ($(($name: ident, $state: ident)),*) => {
        #[allow(non_snake_case)]
        impl<'a, $($name: Fetch<'a>),*> Fetch<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);
            type State = ($($name::State,)*);

            unsafe fn init(_world: &World, state: &Self::State) -> Self {
                let ($($name,)*) = state;
                ($($name::init(_world, $name),)*)
            }

            #[inline]
            unsafe fn set_archetype(&mut self, _state: &Self::State, _archetype: &Archetype) {
                let ($($name,)*) = self;
                let ($($state,)*) = _state;
                $($name.set_archetype($state, _archetype);)*
            }

            #[inline]
            unsafe fn archetype_fetch(&mut self, _archetype_index: usize) -> Option<Self::Item> {
                let ($($name,)*) = self;
                Some(($(match $name.archetype_fetch(_archetype_index) {
					Some(r) => r,
					None => return None
				},)*))
            }
        }

        // SAFE: update_component_access and update_archetype_component_access are called for each item in the tuple
        #[allow(non_snake_case)]
        unsafe impl<$($name: FetchState),*> FetchState for ($($name,)*) {
            fn init(_world: &mut World) -> Self {
                ($($name::init(_world),)*)
            }

            fn update_component_access(&self, _access: &mut FilteredAccess<ComponentId>) {
                let ($($name,)*) = self;
                $($name.update_component_access(_access);)*
            }


            fn matches_archetype(&self, _archetype: &Archetype) -> bool {
                let ($($name,)*) = self;
                true $(&& $name.matches_archetype(_archetype))*
            }
        }

        impl<$($name: WorldQuery),*> WorldQuery for ($($name,)*) {
            type Fetch = ($($name::Fetch,)*);
            type State = ($($name::State,)*);
        }

        /// SAFE: each item in the tuple is read only
        unsafe impl<$($name: ReadOnlyFetch),*> ReadOnlyFetch for ($($name,)*) {}

    };
}

all_tuples!(impl_tuple_fetch, 0, 15, F, S);

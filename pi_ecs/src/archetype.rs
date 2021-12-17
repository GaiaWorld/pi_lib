/// 原型

use crate::{
    component::{ComponentId, StorageType, Component, ComponentInfo},
    entity::Entity,
    storage::{Offset, LocalVersion, Local},
};
use std::{
    borrow::Cow,
    hash::Hash,
    ops::{Index, IndexMut}, any::TypeId,
	ptr::NonNull,
};
use slotmap::{DenseSlotMap, SecondaryMap, SparseSecondaryMap};
use map::{Map};
use hash::XHashMap;

pub type ArchetypeId = Local;

pub enum ComponentStatus {
    Added,
    Mutated,
}

pub struct Archetype {
    id: ArchetypeId,
    pub(crate) entities: DenseSlotMap<LocalVersion, ()>,
	// SecondaryMap<ComponentId, SecondaryMap | SparseSecondaryMap>
	components: SecondaryMap<ComponentId, NonNull<u8>>,
	component_ids: Cow<'static, [ComponentId]>,
}

impl Archetype {
	pub fn create_entity(&mut self) -> Entity {
		Entity::new(self.id, self.entities.insert(()))
	}

	pub fn insert_component<C: Component>(&mut self, local: LocalVersion, value: C, id: ComponentId, storage_type: StorageType) {
		let container = unsafe{ self.get_component(id) };
		match storage_type {
			StorageType::Table => unsafe {(&mut *(container.as_ptr() as usize as *mut SecondaryMap<LocalVersion, C>)).insert(local, value) },
			StorageType::SparseSet => unsafe {(&mut *(container.as_ptr() as usize as *mut SparseSecondaryMap<LocalVersion, C>)).insert(local, value) }
		};
	}

	pub fn new(id: ArchetypeId, component_ids: Cow<'static, [ComponentId]>) -> Self {
		Self {
			id,
			entities: DenseSlotMap::default(),
			components: SecondaryMap::with_capacity(component_ids.len()),
			component_ids: component_ids
		}
	}

	pub fn add_component_type<T: Component, M: Map<Key = LocalVersion, Val=T>>(&mut self, id: ComponentId, mut m: M) {
		self.components.insert(
			id,
			NonNull::new(Box::into_raw(Box::new(m)).cast::<u8>()).unwrap()
		);
	}

    #[inline]
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

	#[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

	#[inline]
	pub unsafe fn get_component(&self, id: ComponentId) -> NonNull<u8> {
		unsafe{self.components.get_unchecked(id)}.clone()
	}


    #[inline]
    pub fn component_ids(&self) -> &[ComponentId] {
        &self.component_ids
    }

	pub fn contains(&self, component_id: ComponentId) -> bool {
        self.components.contains_key(component_id)
    }
}

/// A generational id that changes every time the set of archetypes changes
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArchetypeGeneration(usize);

impl ArchetypeGeneration {
    #[inline]
    pub fn new(generation: usize) -> Self {
        ArchetypeGeneration(generation)
    }

    #[inline]
    pub fn value(self) -> usize {
        self.0
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum ArchetypeIdentity {
	Identity(TypeId),
	Components(Cow<'static, [ComponentId]>),
}

pub type ArchetypeComponentId = Local;

pub struct Archetypes {
    pub(crate) archetypes: Vec<Archetype>,
    pub(crate) archetype_component_count: usize,
    archetype_ids: XHashMap<ArchetypeIdentity, ArchetypeId>,
}

impl Archetypes {
	pub fn new() -> Self {
		Self {
			archetypes: Vec::new(),
			archetype_component_count: 0,
			archetype_ids: XHashMap::default(),
		}
	}

	pub(crate) fn spawn<E: Send + Sync + 'static>(&mut self, id: ArchetypeId) -> Entity {
		self.archetypes[id.offset()].create_entity()
    }

    #[inline]
    pub fn generation(&self) -> ArchetypeGeneration {
        ArchetypeGeneration(self.archetypes.len())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }

    #[inline]
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.offset())
    }

    #[inline]
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.offset())
    }

    #[inline]
    pub(crate) fn get_2_mut(
        &mut self,
        a: ArchetypeId,
        b: ArchetypeId,
    ) -> (&mut Archetype, &mut Archetype) {
        if a.offset() > b.offset() {
            let (b_slice, a_slice) = self.archetypes.split_at_mut(a.offset());
            (&mut a_slice[0], &mut b_slice[b.offset()])
        } else {
            let (a_slice, b_slice) = self.archetypes.split_at_mut(b.offset());
            (&mut a_slice[a.offset()], &mut b_slice[0])
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

	#[inline]
    pub fn get_id_by_ident(&self, type_id: TypeId) -> Option<&ArchetypeId> {
        self.archetype_ids.get(&ArchetypeIdentity::Identity(type_id))
    }

	pub fn get_id_or_insert_by_ident(&mut self, type_id: TypeId, components: Vec<ComponentId>, component_infos: &Vec<ComponentInfo>) -> ArchetypeId {
		let components = Cow::from(components);

        let archetypes = &mut self.archetypes;
		*self
			.archetype_ids
			.entry(ArchetypeIdentity::Identity(type_id))
			.or_insert_with(move || {
				let id = ArchetypeId::new(archetypes.len());
				archetypes.push(Archetype::new(
					id,
					components.clone()
				));

				for c in components.iter() {
					match component_infos[c.offset()].storage_type {
						StorageType::SparseSet => {},
						StorageType::Table => {},
					}
				}

				id
			})
    }

    ///
    /// # Safety
    /// TableId must exist in tables
    pub(crate) fn get_id_or_insert_by_components(
        &mut self,
        components: Vec<ComponentId>,
		component_infos: &Vec<ComponentInfo>,
    ) -> ArchetypeId {
        let components = Cow::from(components);
        let archetype_identity = ArchetypeIdentity::Components(components.clone());

        let archetypes = &mut self.archetypes;
        *self
            .archetype_ids
            .entry(archetype_identity)
            .or_insert_with(move || {
                let id = ArchetypeId::new(archetypes.len());
                archetypes.push(Archetype::new(
                    id,
					components.clone()
                ));

				for c in components.iter() {
					match component_infos[c.offset()].storage_type {
						StorageType::SparseSet => {},
						StorageType::Table => {},
					}
				}
                id
            })
    }
}

impl Index<ArchetypeId> for Archetypes {
    type Output = Archetype;

    #[inline]
    fn index(&self, index: ArchetypeId) -> &Self::Output {
        &self.archetypes[index.offset()]
    }
}

impl IndexMut<ArchetypeId> for Archetypes {
    #[inline]
    fn index_mut(&mut self, index: ArchetypeId) -> &mut Self::Output {
        &mut self.archetypes[index.offset()]
    }
}



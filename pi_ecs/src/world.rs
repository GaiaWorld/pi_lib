use std::any::TypeId;
use std::collections::HashSet;
/// 世界

use std::sync::atomic::{AtomicU32, Ordering};

use pi_ecs_macros::all_tuples;

use crate::archetype::{Archetype, Archetypes};
use crate::component::{Components, ComponentId, Component};
use crate::entity::Entity;

pub struct World {
	id:WorldId,
    pub(crate) components: Components,
    pub(crate) archetypes: Archetypes,
    // pub(crate) storages: Storages,
    // pub(crate) bundles: Bundles,
    // pub(crate) removed_components: SparseSet<ComponentId, Vec<Entity>>,
    // Access cache used by [WorldCell].
    // pub(crate) archetype_component_access: ArchetypeComponentAccess,
    // main_thread_validator: MainThreadValidator,

	change_tick: AtomicU32,
    last_change_tick: u32,
}

impl World {
	pub fn new_archetype<T: Send + Sync + 'static>(&mut self) -> ArchetypeInfo {
		if let Some(r) = self.archetypes.get_id_by_ident(TypeId::of::<T>()) {
			panic!("new_archetype fial");
		}
		ArchetypeInfo {
			world: self,
			type_id: TypeId::of::<T>(),
			components: HashSet::default(),
		}
	}

	pub fn spawn<T: Send + Sync + 'static>(&mut self) -> Entity {
		let archetype_id = match self.archetypes.get_id_by_ident(TypeId::of::<T>()) {
			Some(r) => r,
			None => {
				panic!("spawn fial")
			}
		};
		
		self.archetypes.spawn::<T>(*archetype_id)
	}

	pub fn archetypes(&self) -> &Archetypes {
		&self.archetypes
	}
	pub fn id(&self) -> WorldId {
        self.id
    }
	pub fn read_change_tick(&self) -> u32 {
        self.change_tick.load(Ordering::Acquire)
    }

    #[inline]
    pub fn change_tick(&mut self) -> u32 {
        *self.change_tick.get_mut()
    }

    #[inline]
    pub fn last_change_tick(&self) -> u32 {
        self.last_change_tick
    }
}

#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct WorldId(usize);

pub struct ArchetypeInfo<'a> {
	pub(crate) world: &'a mut World,
	pub(crate) type_id: TypeId,
	pub(crate) components: HashSet<ComponentId>,
}

impl<'a> ArchetypeInfo<'a> {
	pub fn add<C: Component>(&mut self) -> &mut Self{
		let id = self.world.components.get_or_insert_id::<C>();
		self.components.insert(id);

		self
	}

	pub fn create(&mut self) {
		let components = self.components.iter().map(|r| {r.clone()}).collect();
		self.world.archetypes.get_id_or_insert_by_ident(self.type_id, components, &self.world.components.infos);
	}
}
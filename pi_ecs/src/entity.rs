use std::convert::From;
use crate::storage::LocalVersion;
use crate::archetype::ArchetypeId;

// 实体

// pub struct Entity(u64);

pub struct Entity {
	archetype_id: ArchetypeId,
	local: LocalVersion,
}

impl Entity {
	pub(crate) fn new(archetype_id: ArchetypeId, local: LocalVersion) -> Self {
		Self{archetype_id, local}
	}

	pub fn archetype_id(&self) -> ArchetypeId {
		self.archetype_id
	}

	pub fn local(&self) -> LocalVersion {
		self.local
	}
}


// impl From<EntityInfo> for Entity {
// 	fn from(info: EntityInfo) -> Self {
// 		let idx: u64 = *info.local & 0xffff_ffff;
// 		let version: u64 = (*info.local >> 32) | 1; // Ensure version is odd.
		

// 		// archetype_id 8位，version 26位，idx 28位
// 		return Entity((*info.archetype_id as u64) << 56 + version << 36 + idx);
// 	}
// }


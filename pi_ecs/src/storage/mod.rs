pub use slotmap::{Key, KeyData, SlotMap, SecondaryMap, SparseSecondaryMap, DenseSlotMap};
pub use slotmap::dense::{Iter, IterMut, Keys, Values};
use std::convert::From;
use std::ops::Deref;

pub trait Offset: Clone {
	fn offset(&self) -> usize;
}
pub trait FromOffset: Offset {
	fn from_offset(offset: usize) -> Self;
}


#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct LocalVersion(u64);

impl Deref for LocalVersion {
	type Target = u64;

    fn deref(&self) -> &Self::Target {
		&self.0
	}
}
 
unsafe impl Key for LocalVersion {
	#[inline]
    fn data(&self) -> KeyData {
		KeyData::from_ffi(self.0)
	}
}

impl From<KeyData> for LocalVersion {
	#[inline]
    fn from(data: KeyData) -> Self {
		LocalVersion(data.as_ffi())
	}
}

impl Offset for LocalVersion {
	#[inline]
    fn offset(&self) -> usize {
		(self.0 << 32 >> 32) as usize
	}
}


#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Local(usize);

impl Deref for Local {
	type Target = usize;

    fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Local {
	#[inline]
	pub fn new(v: usize) -> Self {
		Self(v)
	}	
}

unsafe impl Key for Local {
	#[inline]
    fn data(&self) -> KeyData {
		KeyData::from_ffi(self.0 as u64 | 1 << 32)
	}
}

impl Offset for Local{
	#[inline]
    fn offset(&self) -> usize {
		self.0
	}
}

impl FromOffset for Local{
	#[inline]
    fn from_offset(offset: usize) -> Self {
		Local(offset)
	}
}

impl From<KeyData> for Local {
	#[inline]
    fn from(data: KeyData) -> Self {
		Local(data.as_ffi() as usize)
	}
}


//! 定义数据结构HashMap（实际上是对`hash::XHashMap`的包装）
//! 为`HashMap`实现`HashMap`trait

use std::hash::Hash;

use hash::XHashMap;

use crate::Map;

/// HashMap
pub struct HashMap<K: Eq + Hash, V>(XHashMap<K, V>);

impl<K: Hash + Eq, V> Map for HashMap<K, V>{
    type Key = K;
	type Val = V;

    #[inline]
    fn len(&self) -> usize{
        self.0.len()
    }

    #[inline]
    fn capacity(&self) -> usize{
        self.0.capacity()
    }

    #[inline]
    fn mem_size(&self) -> usize{
        self.0.capacity() * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
    }

    #[inline]
    fn contains(&self, key: &Self::Key) -> bool{
        self.0.contains_key(key)
    }

    #[inline]
    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.0.get(key)
    }

    #[inline]
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.0.get_mut(key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.0.get(key).unwrap()
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.0.get_mut(key).unwrap()
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.0.remove(key).unwrap()
    }

    #[inline]
    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val> {
        self.0.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.0.remove(key)
	}
	
	fn with_capacity(_capacity: usize) -> Self {
		Self::default()
	}
}

impl<K: Hash + Eq, V> Default for HashMap<K, V> {
    fn default() -> Self{
        HashMap(XHashMap::default())
    }
}
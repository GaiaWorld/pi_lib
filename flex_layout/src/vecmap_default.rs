
use std::ops::{Index, IndexMut};
use std::default::Default;

use ecs::component::GetDefault;
use map::vecmap::VecMap;
use map::Map;


pub struct VecMapWithDefault<T> {
	default_v: T,
	map: VecMap<T>,
}

impl<T> VecMapWithDefault<T> {
	#[inline]
	pub fn clear(&mut self) {
		self.map.clear();
	}
}

impl<T> VecMapWithDefault<T> {
	pub fn set_default(&mut self, v: T) {
		self.default_v = v;
	}
}

impl<T: Default> Default for VecMapWithDefault<T> {
	fn default() -> Self {
		VecMapWithDefault {
			default_v: T::default(),
			map: VecMap::default(),
		}
	}
}

impl<T: Default> Index<usize> for VecMapWithDefault<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        match self.map.get(index) {
			Some(r) => r,
			None => &self.default_v
		}
    }
}

impl<T: Clone + Default> IndexMut<usize> for VecMapWithDefault<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
		unsafe { Map::get_unchecked_mut(self, &index) }
    }
}
impl<T: Clone+ Default> GetDefault<T> for VecMapWithDefault<T> {
    fn get_default(&self) -> Option<&T> {
        Some(&self.default_v)
    }

    fn get_default_mut(&mut self) -> Option<&mut T> {
        Some(&mut self.default_v)
    }
}

impl<T: Clone+ Default> Map for VecMapWithDefault<T> {
	type Key = usize;
	type Val = T;

	#[inline]
	fn with_capacity(capacity: usize) -> VecMapWithDefault<T> {
        VecMapWithDefault {
            default_v: T::default(),
			map: VecMap::with_capacity(capacity),
        }
    }
	
    #[inline]
    fn get(&self, key: &usize) -> Option<&T> {
        self.map.get(*key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.map.get_mut(*key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &usize) -> &T {
        self.map.get_unchecked(*key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &usize) -> &mut T {
		// 这个get了两次，优化TODO
		if self.map.get(*key).is_some() {
			return &mut self.map[*key];
		}
		
		self.map.insert(*key, self.default_v.clone());
		&mut self.map[*key]
        // self.map.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &usize) -> T {
        self.map.remove_unchecked(*key)
    }

    #[inline]
    fn insert(&mut self, key: usize, val: T) -> Option<T> {
        self.map.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<T> {
        self.map.remove(*key)
    }

    #[inline]
    fn contains(&self, key: &usize) -> bool {
        self.map.contains(*key)
    }

    #[inline]
    fn len(&self) -> usize {
        self.map.len()
    }
    #[inline]
    fn capacity(&self) -> usize {
        self.map.capacity()
    }
    #[inline]
    fn capacity_mem_size(&self) -> usize {
        self.map.capacity() * std::mem::size_of::<T>()
    }

    #[inline]
    fn use_mem_size(&self) -> usize {
        self.map.len() * std::mem::size_of::<T>()
    }
}

// impl<T: Default> VecMapWithDefault<T> {
// 	#[inline]
//     fn get_mut1<'a>(&'a mut self, key: &'a usize) -> Option<&'a mut T> {
// 		if self.map.get(*key).is_some() {
// 			return self.map.get_mut(*key);
// 		}
		
// 		self.map.insert(*key, T::default());
// 		self.map.get_mut(*key)
//         // self.map.get_mut(*key)
//     }
// }

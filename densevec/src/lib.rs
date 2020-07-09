
extern crate map;

use std::ops::{Index, IndexMut};
use std::mem::replace;

use map::Map;
use map::vecmap::VecMap;

#[derive(Default, Debug)]
pub struct DenseVecMap<T> {
    data_id: VecMap<usize>,
    data: Vec<T>,
    indexs: Vec<usize>,
}

impl<T> DenseVecMap<T> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    fn get(&self, id: usize) -> Option<&T> {
        match self.data_id.get(id) {
            Some(id) => Some(unsafe { self.data.get_unchecked(*id) } ),
            None => None,
        }  
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        match self.data_id.get(id) {
            Some(id) => Some(unsafe { self.data.get_unchecked_mut(*id) } ),
            None => None,
        }  
    }

    fn remove(&mut self, id: usize) -> Option<T> {
        match self.data_id.get(id) {
            Some(_) => Some(unsafe { self.remove_unchecked(id) } ),
            None => None,
        }  
    }

    fn contains(&self, id: usize) -> bool {
        match self.data_id.get(id) {
            Some(_) => true,
            None => false,
        }
    }

    fn insert(&mut self, id: usize, v: T) -> Option<T> {
        match self.data_id.get(id){
            Some(i) => {
                Some(replace(&mut self.data[*i], v))
            },
            None => {
                self.data_id.insert(id, self.data.len());
                self.data.push(v);
                self.indexs.push(id);
                None
            }
        }
    }

    unsafe fn get_unchecked(&self, id: usize) -> &T {
        let did = *self.data_id.get_unchecked(id);
        self.data.get_unchecked(did)
    }

    unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut T {
        let did = *self.data_id.get_unchecked(id);
        self.data.get_unchecked_mut(did)
    }

    unsafe fn remove_unchecked(&mut self, id: usize) -> T {
        let did = *self.data_id.get_unchecked(id);
        let last = *self.indexs.last().unwrap();
        let r = self.data.swap_remove(did);
        self.indexs.swap_remove(did);
        
        let len = self.data.len();
        if len > 0 {
            self.data_id.replace(id, len);
            self.data_id.remove_unchecked(last);
        }else {
            self.data_id.remove_unchecked(id);
        }
        r
    }
}

impl<T> Map for DenseVecMap<T> {
	type Key = usize;
	type Val = T;
    #[inline]
    fn get(&self, key: &usize) -> Option<&T> {
        self.get(*key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.get_mut(*key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &usize) -> &T {
        self.get_unchecked(*key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &usize) -> &mut T {
        self.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &usize) -> T {
        self.remove_unchecked(*key)
    }

    #[inline]
    fn insert(&mut self, key: usize, val: T) -> Option<T> {
        self.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<T> {
        self.remove(*key)
    }

    #[inline]
    fn contains(&self, key: &usize) -> bool {
        self.contains(*key)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    #[inline]
    fn mem_size(&self) -> usize {
        self.data_id.mem_size() + self.data.capacity() * std::mem::size_of::<T>() + self.indexs.capacity() * std::mem::size_of::<usize>()
    }
}

impl<T> Index<usize> for DenseVecMap<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        let did = self.data_id[index];
        &self.data[did]
    }
}

impl<T> IndexMut<usize> for DenseVecMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let did = self.data_id[index];
        &mut self.data[did]
    }
}


#[test]
fn test(){
    let mut vec = DenseVecMap::default();
    vec.insert(2, 2);
    vec.insert(10, 10);
    vec.insert(20, 20);

    println!("{:?}", vec);
    assert_eq!(2, *unsafe { vec.get_unchecked(2) });
    assert_eq!(10, *unsafe { vec.get_unchecked(10) });
    assert_eq!(20, *unsafe { vec.get_unchecked(20) });

    vec.remove(2);
    println!("{:?}", vec);
}
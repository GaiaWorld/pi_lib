/**
 * index trait and default impl
 */
extern crate slab;
extern crate map;
extern crate ver_index;

use std::fmt::{Debug, Formatter, Result as FResult};
use std::ops::{Deref, DerefMut};

use slab::Slab;
use map::Map;
use ver_index::VerIndex;

pub struct Item<C, T> {
    pub index: usize,
    pub class: C,
    pub value: T,
}
pub struct IndexClassFactory<C, T, I:VerIndex>(Slab<Item<C, T>, I>);

impl<C, T, I: VerIndex + Default> Default for IndexClassFactory<C, T, I> {
    fn default() -> Self {
        IndexClassFactory(Slab::default())
    }
}

impl<C, T, I:VerIndex> IndexClassFactory<C, T, I> {
    pub fn reserve (&mut self, additional: usize) {
        self.0.reserve(additional)
    }
    pub fn create (&mut self, index: usize, class: C, value: T) -> I::ID {
        self.0.insert(Item{index, class, value})
    }
}

impl<C, T, I:VerIndex> Deref for IndexClassFactory<C, T, I>{
    type Target = Slab<Item<C, T>, I>;

    fn deref (&self) -> &Self::Target{
        &self.0
    }
}

impl<C, T, I:VerIndex> DerefMut for IndexClassFactory<C, T, I>{
    fn deref_mut (&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

impl<C, T, I:VerIndex> Map for IndexClassFactory<C, T, I>{
	type Key = I::ID;
	type Val = usize;
    fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        self.0.contains(*key)
    }
    fn get(&self, key: &Self::Key) -> Option<&usize>{
        match self.0.get(*key) {
            Some(elem) => Some(&elem.index),
            None => None,
        }
    }
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut usize>{
        match self.0.get_mut(*key) {
            Some(elem) => Some(&mut elem.index),
            None => None,
        }
    }
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &usize {
        &self.0.get_unchecked(*key).index
    }
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut usize {
        &mut self.0.get_unchecked_mut(*key).index
    }
    fn insert(&mut self, key: Self::Key, value: usize) -> Option<usize>{
        match self.0.get_mut(key) {
            Some(elem) => {
                let r = Some(elem.index);
                elem.index = value;
                r
            },
            None => None,
        }
    }
    fn remove(&mut self, key: &Self::Key) -> Option<usize>{
        match self.0.remove(*key) {
            Some(elem) => {
                let r = Some(elem.index);
                r
            },
            None => None,
        }
    }
    fn clear(&mut self){
        self.0.clear()
    }

}
impl<C: Debug, T: Debug> Debug for Item<C, T> {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "Item{{index: {}, class: {:?}, value: {:?} }}",
               self.index,
               self.class,
               self.value,
        )
    }
}
impl<C: Debug, T: Debug, I:VerIndex> Debug for IndexClassFactory<C, T, I> {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "ClassFactory({:?})",
               self.0,
        )
    }
}
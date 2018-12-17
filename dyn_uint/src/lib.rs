/**
 * index trait and default impl
 */
extern crate slab;

use std::fmt::{Debug, Formatter, Result as FResult};
use std::ops::{Deref, DerefMut};

use slab::Slab;

pub trait UintFactory{
    fn load(&self, index: usize) -> usize;
    fn try_load(&self, index: usize) -> Option<usize>;
	fn store(&mut self, index: usize, value: usize);
    fn try_store(&mut self, index: usize, value: usize) -> bool;
}

pub trait ClassFactory<C>{
    fn get_class(&self, index: usize) -> &C;
	fn set_class(&mut self, index: usize, value: C);
}

pub struct SlabFactory<C, T>(Slab<(usize , C, T)>);

impl<C, T> SlabFactory<C, T> {
    pub fn new () -> SlabFactory<C, T>{
        SlabFactory(Slab::new())
    }

    pub fn create (&mut self, uint: usize, class: C, attach: T) -> usize{
        self.0.insert((uint, class, attach))
    }

    pub fn destroy (&mut self, index: usize){
       self.0.remove(index);
    }
}

impl<C, T> Deref for SlabFactory<C, T>{
    type Target = Slab<(usize, C, T)>;

    fn deref (&self) -> &Self::Target{
        &self.0
    }
}

impl<C, T> DerefMut for SlabFactory<C, T>{
    fn deref_mut (&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

impl<C, T> UintFactory for SlabFactory<C, T>{
    fn load(&self, index: usize) -> usize {
        unsafe{ self.0.get_unchecked(index).0 }
    }

    fn try_load(&self, index: usize) -> Option<usize>{
        match self.0.get(index) {
            Some(elem) => Some(elem.0),
            None => None,
        }
    }

	fn store(&mut self, index: usize, value: usize){
        unsafe{ self.0.get_unchecked_mut(index).0 = value };
    }

    fn try_store(&mut self, index: usize, value: usize) -> bool{
        match self.0.get_mut(index) {
            Some(elem) => {
                elem.0 = value;
                true
            },
            None => false,
        }
    }
}

impl<C, T> ClassFactory<C> for SlabFactory<C, T>{
    fn set_class(&mut self, index: usize, value: C) {
        unsafe{ self.0.get_unchecked_mut(index).1 = value };
    }

    fn get_class(&self, index: usize) -> &C {
        unsafe{&self.0.get_unchecked(index).1 }
    }
}

impl<C: Debug, T: Debug> Debug for SlabFactory<C, T> {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "DynUintFactory({:?})",
               self.0,
        )
    }
}
extern crate im;
extern crate share;

use std::{
    ops::{Deref, DerefMut},
    default::Default,
};
use im::vector::Vector;
use share::Share;

pub trait Listener<E> {
    fn listen(&self, e: &E);
}

pub struct FnListener<E>(pub Share<dyn Fn(&E)>);

unsafe impl<E> Sync for FnListener<E> {}
unsafe impl<E> Send for FnListener<E> {}

impl<E> Listener<E> for FnListener<E> {
    fn listen(&self, e: &E) {
        self.0(e)
    }
}
impl<E> PartialEq for FnListener<E> {
    fn eq(&self, other: &Self) -> bool {
        Share::ptr_eq(&self.0, &other.0)
    }
}
impl<E> Clone for FnListener<E> {
    fn clone(&self) -> Self {
        FnListener(self.0.clone())
    }
}

pub type FnListeners<E> = Listeners<FnListener<E>>;

#[derive(Clone)]
pub struct Listeners<T: Clone> (Vector<T>);

impl<T: Clone + PartialEq> Listeners<T> {
    pub fn mem_size(&self) -> usize {
        self.0.len() * std::mem::size_of::<T>()
    }
    pub fn delete(&mut self, listener: &T) -> bool {
		match self.0.index_of(listener) {
			Some(i) => {
                self.0.remove(i);
                true
            },
			_ => false,
		}
    }
}
impl<T: Clone + Listener<E>, E> Listener<E> for Listeners<T> {
    fn listen(&self, e: &E) {
        for l in self.0.iter() {
            // let time = std::time::Instant::now();
            l.listen(e);
            // println!("listen time----------{:?}", std::time::Instant::now() - time);
        }
    }
}

impl<T: Clone> Default for Listeners<T> {
    fn default() -> Self{
        Listeners(Vector::new())
    }
}

impl<T: Clone> Deref for Listeners<T> {
    type Target=Vector<T>;
    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl<T: Clone> DerefMut for Listeners<T> {
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

extern crate im;

use std::{
    sync::Arc,
    ops::{Deref, DerefMut},
    default::Default,
};
use im::vector::Vector;

pub trait Listener<E> {
    fn listen(&self, e: &E);
}

pub struct FnListener<E>(pub Arc<Fn(&E)>);

impl<E> Listener<E> for FnListener<E> {
    fn listen(&self, e: &E) {
        self.0(e)
    }
}
impl<E> PartialEq for FnListener<E> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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
            l.listen(e)
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

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::default::Default;

pub struct StdCell<T>(RefCell<T>);

impl<T> StdCell<T> {
    pub fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }
}

impl<T> Deref for StdCell<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for StdCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default> Default for StdCell<T> {
    fn default() -> Self {
        StdCell::new(T::default())
    }
}

unsafe impl<T> Send for StdCell<T>{}
unsafe impl<T> Sync for StdCell<T>{}
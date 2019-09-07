#[cfg(not(feature = "arc"))]
use std::rc::{Rc, Weak};
#[cfg(not(feature = "arc"))]
use std::cell::{RefCell, Ref, RefMut, BorrowError, BorrowMutError};
#[cfg(not(feature = "arc"))]
pub type Share<T> = Rc<T>;
#[cfg(not(feature = "arc"))]
pub type ShareWeak<T> = Weak<T>;
#[cfg(not(feature = "arc"))]
pub type ShareMutex<T> = RwCell<T>;
#[cfg(not(feature = "arc"))]
pub type ShareRwLock<T> = RwCell<T>;
#[cfg(not(feature = "arc"))]
pub type ShareCell<T> = RefCell<T>;

#[cfg(feature = "arc")]
use std::sync::{Arc, Weak, Mutex, RwLock};
#[cfg(feature = "arc")]
pub type Share<T> = Arc<T>;
#[cfg(feature = "arc")]
pub type ShareWeak<T> = Weak<T>;
#[cfg(feature = "arc")]
pub type ShareMutex<T> = Mutex<T>;
#[cfg(feature = "arc")]
pub type ShareRwLock<T> = RwLock<T>;
#[cfg(feature = "arc")]
pub type ShareCell<T> = cell::TrustCell<T>;


pub mod cell;


pub struct RwCell<T: ?Sized>(RefCell<T>);
unsafe impl<T> Sync for RwCell<T> where T: Sync {}
unsafe impl<T> Send for RwCell<T> where T: Send {}

impl<T> RwCell<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        RwCell(RefCell::new(value))
    }
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T: ?Sized> RwCell<T> {
    pub fn is_poisoned(&self) -> bool {
        false
    }
    pub fn get_mut(&mut self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
    pub fn read(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.0.try_borrow()
    }
    pub fn try_read(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.0.try_borrow()
    }
    pub fn write(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
    pub fn try_write(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
    pub fn lock(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
    pub fn try_lock(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
}

impl<T: Default> Default for RwCell<T> {
    fn default() -> Self {
        RwCell::new(Default::default())
    }
}

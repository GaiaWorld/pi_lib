//! 提供五个类型：`Share`， `ShareWeak`,，`ShareMutex`, `ShareRwLock`，`ShareCell`.
//! 
//! 在feature="rc"时：
//! 
//! * `Share`等同于`std::rc::Rc`
//! * `ShareWeak`等同于`std::rc::Weak`
//! * `ShareMutex`等同于`std::cell::RwCell`
//! * `ShareRwLock`等同于`std::cell::RwCell`
//! * `ShareCell`等同于`std::cell::RefCell`
//! 
//! 在feature="arc"时:
//! 
//! * `Share`等同于`std::sync::Arc`,
//! * `ShareWeak`等同于`std::sync::Weak`.
//! * `ShareMutex`等同于`std::sync::Mutex`
//! * `ShareRwLock`等同于`std::sync::RwCell`
//! * `ShareCell`等同于`cell::TrustCell`
use std::cell::{RefCell, Ref, RefMut, BorrowError, BorrowMutError};

#[cfg(feature = "rc")]
use std::rc::{Rc, Weak};
#[cfg(feature = "rc")]
pub type Share<T> = Rc<T>;
#[cfg(feature = "rc")]
pub type ShareWeak<T> = Weak<T>;
#[cfg(feature = "rc")]
pub type ShareMutex<T> = RwCell<T>;
#[cfg(feature = "rc")]
pub type ShareRwLock<T> = RwCell<T>;
#[cfg(feature = "rc")]
pub type ShareCell<T> = RefCell<T>;

#[cfg(not(feature = "rc"))]
use std::sync::{Arc, Weak, Mutex, RwLock};
#[cfg(not(feature = "rc"))]
pub type Share<T> = Arc<T>;
#[cfg(not(feature = "rc"))]
pub type ShareWeak<T> = Weak<T>;
#[cfg(not(feature = "rc"))]
pub type ShareMutex<T> = Mutex<T>;
#[cfg(not(feature = "rc"))]
pub type ShareRwLock<T> = RwLock<T>;
#[cfg(not(feature = "rc"))]
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

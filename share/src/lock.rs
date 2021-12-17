
use std::cell::{RefCell, Ref, RefMut, BorrowError, BorrowMutError};


#[derive(Debug)]
pub struct LockCell<T: ?Sized>(RefCell<T>);
unsafe impl<T> Sync for LockCell<T> where T: Sync {}
unsafe impl<T> Send for LockCell<T> where T: Send {}

impl<T> LockCell<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        LockCell(RefCell::new(value))
    }
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
    pub fn is_poisoned(&self) -> bool {
        false
    }
}

impl<T: ?Sized> LockCell<T> {

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

impl<T: Default> Default for LockCell<T> {
    fn default() -> Self {
        LockCell::new(Default::default())
    }
}
impl<T> const From<T> for LockCell<T> {
    fn from(t: T) -> Self {
        LockCell::new(t)
    }
}


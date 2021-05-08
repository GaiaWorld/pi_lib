pub use listener::FnListener;
use listener::{FnListeners, Listener as LibListener};
use share::Share;
use std::ops::Deref;

pub struct CreateEvent {
    pub id: usize,
}

pub struct DeleteEvent {
    pub id: usize,
}

pub struct ModifyEvent {
    pub id: usize,
    pub field: &'static str,
    pub index: usize, // 一般无意义。 只有在数组或向量的元素被修改时，才有意义
}

pub type CreateListeners = FnListeners<CreateEvent>;
pub type DeleteListeners = FnListeners<DeleteEvent>;
pub type ModifyListeners = FnListeners<ModifyEvent>;
pub type CreateFn = FnListener<CreateEvent>;
pub type DeleteFn = FnListener<DeleteEvent>;
pub type ModifyFn = FnListener<ModifyEvent>;

#[derive(Default, Clone)]
pub struct NotifyImpl(pub Share<NotifyImpl1>);

impl NotifyImpl {
    pub fn add_create(&self, listener: CreateFn) {
        unsafe { &mut *(self.0.as_ref() as *const NotifyImpl1 as *mut NotifyImpl1) }
            .create
            .push(listener)
    }
    pub fn add_delete(&self, listener: DeleteFn) {
        unsafe { &mut *(self.0.as_ref() as *const NotifyImpl1 as *mut NotifyImpl1) }
            .delete
            .push(listener)
    }
    pub fn add_modify(&self, listener: ModifyFn) {
        unsafe { &mut *(self.0.as_ref() as *const NotifyImpl1 as *mut NotifyImpl1) }
            .modify
            .push(listener)
    }

    pub fn remove_create(&self, listener: &CreateFn) {
        unsafe { &mut *(self.0.as_ref() as *const NotifyImpl1 as *mut NotifyImpl1) }
            .create
            .delete(listener);
    }
    pub fn remove_delete(&self, listener: &DeleteFn) {
        unsafe { &mut *(self.0.as_ref() as *const NotifyImpl1 as *mut NotifyImpl1) }
            .delete
            .delete(listener);
    }
    pub fn remove_modify(&self, listener: &ModifyFn) {
        unsafe { &mut *(self.0.as_ref() as *const NotifyImpl1 as *mut NotifyImpl1) }
            .modify
            .delete(listener);
    }
}

impl Deref for NotifyImpl {
    type Target = Share<NotifyImpl1>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Clone)]
pub struct NotifyImpl1 {
    pub create: CreateListeners,
    pub delete: DeleteListeners,
    pub modify: ModifyListeners,
}
impl NotifyImpl1 {
    pub fn mem_size(&self) -> usize {
        self.create.mem_size() + self.delete.mem_size() + self.modify.mem_size()
    }
    pub fn create_event(&self, id: usize) {
        let e = CreateEvent { id: id };
        self.create.listen(&e);
    }
    pub fn delete_event(&self, id: usize) {
        let e = DeleteEvent { id: id };
        self.delete.listen(&e);
    }
    pub fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        let e = ModifyEvent {
            id: id,
            field: field,
            index: index,
        };
        self.modify.listen(&e);
    }
}

pub trait Notify {
    fn add_create(&self, f: CreateFn);
    fn add_delete(&self, f: DeleteFn);
    fn add_modify(&self, f: ModifyFn);
    fn create_event(&self, id: usize);
    fn delete_event(&self, id: usize);
    fn modify_event(&self, id: usize, field: &'static str, index: usize);
    fn remove_create(&self, f: &CreateFn);
    fn remove_delete(&self, f: &DeleteFn);
    fn remove_modify(&self, f: &ModifyFn);
}

pub struct Write<'a, T> {
    pub id: usize,
    pub value: &'a mut T,
    pub notify: &'a NotifyImpl,
}

impl<'a, T> Write<'a, T> {
    pub fn new(id: usize, value: &'a mut T, notify: &'a NotifyImpl) -> Write<'a, T> {
        Write { id, value, notify }
    }
}

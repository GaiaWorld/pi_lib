use std::{
    sync::Arc,
    any::TypeId,
    ops::{ Deref, DerefMut},
};

use any::ArcAny;
use pointer::cell::{TrustCell};


use system::{SystemData, SystemMutData};
use monitor::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn, Write};
use {Fetch, Borrow, BorrowMut, TypeIds, World};
use Share;

pub trait SingleCase: Notify + ArcAny {
}
impl_downcast_arc!(SingleCase);

pub type CellSingleCase<T> = TrustCell<SingleCaseImpl<T>>;

impl<T: Share> SingleCase for CellSingleCase<T> {}

// TODO 以后用宏生成
impl<T: Share> Notify for CellSingleCase<T> {
    fn add_create(&self, listener: CreateFn) {
        self.borrow_mut().notify.create.push_back(listener)
    }
    fn add_delete(&self, listener: DeleteFn) {
        self.borrow_mut().notify.delete.push_back(listener)
    }
    fn add_modify(&self, listener: ModifyFn) {
        self.borrow_mut().notify.modify.push_back(listener)
    }
    fn create_event(&self, id: usize) {
        self.borrow().notify.create_event(id);
    }
    fn delete_event(&self, id: usize) {
        self.borrow().notify.delete_event(id);
    }
    fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        self.borrow().notify.modify_event(id, field, index);
    }
    fn remove_create(&self, listener: &CreateFn) {
        self.borrow_mut().notify.create.delete(listener);
    }
    fn remove_delete(&self, listener: &DeleteFn) {
        self.borrow_mut().notify.delete.delete(listener);
    }
    fn remove_modify(&self, listener: &ModifyFn) {
        self.borrow_mut().notify.modify.delete(listener);
    }
}

pub struct SingleCaseImpl<T: Share> {
    value: T,
    notify: NotifyImpl,
}

impl<T: Share> Deref for SingleCaseImpl<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target{
        &self.value
    }
}

impl<T: Share> DerefMut for SingleCaseImpl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.value
    }
}

impl<T: Share> SingleCaseImpl<T> {
    pub fn new(value: T) -> TrustCell<Self>{
        TrustCell::new(SingleCaseImpl{
            value,
            notify: NotifyImpl::default(),
        })
    }
    pub fn get_notify(&self) -> &NotifyImpl{
        &self.notify
    }
    pub fn get_write(&mut self) -> Write<T>{
        Write::new(0, &mut self.value, &self.notify)
    }
}

impl<'a, T: Share> SystemData<'a> for &'a SingleCaseImpl<T> {
    type FetchTarget = ShareSingleCase<T>;
}
impl<'a, T: Share> SystemMutData<'a> for &'a mut SingleCaseImpl<T> {
    type FetchTarget = ShareSingleCase<T>;
}

pub type ShareSingleCase<T> = Arc<CellSingleCase<T>>;

impl<T: Share> Fetch for ShareSingleCase<T> {
    fn fetch(world: &World) -> Self {
        world.fetch_single::<T>().unwrap()
    }
}

impl<T: Share> TypeIds for ShareSingleCase<T> {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![(TypeId::of::<()>(), TypeId::of::<T>())]
    }
}

impl<'a, T: Share> Borrow<'a> for ShareSingleCase<T> {
    type Target = &'a SingleCaseImpl<T>;
    fn borrow(&'a self) -> Self::Target {
        unsafe {&* (&* self.deref().borrow() as *const SingleCaseImpl<T>)}
    }
}

impl<'a, T: Share> BorrowMut<'a> for ShareSingleCase<T> {
    type Target = &'a mut SingleCaseImpl<T>;
    fn borrow_mut(&'a self) -> Self::Target {
        unsafe {&mut * (&mut *self.deref().borrow_mut() as *mut SingleCaseImpl<T>)}
    }
}
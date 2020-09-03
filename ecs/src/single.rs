use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use any::ArcAny;
// use pointer::cell::{StdCell};

use cell::StdCell;
use monitor::{CreateFn, DeleteFn, ModifyFn, Notify, NotifyImpl, Write};
use system::{SystemData, SystemMutData};
use {Fetch, Lend, LendMut, TypeIds, World};

pub trait SingleCase: Notify + ArcAny {}
impl_downcast_arc!(SingleCase);

pub type CellSingleCase<T> = StdCell<SingleCaseImpl<T>>;

impl<T: 'static> SingleCase for CellSingleCase<T> {}

// TODO 以后用宏生成
impl<T: 'static> Notify for CellSingleCase<T> {
    fn add_create(&self, listener: CreateFn) {
        self.borrow_mut().notify.add_create(listener);
    }
    fn add_delete(&self, listener: DeleteFn) {
        self.borrow_mut().notify.add_delete(listener)
    }
    fn add_modify(&self, listener: ModifyFn) {
        self.borrow_mut().notify.add_modify(listener)
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
        self.borrow_mut().notify.remove_create(listener);
    }
    fn remove_delete(&self, listener: &DeleteFn) {
        self.borrow_mut().notify.remove_delete(listener);
    }
    fn remove_modify(&self, listener: &ModifyFn) {
        self.borrow_mut().notify.remove_modify(listener);
    }
}

pub struct SingleCaseImpl<T: 'static> {
    value: T,
    notify: NotifyImpl,
}

impl<T: 'static> Deref for SingleCaseImpl<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: 'static> DerefMut for SingleCaseImpl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: 'static> SingleCaseImpl<T> {
    pub fn new(value: T) -> StdCell<Self> {
        StdCell::new(SingleCaseImpl {
            value,
            notify: NotifyImpl::default(),
        })
    }
    pub fn get_notify(&self) -> NotifyImpl {
        self.notify.clone()
    }

    pub fn get_notify_ref(&self) -> &NotifyImpl {
        &self.notify
    }

    pub fn get_write(&mut self) -> Write<T> {
        Write::new(0, &mut self.value, &self.notify)
    }
}

impl<'a, T: 'static> SystemData<'a> for &'a SingleCaseImpl<T> {
    type FetchTarget = ShareSingleCase<T>;
}
impl<'a, T: 'static> SystemMutData<'a> for &'a mut SingleCaseImpl<T> {
    type FetchTarget = ShareSingleCase<T>;
}

pub type ShareSingleCase<T> = Arc<CellSingleCase<T>>;

impl<T: 'static> Fetch for ShareSingleCase<T> {
    fn fetch(world: &World) -> Self {
        world.fetch_single::<T>().unwrap()
    }
}

impl<T: 'static> TypeIds for ShareSingleCase<T> {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![(TypeId::of::<()>(), TypeId::of::<T>())]
    }
}

impl<'a, T: 'static> Lend<'a> for ShareSingleCase<T> {
    type Target = &'a SingleCaseImpl<T>;
    type Target1 = usize;

    fn lend1(&'a self) -> Self::Target1 {
        &*self.deref().borrow() as *const SingleCaseImpl<T> as usize
    }

    fn lend2(&'a self, ptr: &usize) -> Self::Target {
        unsafe { &*(*ptr as *const SingleCaseImpl<T>) }
    }

    fn lend(&'a self) -> Self::Target {
        unsafe { &*(&*self.deref().borrow() as *const SingleCaseImpl<T>) }
    }
}

impl<'a, T: 'static> LendMut<'a> for ShareSingleCase<T> {
    type Target = &'a mut SingleCaseImpl<T>;
    type Target1 = usize;

    fn lend_mut1(&'a self) -> Self::Target1 {
        &mut *self.deref().borrow_mut() as *mut SingleCaseImpl<T> as usize
    }

    fn lend_mut2(&'a self, ptr: &usize) -> Self::Target {
        unsafe { &mut *(*ptr as *mut SingleCaseImpl<T>) }
    }

    fn lend_mut(&'a self) -> Self::Target {
        unsafe { &mut *(&mut *self.deref().borrow_mut() as *mut SingleCaseImpl<T>) }
    }
}

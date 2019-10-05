
use std::{
    sync::Arc,
    marker::PhantomData,
    any::TypeId,
    ops::Deref,
};

use any::ArcAny;
// use pointer::cell::{TrustCell};
use map::{Map};
use listener::Listener;

use system::{SystemData, SystemMutData};
use monitor::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn, Write, DeleteEvent};
use entity::CellEntity;
use {Fetch, Lend, LendMut, TypeIds, World};
use cell::StdCell;

pub trait Component: Sized + 'static {
    type Storage: Map<Key=usize, Val=Self> + Default;
}

pub trait MultiCase: Notify + ArcAny {
    fn delete(&self, id: usize);
}
impl_downcast_arc!(MultiCase);

pub type CellMultiCase<E, C> = StdCell<MultiCaseImpl<E, C>>;

impl<E: 'static, C: Component> MultiCase for CellMultiCase<E, C> {
    fn delete(&self, id: usize) {
        let notify = self.borrow_mut().notify.delete.clone();
        let e = DeleteEvent{
            id: id,
        };
        notify.listen(&e);
        self.borrow_mut().map.remove(&id);
    }
}
impl<E: 'static, C: Component> Notify for CellMultiCase<E, C> {
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

pub struct MultiCaseImpl<E, C: Component> {
    map: C::Storage,
    notify: NotifyImpl,
    entity: Arc<CellEntity<E>>,
    bit_index: usize,
    marker: PhantomData<E>,
}

impl<E: 'static, C: Component> MultiCaseImpl<E, C> {
    pub fn new(entity: Arc<CellEntity<E>>, bit_index: usize) -> StdCell<Self>{
        StdCell::new(MultiCaseImpl{
            map: C::Storage::default(),
            notify: NotifyImpl::default(),
            entity: entity,
            bit_index: bit_index,
            marker: PhantomData,
        })
    }
    pub fn mem_size(&self) -> usize {
        self.map.mem_size() + self.notify.mem_size()
    }
    pub fn get(&self, id: usize) -> Option<&C> {
        self.map.get(&id)
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut C> {
        self.map.get_mut(&id)
    }
    pub unsafe fn get_unchecked(&self, id: usize) -> &C {
        self.map.get_unchecked(&id)
    }
    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut C {
        self.map.get_unchecked_mut(&id)
    }
    pub fn get_write(&mut self, id: usize) -> Option<Write<C>> {
        match self.map.get_mut(&id) {
            Some(r) => Some(Write::new(id, r, &self.notify)),
            None => None,
        }
    }
    pub unsafe fn get_unchecked_write(&mut self, id: usize) -> Write<C> {
        Write::new(id, self.map.get_unchecked_mut(&id), &self.notify)
    }
    pub fn insert(&mut self, id: usize, c: C) -> Option<C> {
        let r = self.map.insert(id, c);
        match r {
            Some(_) => self.notify.modify_event(id, "", 0),
            _ => {
                self.entity.borrow_mut().mark(id, self.bit_index);
                self.notify.create_event(id);
            },
        }
        None
    }

    pub fn insert_no_notify(&mut self, id: usize, c: C) -> Option<C> {
        let r = self.map.insert(id, c);
        if let None = r {
            self.entity.borrow_mut().mark(id, self.bit_index)
        }
        r
    }
    
    pub fn delete(&mut self, id: usize) -> Option<C> {
        self.entity.borrow_mut().un_mark(id, self.bit_index);
        self.notify.delete_event(id);
        self.map.remove(&id)
    }

    pub fn get_notify(&self) -> NotifyImpl{
        self.notify.clone()
    }

    pub fn get_notify_ref(&self) -> &NotifyImpl{
        &self.notify
    }

    // fn remove(&mut self, id: usize) -> DeleteListeners {
    //     self.map.remove(&id);
    //     self.notify.delete.clone()
    // }
}

impl<'a, E: 'static, C: Component> SystemData<'a> for &'a MultiCaseImpl<E, C> {
    type FetchTarget = ShareMultiCase<E, C>;
}
impl<'a, E: 'static, C: Component> SystemMutData<'a> for &'a mut MultiCaseImpl<E, C> {
    type FetchTarget = ShareMultiCase<E, C>;
}

pub type ShareMultiCase<E, C> = Arc<CellMultiCase<E, C>>;

impl<E: 'static, C: Component> Fetch for ShareMultiCase<E, C> {
    fn fetch(world: &World) -> Self {
        world.fetch_multi::<E, C>().unwrap()
    }
}

impl<E: 'static, C: Component> TypeIds for ShareMultiCase<E, C> {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![(TypeId::of::<E>(), TypeId::of::<C>())]
    }
}

impl<'a, E: 'static, C: Component> Lend<'a> for ShareMultiCase<E, C> {
    type Target = &'a MultiCaseImpl<E, C>;
    type Target1 = usize;

    fn lend1(&'a self) -> Self::Target1 {
        &*self.deref().borrow() as *const MultiCaseImpl<E, C> as usize
    }

    fn lend2(&'a self, ptr: &Self::Target1) -> Self::Target {
        unsafe { &* (*ptr as  *const MultiCaseImpl<E, C>) }
    }

    fn lend(&'a self) -> Self::Target {
        unsafe {&* (&* self.deref().borrow() as *const MultiCaseImpl<E, C>)}
    }
}

impl<'a, E: 'static, C: Component> LendMut<'a> for ShareMultiCase<E, C> {
    type Target = &'a mut MultiCaseImpl<E, C>;
    type Target1 = usize;

    fn lend_mut1(&'a self) -> Self::Target1 {
        &mut *self.deref().borrow_mut() as *mut MultiCaseImpl<E, C> as usize
    }

    fn lend_mut2(&'a self, ptr: &Self::Target1) -> Self::Target {
        unsafe { &mut * (*ptr as  *mut MultiCaseImpl<E, C>) }
    }

    fn lend_mut(&'a self) -> Self::Target {
        unsafe {&mut * (&mut *self.deref().borrow_mut() as *mut MultiCaseImpl<E, C>)}
    }
}

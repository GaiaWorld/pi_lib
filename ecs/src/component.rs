
use std::{
    sync::Arc,
    marker::PhantomData,
    any::TypeId,
    ops::Deref,
};

use any::ArcAny;
use pointer::cell::{TrustCell};
use map::{Map};


use system::{SystemData, SystemMutData};
use monitor::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn, Write};
use entity::CellEntity;
use {Fetch, Borrow, BorrowMut, TypeIds, World};
use Share;

pub trait Component: Sized + Share {
    type Storage: Map<Key=usize, Val=Self> + Default + Share;
}

pub trait MultiCase: Notify + ArcAny {
    fn delete(&self, id: usize);
}
impl_downcast_arc!(MultiCase);

pub type CellMultiCase<E, C> = TrustCell<MultiCaseImpl<E, C>>;

impl<E: Share, C: Component> MultiCase for CellMultiCase<E, C> {
    fn delete(&self, id: usize) {
        self.borrow_mut().remove(id)
    }
}
// TODO 以后用宏生成
impl<E: Share, C: Component> Notify for CellMultiCase<E, C> {
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

pub struct MultiCaseImpl<E: Share, C: Component> {
    map: C::Storage,
    notify: NotifyImpl,
    entity: Arc<CellEntity<E>>,
    bit_index: usize,
    marker: PhantomData<E>,
}

impl<E: Share, C: Component> MultiCaseImpl<E, C> {
    pub fn new(entity: Arc<CellEntity<E>>, bit_index: usize) -> TrustCell<Self>{
        TrustCell::new(MultiCaseImpl{
            map: C::Storage::default(),
            notify: NotifyImpl::default(),
            entity: entity,
            bit_index: bit_index,
            marker: PhantomData,
        })
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
        self.entity.borrow_mut().mark(id, self.bit_index);
        match r {
            Some(_) => self.notify.modify_event(id, "", 0),
            _ => self.notify.create_event(id),
        }
        r
    }
    pub fn delete(&mut self, id: usize) {
        self.entity.borrow_mut().un_mark(id, self.bit_index);
        self.notify.delete_event(id);
        self.map.remove(&id);
    }
    fn remove(&mut self, id: usize) {
        self.notify.delete_event(id);
        self.map.remove(&id);
    }
}

impl<'a, E: Share, C: Component> SystemData<'a> for &'a MultiCaseImpl<E, C> {
    type FetchTarget = ShareMultiCase<E, C>;
}
impl<'a, E: Share, C: Component> SystemMutData<'a> for &'a mut MultiCaseImpl<E, C> {
    type FetchTarget = ShareMultiCase<E, C>;
}

pub type ShareMultiCase<E, C> = Arc<CellMultiCase<E, C>>;

impl<E: Share, C: Component> Fetch for ShareMultiCase<E, C> {
    fn fetch(world: &World) -> Self {
        world.fetch_multi::<E, C>().unwrap()
    }
}

impl<E: Share, C: Component> TypeIds for ShareMultiCase<E, C> {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![(TypeId::of::<E>(), TypeId::of::<C>())]
    }
}

impl<'a, E: Share, C: Component> Borrow<'a> for ShareMultiCase<E, C> {
    type Target = &'a MultiCaseImpl<E, C>;
    fn borrow(&'a self) -> Self::Target {
        unsafe {&* (&* self.deref().borrow() as *const MultiCaseImpl<E, C>)}
    }
}

impl<'a, E: Share, C: Component> BorrowMut<'a> for ShareMultiCase<E, C> {
    type Target = &'a mut MultiCaseImpl<E, C>;
    fn borrow_mut(&'a self) -> Self::Target {
        unsafe {&mut * (&mut *self.deref().borrow_mut() as *mut MultiCaseImpl<E, C>)}
    }
}

use std::{
    sync::Arc,
    marker::PhantomData,
};

pub use any::ArcAny;

use pointer::cell::{TrustCell};
use map::{Map, vecmap::VecMap};


use system::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};
use entity::CellEntity;
use Share;

pub trait Component: Sized + Share {
    type Strorage: Map<Key=usize, Val=Self>;
}


pub trait SingleCase: Notify + ArcAny {
}
impl_downcast_arc!(SingleCase);

pub trait MultiCase: Notify + ArcAny {
    fn delete(&self, id: usize);
}
impl_downcast_arc!(MultiCase);

pub type CellMultiCase<E: Share, C: Component> = TrustCell<MultiCaseImpl<E, C>>;
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
impl<E: Share, C: Component> MultiCase for CellMultiCase<E, C> {
    fn delete(&self, id: usize) {
        self.borrow_mut().delete(id)
    }
}

#[derive(Default)]
pub struct MultiCaseImpl<E: Share, C: Component> {
    map: VecMap<C>,
    notify: NotifyImpl,
    entity: Arc<CellEntity>,
    bit_index: usize,
    marker: PhantomData<E>,
}

impl<E: Share, C: Component> MultiCaseImpl<E, C> {
    pub fn new(entity: Arc<CellEntity>, bit_index: usize) -> TrustCell<Self>{
        TrustCell::new(MultiCaseImpl{
            map: VecMap::default(),
            notify: NotifyImpl::default(),
            entity: entity,
            bit_index: bit_index,
            marker: PhantomData,
        })
    }
    pub fn delete(&mut self, id: usize) {
        self.notify.delete_event(id);
        self.map.remove(id);
    }
}

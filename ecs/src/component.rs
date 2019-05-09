
use std::{
    sync::Arc,
    marker::PhantomData,
    any::TypeId,
};

pub use any::ArcAny;
use pointer::cell::{TrustCell};
use map::{Map};


use system::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn, SystemData, SystemMutData};
use entity::Entity;
use world::{Fetch, World, Borrow, BorrowMut, TypeIds};
use Share;

pub trait Component: Sized + Share {
    type Strorage: Map<Key=usize, Val=Self> + Default + Share;
}


pub trait SingleCase: Notify + ArcAny {
}
impl_downcast_arc!(SingleCase);

pub trait MultiCase: Notify + ArcAny {
    fn delete(&self, id: usize);
}
impl_downcast_arc!(MultiCase);

pub type CellMultiCase<E, C> = TrustCell<MultiCaseImpl<E, C>>;
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
    map: C::Strorage,
    notify: NotifyImpl,
    entity: Arc<CellEntity<E>>,
    bit_index: usize,
    marker: PhantomData<E>,
}

impl<E: Share, C: Component> MultiCaseImpl<E, C> {
    pub fn new(entity: Arc<CellEntity<E>>, bit_index: usize) -> TrustCell<Self>{
        TrustCell::new(MultiCaseImpl{
            map: C::Strorage::default(),
            notify: NotifyImpl::default(),
            entity: entity,
            bit_index: bit_index,
            marker: PhantomData,
        })
    }
    pub fn get(&mut self, id: usize) -> Option<&C> {
        self.map.get(&id)
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut C> {
        self.map.get_mut(&id)
    }
    pub unsafe fn get_unchecked(&mut self, id: usize) -> &C {
        self.map.get_unchecked(&id)
    }
    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut C {
        self.map.get_unchecked_mut(&id)
    }
    pub fn insert(&mut self, id: usize, c: C) -> Option<C> {
        let r = self.map.insert(id, c);
        match r {
            Some(_) => self.notify.modify_event(id, "", 0),
            _ => self.notify.create_event(id),
        }
        r
    }
    pub fn delete(&mut self, id: usize) {
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

pub struct ShareMultiCase<E: Share, C: Component>(Arc<CellMultiCase<E, C>>);

impl<E: Share, C: Component> Fetch for ShareMultiCase<E, C> {
    fn fetch(world: &World) -> Self {
        match world.fetch_multi::<E, C>().unwrap().downcast() {
            Ok(r) => ShareMultiCase(r),
            Err(_) => panic!("downcast err"),
        }
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
        unsafe {&* (&*self.0.borrow() as *const MultiCaseImpl<E, C>)}
    }
}

impl<'a, E: Share, C: Component> BorrowMut<'a> for ShareMultiCase<E, C> {
    type Target = &'a mut MultiCaseImpl<E, C>;
    fn borrow_mut(&'a self) -> Self::Target {
        unsafe {&mut * (&mut *self.0.borrow_mut() as *mut MultiCaseImpl<E, C>)}
    }
}



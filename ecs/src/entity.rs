use std::{
    sync::Arc,
    mem::size_of,
    any::TypeId,
    marker::PhantomData,
    ops::Deref,
};

pub use any::ArcAny;
use pointer::cell::TrustCell;
use slab::Slab;


use {Fetch, Lend, LendMut, TypeIds, World};
use system::{SystemData, SystemMutData};
use monitor::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};
use component::MultiCase;
use Share;


pub trait Entity: Notify + ArcAny {
    fn get_mask(&self) -> usize;
    fn register_component(&mut self, component: Arc<MultiCase>);
    fn create(&mut self) -> usize;
    fn delete(&mut self, id: usize);
}
impl_downcast_arc!(Entity);

pub type CellEntity<T> = TrustCell<EntityImpl<T>>;
impl<T: Share> Notify for CellEntity<T> {
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
impl<T: Share> Entity for CellEntity<T> {
    fn get_mask(&self) -> usize {
        self.borrow().get_mask()
    }
    fn register_component(&mut self, component: Arc<MultiCase>) {
        self.borrow_mut().register_component(component)
    }
    fn create(&mut self) -> usize {
        self.borrow_mut().create()
    }
    fn delete(&mut self, id: usize) {
        self.borrow_mut().delete(id)
    }

}


pub struct EntityImpl<T>{
    slab: Slab<u64>, // 值usize 记录每个id所关联的component的掩码位
    components: Vec<Arc<MultiCase>>, // 组件
    notify: NotifyImpl,
    marker: PhantomData<T>,
}
impl<T> EntityImpl<T> {
    pub fn new() -> EntityImpl<T> {
        EntityImpl{
            slab: Slab::default(),
            components: Vec::new(),
            notify: NotifyImpl::default(),
            marker: PhantomData,
        }
    }
    pub fn get_mask(&self) -> usize {
        self.components.len()
    }
    pub fn register_component(&mut self, component: Arc<MultiCase>) {
        if self.components.len() >= size_of::<u64>()<<3 {
            panic!("components overflow")
        }
        self.components.push(component);
    }
    pub fn create(&mut self) -> usize {
        let id = self.slab.insert(0);
        self.notify.create_event(id);
        id
    }
    pub fn mark(&mut self, id: usize, bit_index: usize) {
        let mask = self.slab.get_mut(id).unwrap();
        *mask |= 1<<bit_index;
    }
    pub fn un_mark(&mut self, id: usize, bit_index: usize) {
        match self.slab.get_mut(id) {
            Some(mask) => *mask &= !(1<<bit_index),
            _ => ()
        }
    }
    pub fn delete(&mut self, id: usize) {
        let mask = self.slab.remove(id);
        self.notify.modify_event(id, "", 0);
        if mask == 0 {
            return
        }
        // 依次删除对应的组件
        for i in mask.trailing_zeros() as usize..(size_of::<usize>() <<3 )-(mask.leading_zeros() as usize) {
            if mask & (1<<i) != 0 {
                self.components[i].delete(id)
            }
        }
        self.notify.delete_event(id);
    }

}

impl<'a, T: Share> SystemData<'a> for &'a EntityImpl<T> {
    type FetchTarget = ShareEntity<T>;
}
impl<'a, T: Share> SystemMutData<'a> for &'a mut EntityImpl<T> {
    type FetchTarget = ShareEntity<T>;
}

pub type ShareEntity<T> = Arc<CellEntity<T>>;

impl<T: Share> Fetch for ShareEntity<T> {
    fn fetch(world: &World) -> Self {
        world.fetch_entity::<T>().unwrap()
    }
}

impl<T: Share> TypeIds for ShareEntity<T> {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![(TypeId::of::<T>(), TypeId::of::<()>())]
    }
}

impl<'a, T: Share> Lend<'a> for ShareEntity<T> {
    type Target = &'a EntityImpl<T>;
    fn lend(&'a self) -> Self::Target {
        unsafe {&* (&*self.deref().borrow() as *const EntityImpl<T>)}
    }
}

impl<'a, T: Share> LendMut<'a> for ShareEntity<T> {
    type Target = &'a mut EntityImpl<T>;
    fn lend_mut(&'a self) -> Self::Target {
        unsafe {&mut * (&mut *self.deref().borrow_mut() as *mut EntityImpl<T>)}
    }
}

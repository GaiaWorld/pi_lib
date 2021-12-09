use std::{any::TypeId, marker::PhantomData, mem::size_of, ops::Deref, sync::Arc};

pub use any::ArcAny;
// use pointer::cell::StdCell;
use slab::{Slab, SlabIter};

use crate::cell::StdCell;
use crate::component::MultiCase;
use crate::monitor::{CreateFn, DeleteFn, ModifyFn, Notify, NotifyImpl};
use crate::system::{SystemData, SystemMutData};
use crate::{Fetch, Lend, LendMut, TypeIds, World};

pub trait Entity: Notify + ArcAny {
    fn get_mask(&self) -> usize;
    fn register_component(&mut self, component: Arc<dyn MultiCase>);
    fn create(&mut self) -> usize;
    fn delete(&mut self, id: usize);
    fn clear(&self);
}
impl_downcast_arc!(Entity);

pub type CellEntity<T> = StdCell<EntityImpl<T>>;

impl<T: 'static> Notify for CellEntity<T> {
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
impl<T: 'static> Entity for CellEntity<T> {
    fn get_mask(&self) -> usize {
        self.borrow().get_mask()
    }
    fn register_component(&mut self, component: Arc<dyn MultiCase>) {
        self.borrow_mut().register_component(component)
    }
    fn create(&mut self) -> usize {
        self.borrow_mut().create()
    }
    fn delete(&mut self, id: usize) {
        self.borrow_mut().delete(id)
    }
    fn clear(&self) {
        let r = unsafe{&mut *(self as *const Self as usize as *mut Self)};
        r.borrow_mut().components.clear();
        r.borrow_mut().slab.clear();
    }
}

pub struct EntityImpl<T> {
    slab: Slab<u64>,                     // 值usize 记录每个id所关联的component的掩码位
    components: Vec<Arc<dyn MultiCase>>, // 组件
    notify: NotifyImpl,
    marker: PhantomData<T>,
}
impl<T> EntityImpl<T> {
    pub fn new() -> EntityImpl<T> {
        EntityImpl {
            slab: Slab::default(),
            components: Vec::new(),
            notify: NotifyImpl::default(),
            marker: PhantomData,
        }
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
		EntityImpl {
            slab: Slab::with_capacity(capacity),
            components: Vec::new(),
            notify: NotifyImpl::default(),
            marker: PhantomData,
        }
	}

    pub fn mem_size(&self) -> usize {
        let mut r = 0;
        r += self.slab.mem_size();
        r += self.components.capacity() * std::mem::size_of::<Arc<dyn MultiCase>>();
        r += self.notify.mem_size();
        r
    }

    pub fn is_exist(&self, id: usize) -> bool {
        match self.slab.get(id) {
            Some(_) => true,
            None => false,
        }
    }
    pub fn get_mask(&self) -> usize {
        self.components.len()
    }
    pub fn register_component(&mut self, component: Arc<dyn MultiCase>) {
        if self.components.len() >= size_of::<u64>() << 3 {
            log::error!("components overflow");
            panic!()
        }
        self.components.push(component);
    }
    pub fn create(&mut self) -> usize {
        let id = self.slab.insert(0);
        self.notify.create_event(id);
        id
	}
	
	// 创建但不通知
	pub fn create_but_no_notify(&mut self) -> usize {
        let id = self.slab.insert(0);
        id
	}
	
    pub fn mark(&mut self, _id: usize, _bit_index: usize) {
        // let mask = self.slab.get_mut(id).unwrap();
        // *mask |= 1 << bit_index;
    }
    pub fn un_mark(&mut self, _id: usize, _bit_index: usize) {
        // match self.slab.get_mut(id) {
        //     Some(mask) => *mask &= !(1 << bit_index),
        //     _ => (),
        // }
    }
    pub fn delete(&mut self, id: usize) {
        // let _mask = self.slab.remove(id);
        self.slab.remove(id);
        self.notify.modify_event(id, "", 0);
        // if mask == 0 {
        //     return;
        // }
        // // 依次删除对应的组件
        // for i in mask.trailing_zeros() as usize
        //     ..(size_of::<u64>() << 3) - (mask.leading_zeros() as usize)
        // {
        //     if mask & (1 << i) != 0 {
        //         self.components[i].delete(id)
        //     }
		// }
		for ci in self.components.iter() {
			ci.delete(id);
		}

        self.notify.delete_event(id);
    }

    pub fn iter(&self) -> EntityIter {
        EntityIter(self.slab.iter())
	}
	
	pub fn len(&self) -> usize {
        self.slab.len()
    }
}

pub struct EntityIter<'a>(SlabIter<'a, u64>);

impl<'a> Iterator for EntityIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self.0.next() {
            Some(r) => Some(r.0),
            None => None,
        }
    }
}

impl<'a, T: 'static> SystemData<'a> for &'a EntityImpl<T> {
    type FetchTarget = ShareEntity<T>;
}
impl<'a, T: 'static> SystemMutData<'a> for &'a mut EntityImpl<T> {
    type FetchTarget = ShareEntity<T>;
}

pub type ShareEntity<T> = Arc<CellEntity<T>>;

impl<T: 'static> Fetch for ShareEntity<T> {
    fn fetch(world: &World) -> Self {
		match world.fetch_entity::<T>() {
			Some(r) => r,
			None => {
                log::error!("fetch_multi fail:{:?}",  std::any::type_name::<T>());
                std::panic!();
            },
		}
    }
}

impl<T: 'static> TypeIds for ShareEntity<T> {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![(TypeId::of::<T>(), TypeId::of::<()>())]
    }
}

impl<'a, T: 'static> Lend<'a> for ShareEntity<T> {
    type Target = &'a EntityImpl<T>;
    type Target1 = usize;

    fn lend1(&'a self) -> Self::Target1 {
        &*self.deref().borrow() as *const EntityImpl<T> as usize
    }

    fn lend2(&'a self, ptr: &usize) -> Self::Target {
        unsafe { &*(*ptr as *const EntityImpl<T>) }
    }

    fn lend(&'a self) -> Self::Target {
        unsafe { &*(&*self.deref().borrow() as *const EntityImpl<T>) }
    }
}

impl<'a, T: 'static> LendMut<'a> for ShareEntity<T> {
    type Target = &'a mut EntityImpl<T>;
    type Target1 = usize;

    fn lend_mut1(&'a self) -> Self::Target1 {
        &mut *self.deref().borrow_mut() as *mut EntityImpl<T> as usize
    }

    fn lend_mut2(&'a self, ptr: &usize) -> Self::Target {
        unsafe { &mut *(*ptr as *mut EntityImpl<T>) }
    }

    fn lend_mut(&'a self) -> Self::Target {
        unsafe { &mut *(&mut *self.deref().borrow_mut() as *mut EntityImpl<T>) }
    }
}

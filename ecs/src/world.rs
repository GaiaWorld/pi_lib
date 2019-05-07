
use std::{
    sync::Arc,
    any::TypeId,
    intrinsics::type_name,
};


use im::hashmap::HashMap;

use atom::Atom;
use listener::{FnListeners, FnListener};
use pointer::cell::{Ref, RefMut, TrustCell};

use system::{System, RunnerFn};
use entity::{Entity, CellEntity};
use component::{SingleCase, MultiCase, CellMultiCase, MultiCaseImpl};
use dispatch::Dispatcher;
// TODO component

#[derive(Default)]
pub struct World {
    entity: HashMap<TypeId, Arc<CellEntity>>,
    single: HashMap<TypeId, Arc<SingleCase>>,
    multi: HashMap<(TypeId, TypeId), Arc<MultiCase>>,
    system: HashMap<Atom, Arc<System>>,
    runner: HashMap<Atom, Arc<Dispatcher>>,
}

impl World {
    pub fn register_entity<E: 'static>(&mut self) {
        let id = TypeId::of::<E>();
        match self.entity.insert(id, Arc::new(TrustCell::new(Entity::default()))) {
            Some(_) => panic!("duplicate registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, id),
            _ => ()
        }
    }
    /// 注册单例组件
    pub fn register_single<C: SingleCase + 'static>(&mut self, c: C) {
        let id = TypeId::of::<C>();
        match self.single.insert(id, Arc::new(c)) {
            Some(_) => panic!("duplicate registration, component: {:?}, id: {:?}", unsafe{type_name::<C>()}, id),
            _ => ()
        }
    }
    /// 注册多例组件，必须声明是那种entity上的组件
    pub fn register_multi<E: 'static, C: 'static>(&mut self) {
        let eid = TypeId::of::<E>();
        let cid = TypeId::of::<C>();
        match self.entity.get(&eid) {
            Some(v) => {
                let mut entity = v.borrow_mut();
                let m: Arc<CellMultiCase<E, C>> = Arc::new(MultiCaseImpl::new(v.clone(), entity.get_mask()));
                entity.register_component(m.clone());
                match self.multi.insert((eid, cid), m) {
                    Some(_) => panic!("duplicate registration, entity: {:?}, component: {:?}", unsafe{type_name::<E>()}, unsafe{type_name::<C>()}),
                    _ => ()
                }
            }
            _ => panic!("need registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, eid),
        }
    }
    pub fn register_system<T>(&mut self, name: Atom, sys: T) {
        // 如果是Runner则调用setup方法， 获取所有实现了监听器的类型，动态注册到对应的组件监听器上Atom
    }
    pub fn get_system(&self, name: &Atom) -> Option<&Arc<System>> {
        self.system.get(name)
    }
    pub fn unregister_system(&mut self, name: &Atom) {
        // 要求该system不能在dispatcher中， 取消所有的监听器
        // 如果是Runner则调用dispose方法
    }
    pub fn create_entity<E: 'static>(&self) -> usize {
        let id = TypeId::of::<E>();
        match self.entity.get(&id) {
            Some(v) => {
                v.borrow_mut().create()
            }
            _ => panic!("not registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, id),
        }
    }
    pub fn free_entity<E: 'static>(&self, id: usize) {
        let eid = TypeId::of::<E>();
        match self.entity.get(&eid) {
            Some(v) => {
                v.borrow_mut().delete(id);
            }
            _ => panic!("not registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, eid),
        }
    }
    pub fn add_dispatcher<D: Dispatcher + 'static>(&mut self, name: Atom, dispatcher: D) {
        self.runner.insert(name, Arc::new(dispatcher));
    }
    pub fn get_dispatcher(&self, name: &Atom) -> Option<&Arc<Dispatcher>> {
        self.runner.get(name)
    }
    pub fn remove_dispatcher(&mut self, name: &Atom) -> Option<Arc<Dispatcher>> {
        self.runner.remove(name)
    }
    pub fn fetch_entry<T: 'static>(&self) -> Option<Arc<CellEntity>> {
        let id = TypeId::of::<T>();
        match self.entity.get(&id) {
            Some(v) => Some(v.clone()),
            _ => None
        }
    }
    pub fn fetch_single<T: 'static>(&self) -> Option<Arc<SingleCase>> {
        let id = TypeId::of::<T>();
        match self.single.get(&id) {
            Some(v) => Some(v.clone()),
            _ => None
        }
    }
    pub fn fetch_multi<E: 'static, C: 'static>(&self) -> Option<Arc<MultiCase>> {
        let eid = TypeId::of::<E>();
        let cid = TypeId::of::<C>();
        match self.multi.get(&(eid, cid)) {
            Some(v) => Some(v.clone()),
            _ => None
        }
    }

    pub fn run(&self, name: &Atom) {
        match self.runner.get(name) {
            Some(v) => v.run(),
            _ => ()
        }
    }
}

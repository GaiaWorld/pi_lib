use std::{any::TypeId, intrinsics::type_name, mem, sync::Arc};

use hash::XHashMap;
// use im::hashmap::HashMap;

use atom::Atom;
use share::Share;
// use pointer::cell::{TrustCell};

use crate::cell::StdCell;
use crate::component::{CellMultiCase, Component, MultiCase, MultiCaseImpl};
use crate::dispatch::Dispatcher;
use crate::entity::{CellEntity, Entity, EntityImpl};
use crate::single::{CellSingleCase, SingleCase, SingleCaseImpl};
use crate::system::System;
use crate::LendMut;
use crate::RunTime;

#[derive(Default, Clone)]
pub struct World {
    entity: XHashMap<TypeId, Arc<dyn Entity>>,
    single: XHashMap<TypeId, Arc<dyn SingleCase>>,
    multi: XHashMap<(TypeId, TypeId), Arc<dyn MultiCase>>,
    system: XHashMap<Atom, Arc<dyn System>>,
    runner: XHashMap<Atom, Arc<dyn Dispatcher>>,
    // #[cfg(feature = "runtime")]
	pub runtime: Share<Vec<RunTime>>,
	pub capacity: usize,
}

impl Drop for World {
    fn drop(&mut self) {
        for (_, v) in self.entity.iter_mut() {
            v.clear();
        }
        // std::mem::replace(&mut self.entity, XHashMap::default());
    }
}

impl World {
    pub fn register_entity<E: 'static>(&mut self) {
        let id = TypeId::of::<E>();
        match self
            .entity
            .insert(id, Arc::new(StdCell::new(EntityImpl::<E>::with_capacity(self.capacity))))
        {
            Some(_) => panic!(
                "duplicate registration, entity: {:?}, id: {:?}",
                type_name::<E>(),
                id
            ),
            _ => (),
        }
    }
    /// 注册单例组件
    pub fn register_single<T: 'static>(&mut self, t: T) {
        let id = TypeId::of::<T>();
        match self.single.insert(id, Arc::new(SingleCaseImpl::new(t))) {
            Some(_) => panic!(
                "duplicate registration, component: {:?}, id: {:?}",
                type_name::<T>(),
                id
            ),
            _ => (),
        }
    }
    /// 注册多例组件，必须声明是那种entity上的组件
    pub fn register_multi<E: 'static, C: Component>(&mut self) {
        let eid = TypeId::of::<E>();
        let cid = TypeId::of::<C>();
        match self.entity.get(&eid) {
            Some(v) => {
                match v.clone().downcast() {
                    Ok(r) => {
                        let r: Arc<CellEntity<E>> = r;
                        let rc = r.clone();
                        let entity = LendMut::lend_mut(&r);
                        let m: Arc<CellMultiCase<E, C>> =
                            Arc::new(MultiCaseImpl::new(rc, entity.get_mask(), self.capacity));
                        entity.register_component(m.clone());
                        match self.multi.insert((eid, cid), m) {
                            Some(_) => panic!(
                                "duplicate registration, entity: {:?}, component: {:?}",
                                type_name::<E>(),
                                type_name::<C>()
                            ),
                            _ => (),
                        }
                    }
                    Err(_) => panic!("downcast err"),
                };
            }
            _ => panic!(
                "need registration, entity: {:?}, id: {:?}",
                type_name::<E>(),
                eid
            ),
        }
    }
    pub fn register_system<T: System>(&mut self, name: Atom, sys: T) {
        // 调用setup方法， 将所有实现了监听器的类型，动态注册到对应的组件监听器上
        let t = Arc::new(sys);
        let tc = t.clone();
        let ptr = Arc::into_raw(t) as usize as *mut T;
        System::setup(unsafe { &mut *ptr }, tc, self, &name);
        self.system.insert(name, unsafe { Arc::from_raw(ptr) });
    }
    pub fn get_system(&self, name: &Atom) -> Option<&Arc<dyn System>> {
        self.system.get(name)
    }
    pub fn unregister_system(&mut self, name: &Atom) {
        // 如果该system在dispatcher中，需要自己去释放
        // 用dispose方法， 取消所有的监听器
        match self.system.remove(name) {
            Some(sys) => sys.dispose(self),
            _ => (),
        }
    }
    pub fn create_entity<E: 'static>(&self) -> usize {
        let id = TypeId::of::<E>();
        match self.entity.get(&id) {
            Some(v) => match v.clone().downcast() {
                Ok(r) => {
                    let rc: Arc<CellEntity<E>> = r;
                    LendMut::lend_mut(&rc).create()
                }
                Err(_) => panic!("downcast err"),
            },
            _ => panic!(
                "not registration, entity: {:?}, id: {:?}",
                type_name::<E>(),
                id
            ),
        }
    }
    pub fn free_entity<E: 'static>(&self, id: usize) {
        let eid = TypeId::of::<E>();
        match self.entity.get(&eid) {
            Some(v) => match v.clone().downcast() {
                Ok(r) => {
                    let r: Arc<CellEntity<E>> = r;
                    LendMut::lend_mut(&r).delete(id);
                }
                Err(_) => panic!("downcast err"),
            },
            _ => panic!(
                "not registration, entity: {:?}, id: {:?}",
                type_name::<E>(),
                eid
            ),
        }
    }
    pub fn add_dispatcher<D: Dispatcher + 'static>(&mut self, name: Atom, dispatcher: D) {
        self.runner.insert(name, Arc::new(dispatcher));
    }
    pub fn get_dispatcher(&self, name: &Atom) -> Option<&Arc<dyn Dispatcher>> {
        self.runner.get(name)
    }
    pub fn get_dispatcher_mut(&mut self, name: &Atom) -> Option<&mut Arc<dyn Dispatcher>> {
        self.runner.get_mut(name)
    }
    pub fn remove_dispatcher(&mut self, name: &Atom) -> Option<Arc<dyn Dispatcher>> {
        self.runner.remove(name)
    }
    pub fn fetch_entity<T: 'static>(&self) -> Option<Arc<CellEntity<T>>> {
        let id = TypeId::of::<T>();
        let r = match self.entity.get(&id) {
            Some(v) => v.clone(),
            _ => return None,
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }
    pub fn fetch_single<T: 'static>(&self) -> Option<Arc<CellSingleCase<T>>> {
        let id = TypeId::of::<T>();
        let r = match self.single.get(&id) {
            Some(v) => v.clone(),
            _ => return None,
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }
    pub fn fetch_multi<E: 'static, C: Component>(&self) -> Option<Arc<CellMultiCase<E, C>>> {
        let eid = TypeId::of::<E>();
        let cid = TypeId::of::<C>();
        let r = match self.multi.get(&(eid, cid)) {
            Some(v) => v.clone(),
            _ => return None,
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }

    pub fn fetch_sys<S: System>(&self, name: &Atom) -> Option<Arc<S>> {
        let r = match self.system.get(&name) {
            Some(v) => v.clone(),
            _ => return None,
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }

    pub fn run(&self, name: &Atom) {
        // #[cfg(feature = "runtime")]
        for r in
            unsafe { &mut *(self.runtime.as_ref() as *const Vec<RunTime> as *mut Vec<RunTime>) }
                .iter_mut()
        {
            r.cost_time = std::time::Duration::from_millis(0);
        }
        match self.runner.get(name) {
            Some(v) => v.run(),
            _ => (),
        }
    }
}


use std::{
    sync::Arc,
    any::TypeId,
    intrinsics::type_name,
};

use fnv::FnvHashMap;
// use im::hashmap::HashMap;

use atom::Atom;
// use pointer::cell::{TrustCell};

use system::{System};
use entity::{Entity, EntityImpl, CellEntity};
use component::{MultiCase, CellMultiCase, MultiCaseImpl, Component};
use single::{SingleCase, CellSingleCase, SingleCaseImpl};
use dispatch::Dispatcher;
use { Share, LendMut};
use cell::StdCell;

#[derive(Default, Clone)]
pub struct World {
    entity: FnvHashMap<TypeId, Arc<dyn Entity>>,
    single: FnvHashMap<TypeId, Arc<dyn SingleCase>>,
    multi: FnvHashMap<(TypeId, TypeId), Arc<dyn MultiCase>>,
    system: FnvHashMap<Atom, Arc<dyn System>>,
    runner: FnvHashMap<Atom, Arc<dyn Dispatcher>>,
}

impl World {
    pub fn register_entity<E: Share>(&mut self) {
        let id = TypeId::of::<E>();
        match self.entity.insert(id, Arc::new(StdCell::new(EntityImpl::<E>::new()))) {
            Some(_) => panic!("duplicate registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, id),
            _ => ()
        }
    }
    /// 注册单例组件
    pub fn register_single<T: Share>(&mut self, t: T) {
        let id = TypeId::of::<T>();
        match self.single.insert(id, Arc::new(SingleCaseImpl::new(t))) {
            Some(_) => panic!("duplicate registration, component: {:?}, id: {:?}", unsafe{type_name::<T>()}, id),
            _ => ()
        }
    }
    /// 注册多例组件，必须声明是那种entity上的组件
    pub fn register_multi<E: Share, C: Component>(&mut self) {
        let eid = TypeId::of::<E>();
        let cid = TypeId::of::<C>();
        match self.entity.get(&eid) {
            Some(v) => {
                match v.clone().downcast(){
                    Ok(r) => {
                        let r: Arc<CellEntity<E>> = r;
                        let rc = r.clone();
                        let entity = LendMut::lend_mut(&r);
                        let m: Arc<CellMultiCase<E, C>> = Arc::new(MultiCaseImpl::new(rc, entity.get_mask()));
                        entity.register_component(m.clone());
                        match self.multi.insert((eid, cid), m) {
                            Some(_) => panic!("duplicate registration, entity: {:?}, component: {:?}", unsafe{type_name::<E>()}, unsafe{type_name::<C>()}),
                            _ => ()
                        }
                    },
                    Err(_) => panic!("downcast err")
                };
            },
            _ => panic!("need registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, eid),
        }
    }
    pub fn register_system<T:System>(&mut self, name: Atom, sys: T) {
        // 调用setup方法， 将所有实现了监听器的类型，动态注册到对应的组件监听器上
        let t = Arc::new(sys);
        let tc = t.clone();
        let ptr = Arc::into_raw(t) as usize as *mut T;
        System::setup(unsafe{&mut *ptr}, tc, self);
        self.system.insert(name, unsafe{ Arc::from_raw(ptr)});
    }
    pub fn get_system(&self, name: &Atom) -> Option<&Arc<dyn System>> {
        self.system.get(name)
    }
    pub fn unregister_system(&mut self, name: &Atom) {
        // 如果该system在dispatcher中，需要自己去释放
        // 用dispose方法， 取消所有的监听器
        match self.system.remove(name) {
            Some(sys) => sys.dispose(self),
            _ => ()
        }
    }
    pub fn create_entity<E: Share>(&self) -> usize {
        let id = TypeId::of::<E>();
        match self.entity.get(&id) {
            Some(v) => match v.clone().downcast() {
                Ok(r) => {
                    let rc: Arc<CellEntity<E>> = r;
                    LendMut::lend_mut(&rc).create()
                },
                Err(_) => panic!("downcast err")
            }
            _ => panic!("not registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, id),
        }
    }
    pub fn free_entity<E: Share>(&self, id: usize) {
        let eid = TypeId::of::<E>();
        match self.entity.get(&eid) {
            Some(v) => match v.clone().downcast() {
                Ok(r) => {
                    let r: Arc<CellEntity<E>> = r;
                    LendMut::lend_mut(&r).delete(id);
                },
                Err(_) => panic!("downcast err")
            },
            _ => panic!("not registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, eid),
        }
    }
    pub fn add_dispatcher<D: Dispatcher + 'static>(&mut self, name: Atom, dispatcher: D) {
        self.runner.insert(name, Arc::new(dispatcher));
    }
    pub fn get_dispatcher(&self, name: &Atom) -> Option<&Arc<dyn Dispatcher>> {
        self.runner.get(name)
    }
    pub fn remove_dispatcher(&mut self, name: &Atom) -> Option<Arc<dyn Dispatcher>> {
        self.runner.remove(name)
    }
    pub fn fetch_entity<T: Share>(&self) -> Option<Arc<CellEntity<T>>> {
        let id = TypeId::of::<T>();
        let r = match self.entity.get(&id) {
            Some(v) => v.clone(),
            _ => return None
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }
    pub fn fetch_single<T: Share>(&self) -> Option<Arc<CellSingleCase<T>>> {
        let id = TypeId::of::<T>();
        let r = match self.single.get(&id) {
            Some(v) => v.clone(),
            _ => return None
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }
    pub fn fetch_multi<E: Share, C: Component>(&self) -> Option<Arc<CellMultiCase<E, C>>> {
        let eid = TypeId::of::<E>();
        let cid = TypeId::of::<C>();
        let r = match self.multi.get(&(eid, cid)) {
            Some(v) => v.clone(),
            _ => return None
        };
        match r.downcast() {
            Ok(r) => Some(r),
            Err(_) => panic!("downcast err"),
        }
    }

    pub fn run(&self, name: &Atom) {
        match self.runner.get(name) {
            Some(v) => v.run(),
            _ => ()
        }
    }
}
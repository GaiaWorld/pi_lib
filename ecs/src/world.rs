
use std::{
    sync::Arc,
    any::TypeId,
    intrinsics::type_name,
};


use im::hashmap::HashMap;

use atom::Atom;
use pointer::cell::{TrustCell};

use system::{System};
use entity::{Entity, EntityImpl, CellEntity};
use component::{SingleCase, MultiCase, CellMultiCase, MultiCaseImpl, Component};
use dispatch::Dispatcher;
use Share;


pub trait Fetch: Sized + 'static {
    fn fetch(world: &World) -> Self;
}

pub trait Borrow<'a> {
    type Target;
    fn borrow(&'a self) -> Self::Target;
}

pub trait BorrowMut<'a> {
    type Target;
    fn borrow_mut(&'a self) -> Self::Target;
}

pub trait TypeIds {
    fn type_ids() -> Vec<(TypeId, TypeId)>;
}

#[derive(Default)]
pub struct World {
    entity: HashMap<TypeId, Arc<Entity>>,
    single: HashMap<TypeId, Arc<SingleCase>>,
    multi: HashMap<(TypeId, TypeId), Arc<MultiCase>>,
    system: HashMap<Atom, Arc<System>>,
    runner: HashMap<Atom, Arc<Dispatcher>>,
}

impl World {
    pub fn register_entity<E: Share>(&mut self) {
        let id = TypeId::of::<E>();
        match self.entity.insert(id, Arc::new(TrustCell::new(EntityImpl::<E>::new()))) {
            Some(_) => panic!("duplicate registration, entity: {:?}, id: {:?}", unsafe{type_name::<E>()}, id),
            _ => ()
        }
    }
    /// 注册单例组件
    pub fn register_single<C: SingleCase>(&mut self, c: C) {
        let id = TypeId::of::<C>();
        match self.single.insert(id, Arc::new(c)) {
            Some(_) => panic!("duplicate registration, component: {:?}, id: {:?}", unsafe{type_name::<C>()}, id),
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
                        let mut entity = BorrowMut::borrow_mut(&r);
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
    pub fn register_system<T>(&mut self, name: Atom, sys: T) {
        let t = TrustCell::new(sys);
        
        // 如果是Runner则调用setup方法， 获取所有实现了监听器的类型，动态注册到对应的组件监听器上Atom
    }
    pub fn get_system(&self, name: &Atom) -> Option<&Arc<System>> {
        self.system.get(name)
    }
    pub fn unregister_system(&mut self, name: &Atom) {
        // 要求该system不能在dispatcher中， 取消所有的监听器
        // 如果是Runner则调用dispose方法
    }
    pub fn create_entity<E: Share>(&self) -> usize {
        let id = TypeId::of::<E>();
        match self.entity.get(&id) {
            Some(v) => match v.clone().downcast() {
                Ok(r) => {
                    let rc: Arc<CellEntity<E>> = r;
                    rc.borrow_mut().create()
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
                    r.borrow_mut().delete(id);
                },
                Err(_) => panic!("downcast err")
            },
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
    pub fn fetch_entity<T: 'static>(&self) -> Option<Arc<Entity>> {
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
    pub fn fetch_multi<E: Share, C: Component>(&self) -> Option<Arc<MultiCase>> {
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

macro_rules! impl_trait {
    ( $($ty:ident),* ) => {
        impl<$($ty),*> TypeIds for ( $( $ty , )* ) where $( $ty: TypeIds),*{
            fn type_ids() -> Vec<(TypeId, TypeId)> {
                let mut arr = Vec::new();
                $(arr.extend_from_slice( &$ty::type_ids() );)*
                arr
            }
        }

        impl<$($ty),*> Fetch for ( $( $ty , )* ) where $( $ty: Fetch),*{
            fn fetch(world: &World) -> Self {
                ( $($ty::fetch(world),)* )
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($ty),*> Borrow<'a> for ( $( $ty , )* ) where $( $ty: Borrow<'a>),*{
            type Target = ( $($ty::Target,)* );
            fn borrow(&'a self) -> Self::Target {
                let ($($ty,)*) = self;
                ( $($ty.borrow(),)* )
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($ty),*> BorrowMut<'a> for ( $( $ty , )* ) where $( $ty: BorrowMut<'a>),*{
            type Target = ( $($ty::Target,)* );
            fn borrow_mut(&'a self) -> Self::Target {
                let ( $($ty,)* ) = self;
                ( $($ty.borrow_mut(),)* )
            }
        }
    };
}

impl_trait!(A);
impl_trait!(A, B);
impl_trait!(A, B, C);
impl_trait!(A, B, C, D);
impl_trait!(A, B, C, D, E);
impl_trait!(A, B, C, D, E, F);
impl_trait!(A, B, C, D, E, F, G);
impl_trait!(A, B, C, D, E, F, G, H);
impl_trait!(A, B, C, D, E, F, G, H, I);
impl_trait!(A, B, C, D, E, F, G, H, I, J);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_trait!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
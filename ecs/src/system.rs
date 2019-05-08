
use std::{
    any::{TypeId},
};
use world::{ World, Fetch, TypeIds};
use listener::{Listener as LibListener, FnListeners};
pub use listener::FnListener;

pub trait Runner<'a> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn setup(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn dispose(&mut self, read: Self::ReadData, write: Self::WriteData);
}

pub trait SystemData<'a> where Self: std::marker::Sized{
    type FetchTarget: Fetch + TypeIds;
}

pub trait SystemMutData<'a> where Self: std::marker::Sized{
    type FetchTarget: Fetch + TypeIds;
}

pub struct CreateEvent{
    pub id: usize,
}

pub struct DeleteEvent{
    pub id: usize,
}

pub struct ModifyEvent{
    pub id: usize,
    pub field: &'static str,
    pub index: usize, // 一般无意义。 只有在数组或向量的元素被修改时，才有意义
}

/// E 是Entity的类型， 如果是单例组件， 则E为()。 C是组件类型， 如果仅监听Entity的创建和删除， 则C为()。 EV是事件类型
pub trait Listener<'a, E, C, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn listen(&mut self, event: &EV, read: Self::ReadData, write: Self::WriteData);
}

pub type CreateListeners = FnListeners<CreateEvent>;
pub type DeleteListeners = FnListeners<DeleteEvent>;
pub type ModifyListeners = FnListeners<ModifyEvent>;
pub type CreateFn = FnListener<CreateEvent>;
pub type DeleteFn = FnListener<DeleteEvent>;
pub type ModifyFn = FnListener<ModifyEvent>;
pub type RunnerFn = FnListener<()>;


#[derive(Default)]
pub struct NotifyImpl {
    pub create: CreateListeners,
    pub delete: DeleteListeners,
    pub modify: ModifyListeners,
}
impl NotifyImpl {
    pub fn create_event(&self, id: usize) {
        let e = CreateEvent{
            id: id,
        };
        self.create.listen(&e);
    }
    pub fn delete_event(&self, id: usize) {
        let e = DeleteEvent{
            id: id,
        };
        self.delete.listen(&e);
    }
    pub fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        let e = ModifyEvent{
            id: id,
            field: field,
            index: index,
        };
        self.modify.listen(&e);
    }
}

pub trait Notify {
    fn add_create(&self, CreateFn);
    fn add_delete(&self, DeleteFn);
    fn add_modify(&self, ModifyFn);
    fn create_event(&self, id: usize);
    fn delete_event(&self, id: usize);
    fn modify_event(&self, id: usize, field: &'static str, index: usize);
    fn remove_create(&self, &CreateFn);
    fn remove_delete(&self, &DeleteFn);
    fn remove_modify(&self, &ModifyFn);
}

pub trait System{ 
    fn fetch_setup(self, world: &World) -> Option<RunnerFn>;
    fn fetch_run(self, world: &World) -> Option<RunnerFn>;
    fn fetch_dispose(self, world: &World) -> Option<RunnerFn>;
    fn get_depends(&self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
}

pub trait Monitor<E, C, EV>{
    fn get_depends(&self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
    fn fetch_setup(self, world: &World) -> Result<(), String>;
}

#[macro_export]
macro_rules! impl_monitor {
    ($share_system: ident, $system: ident, {$(<$e: ident, $c: ident, $ev: ident>)*}) => {
        $(
            impl $crate::system::Monitor<$e, $c, $ev> for $share_system{
                fn get_depends(&self) -> (Vec<(std::any::TypeId, std::any::TypeId)>, Vec<(std::any::TypeId, std::any::TypeId)>) {
                    (
                        <<$system as $crate::system::Listener<'_, $e, $c, $ev>>::ReadData as $crate::system::SystemData>::FetchTarget::type_ids(), 
                        <<$system as $crate::system::Listener<'_, $e, $c, $ev>>::WriteData as $crate::system::SystemMutData>::FetchTarget::type_ids()
                    )
                }

                fn fetch_setup(self, world: &$crate::world::World) -> Result<(), String>{
                    let read = <<$system as $crate::system::Listener<'_, $e, $c, $ev>>::ReadData as $crate::system::SystemData>::FetchTarget::fetch(world);
                    let write = <<$system as $crate::system::Listener<'_, $e, $c, $ev>>::WriteData as $crate::system::SystemMutData>::FetchTarget::fetch(world);
                    let f = $crate::system::FnListener(std::sync::Arc::new( move |e: &CreateEvent| {
                        let read_data = read.borrow();
                        let write = write.borrow_mut();
                        self.0.borrow_mut().listen(e, read_data, write);
                    }));
                    let setup_target: Arc<CellMultiCase<Node, Position>> = match world.fetch_multi::<Node, Position>().unwrap().downcast() {
                        Ok(r) => r,
                        Err(_) => return Err("downcast err".to_string()),
                    };
                    Notify::add_create(&*setup_target, f);
                    Ok(())
                }
            }
        )*

        impl $crate::system::Monitor<(), (), ()> for $share_system {
            fn get_depends(&self) -> (Vec<(std::any::TypeId, std::any::TypeId)>, Vec<(std::any::TypeId, std::any::TypeId)>) {
                let mut read_ids = Vec::new();
                let mut write_ids = Vec::new();
                $(
                let ids = $crate::system::<$e, $c, $ev>::get_depends(self);
                read_ids.extend_from_slice(&ids.0);
                write_ids.extend_from_slice(&ids.1);
                )*
                (read_ids, write_ids)
            }

            fn fetch_setup(self, world: &$crate::world::World) -> Result<(), String>{
                $(
                $crate::system::Monitor::<$e, $c, $ec>::fetch_setup(self.clone(), world)?;
                )*
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_system {
    ($share_system: ident, $system: ident) => {
        impl $crate::system::System for $share_system{
            fn get_depends(&self) -> (Vec<(std::any::TypeId, std::any::TypeId)>, Vec<(std::any::TypeId, std::any::TypeId)>) {
                (
                    <<$system as $crate::system::Runner::ReadData as $crate::system::SystemData>::FetchTarget::type_ids(), 
                    <<$system as $crate::system::Runner::WriteData as $crate::system::SystemMutData>::FetchTarget::type_ids()
                )
            }

            fn fetch_setup(self, world: &$crate::world::World) -> Option<$crate::system::RunnerFn> {
                let read = <<$system as Runner>::ReadData as SystemData>::FetchTarget::fetch(world);
                let write = <<$system as Runner>::WriteData as SystemMutData>::FetchTarget::fetch(world);
                let f = move |_e: &()| {
                    let read_data = read.borrow();
                    let write_data = write.borrow_mut();
                    self.0.borrow_mut().setup(read_data, write_data);
                };
                Some($crate::system::FnListener(Arc::new(f)))
            }

            fn fetch_run(self, world: &$crate::world::World) -> Option<$crate::system::RunnerFn> {
                let read = <<$system as Runner>::ReadData as SystemData>::FetchTarget::fetch(world);
                let write = <<$system as Runner>::WriteData as SystemMutData>::FetchTarget::fetch(world);
                let f = move |_e: &()| {
                    let read_data = read.borrow();
                    let write_data = write.borrow_mut();
                    self.0.borrow_mut().run(read_data, write_data);
                };
                Some($crate::system::FnListener(Arc::new(f)))
            }

            fn fetch_dispose(self, world: &$crate::world::World) -> Option<$crate::system::RunnerFn> {
                let read = <<$system as Runner>::ReadData as SystemData>::FetchTarget::fetch(world);
                let write = <<$system as Runner>::WriteData as SystemMutData>::FetchTarget::fetch(world);
                let f = move |_e: &()| {
                    let read_data = read.borrow();
                    let write_data = write.borrow_mut();
                    self.0.borrow_mut().dispose(read_data, write_data);
                };
                Some($crate::system::FnListener(Arc::new(f)))
            }
        }
    };
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> SystemData<'a> for ( $( $ty , )* ) where $( $ty : SystemData<'a> ),*{
            type FetchTarget = ($($ty::FetchTarget,)*);
        }

        impl<'a, $($ty),*> SystemMutData<'a> for ( $( $ty , )* ) where $( $ty : SystemMutData<'a> ),*{
            type FetchTarget = ($($ty::FetchTarget,)*);
        } 
    };
}

impl_data!(A);
impl_data!(A, B);
impl_data!(A, B, C);
impl_data!(A, B, C, D);
impl_data!(A, B, C, D, E);
impl_data!(A, B, C, D, E, F);
impl_data!(A, B, C, D, E, F, G);
impl_data!(A, B, C, D, E, F, G, H);
impl_data!(A, B, C, D, E, F, G, H, I);
impl_data!(A, B, C, D, E, F, G, H, I, J);
impl_data!(A, B, C, D, E, F, G, H, I, J, K);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

// Node{};
// CharNode{};

// Pos{

// }

// pub struct Xy {};

// mod Xy{
//     const xx: HashMap<>;
// }

// struct CellXy = (TrustCell<Xy>);
// impl System for CellXy {
//     fn fetch_run(&self, me: Arc<Any>) -> Option<RunnerFn> {
//         let f = |e: &E| -> {
            
//             system.listen(e, &read_data, &mut write_data)
//         };
//         f
//     }
// }
// [#aa(dd)]
// impl Listener<T, Pos, CreateEvent> for Xy {
//     type ReadData = CellMultiCase<Node, WorldMatrix>;
//     type WriteData: Overflow;
//     fn listen(&mut self, event: &E, read: Self::ReadData, write: Self::WriteData) {

//     }
// }

// impl Listener<T, Pos, CreateEvent> for Xy {
//     install(world: &World) {
//         system;
//         let read_data = xxx.fetch(world: &World);
//         let write_data = xxx.fetch(world: &World);
//         let fn = |e: &E| -> {
//             system.listen(e, &read_data, &mut write_data)
//         };
//         let mut notify = world.get_notify<T, Pos>();
//         notify.create.push_back(Arc<fn>);
//     }
//     uninstall()
// }

// [#aa(dd)]
// impl Listener<T, Pos, DeleteEvent> for Xy {
//     type ReadData = MultiCase<Node, WorldMatrix>;
//     type WriteData: Overflow;
//     fn listen(&mut self, event: &E, read: Self::ReadData, write: Self::WriteData) {

//     }
// }
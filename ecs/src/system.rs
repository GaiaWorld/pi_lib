
use std::{
    sync::Arc,
    any::{TypeId},
};

use world::{ World, Fetch, Borrow, BorrowMut, TypeIds};
use listener::{Listener as Lis, FnListener, FnListeners};

pub trait Runner<'a> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn setup(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData);
    fn dispose(&mut self, read: Self::ReadData, write: Self::WriteData);
}

pub trait SystemData<'a> where Self: std::marker::Sized{
    type FetchTarget: Fetch + Borrow<'a, Target=Self> + TypeIds;
}

pub trait SystemMutData<'a> where Self: std::marker::Sized{
    type FetchTarget: Fetch + BorrowMut<'a, Target=Self> + TypeIds;
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


/// E 是Entity的类型， C是组件类型， EV是事件类型
pub trait MultiCaseListener<'a, E, C, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn listen(&mut self, event: &EV, read: &Self::ReadData, write: &mut Self::WriteData);
}

/// Entity监听器， 监听Entity的创建和删除， EV是事件类型
pub trait EntityListener<'a, E, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn listen(&mut self, event: &EV, read: &Self::ReadData, write: &mut Self::WriteData);
}
/// 单例组件监听器， EV是事件类型
pub trait SingleCaseListener<'a, C, EV> {
    type ReadData: SystemData<'a>;
    type WriteData: SystemMutData<'a>;

    fn listen(&mut self, event: &EV, read: &Self::ReadData, write: &mut Self::WriteData);
}

pub trait Monitor<'a> {
    fn notify(&mut self);
    fn get_depends(&'a self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
}

pub trait System<'a> {
    fn setup(&'a self);
    fn run(&'a self);
    fn dispose(&'a self);
    fn get_depends(&'a self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
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

// pub trait System {
//     fn get_depends(&self) -> (Vec<(TypeId, TypeId)>, Vec<(TypeId, TypeId)>);
//     fn fetch_setup(&self, me: Arc<System>, world: &World) -> Option<RunnerFn>;
//     fn fetch_run(&self, me: Arc<System>, world: &World) -> Option<RunnerFn>;
//     fn fetch_dispose(&self, me: Arc<System>, world: &World) -> Option<RunnerFn>;
// }

macro_rules! impl_data {
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
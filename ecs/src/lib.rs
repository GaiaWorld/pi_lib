#![feature(core_intrinsics)]
#![feature(proc_macro_hygiene)]

extern crate slab;
extern crate atom;
extern crate fnv;
extern crate map;
extern crate listener;
extern crate pointer;
#[macro_use]
extern crate any;

// extern crate im;
pub extern crate paste;

pub mod world;
#[macro_use]
pub mod system;
pub mod entity;
pub mod component;
pub mod dispatch;
pub mod single;
pub mod monitor;

pub mod idtree;

pub use world::World;
pub use system::{Runner, SingleCaseListener, MultiCaseListener, EntityListener, System};
pub use component::{Component, MultiCaseImpl};
pub use single::{SingleCaseImpl};
pub use monitor::{CreateEvent, ModifyEvent, DeleteEvent, Write};
pub use dispatch::{SeqDispatcher, Dispatcher};

use std::any::TypeId;

pub trait Share: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Share for T {}

pub trait Fetch: Sized + 'static {
    fn fetch(world: &World) -> Self;
}

pub trait Lend<'a> {
    type Target;
    fn lend(&'a self) -> Self::Target;
}

pub trait LendMut<'a> {
    type Target;
    fn lend_mut(&'a self) -> Self::Target;
}

pub trait TypeIds {
    fn type_ids() -> Vec<(TypeId, TypeId)>;
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
        impl<'a, $($ty),*> Lend<'a> for ( $( $ty , )* ) where $( $ty: Lend<'a>),*{
            type Target = ( $($ty::Target,)* );
            fn lend(&'a self) -> Self::Target {
                let ($($ty,)*) = self;
                ( $($ty.lend(),)* )
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($ty),*> LendMut<'a> for ( $( $ty , )* ) where $( $ty: LendMut<'a>),*{
            type Target = ( $($ty::Target,)* );
            fn lend_mut(&'a self) -> Self::Target {
                let ( $($ty,)* ) = self;
                ( $($ty.lend_mut(),)* )
            }
        }
    };
}

impl<'a> LendMut<'a> for () {
    type Target = ();
    fn lend_mut(&'a self) -> Self::Target {
        ()
    }
}

impl<'a> Lend<'a> for () {
    type Target = ();
    fn lend(&'a self) -> Self::Target {
        ()
    }
}

impl TypeIds for (){
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![]
    }
}

impl Fetch for (){
    fn fetch(_world: &World) -> Self {
        ()
    }
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

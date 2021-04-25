#![feature(core_intrinsics)]
#![feature(proc_macro_hygiene)]

extern crate atom;
extern crate listener;
extern crate map;
extern crate pointer;
extern crate slab;
#[macro_use]
extern crate any;
extern crate hash;
extern crate share;
// #[cfg(feature = "wasm-bindgen")]
// extern crate wasm_bindgen_cross_performance;
// #[cfg(feature = "native")]
// extern crate native_cross_performance;
// extern crate im;
pub extern crate paste;

pub extern crate time;

// #[cfg(feature = "wasm-bindgen")]
// pub crate use wasm_bindgen_cross_performance as cross_performance;
// #[cfg(feature = "native")]
// pub crate use native_cross_performance as cross_performance;

pub mod cell;
pub mod world;
#[macro_use]
pub mod system;
pub mod component;
pub mod dispatch;
pub mod entity;
pub mod monitor;
pub mod single;

pub use component::{CellMultiCase, Component, MultiCaseImpl};
pub use dispatch::{Dispatcher, SeqDispatcher};
pub use entity::{CellEntity, EntityImpl};
pub use monitor::{CreateEvent, DeleteEvent, ModifyEvent, Write};
pub use single::{CellSingleCase, SingleCaseImpl};
pub use system::{EntityListener, MultiCaseListener, Runner, SingleCaseListener, System};
pub use world::World;
pub use cell::StdCell;

use std::any::TypeId;

// pub static mut PRINT_TIME: bool = false;

// pub fn set_print(v: bool){
// 	unsafe {PRINT_TIME = v};
// }

pub trait Fetch: Sized + 'static {
    fn fetch(world: &World) -> Self;
}

pub trait Lend<'a> {
    type Target;
    type Target1;
    fn lend(&'a self) -> Self::Target;
    fn lend1(&'a self) -> Self::Target1;
    fn lend2(&'a self, ptr: &Self::Target1) -> Self::Target;
}

pub trait LendMut<'a> {
    type Target;
    type Target1;
    fn lend_mut(&'a self) -> Self::Target;
    fn lend_mut1(&'a self) -> Self::Target1;
    fn lend_mut2(&'a self, ptr: &Self::Target1) -> Self::Target;
}

pub trait TypeIds {
    fn type_ids() -> Vec<(TypeId, TypeId)>;
}

#[derive(Debug)]
pub struct RunTime {
    pub sys_name: atom::Atom,
    pub cost_time: std::time::Duration, // 单位ms
}

macro_rules! impl_trait {
    (( $($ty:ident),* ), ( $($name:ident),* ) ) => {
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
            type Target1 = ( $($ty::Target1,)* );

            fn lend1(&'a self) -> Self::Target1 {
                let ( $($ty,)* ) = self;
                ( $($ty.lend1(),)* )
            }

            fn lend2(&'a self, ptr: &Self::Target1) -> Self::Target {
                let ( $($name,)* ) = ptr;
                let ( $($ty,)* ) = self;
                ( $($ty.lend2($name),)* )
            }

            fn lend(&'a self) -> Self::Target {
                let ($($ty,)*) = self;
                ( $($ty.lend(),)* )
            }
        }

        #[allow(non_snake_case)]
        impl<'a, $($ty),*> LendMut<'a> for ( $( $ty , )* ) where $( $ty: LendMut<'a>),*{
            type Target = ( $($ty::Target,)* );
            type Target1 = ( $($ty::Target1,)* );

            fn lend_mut1(&'a self) -> Self::Target1 {
                let ( $($ty,)* ) = self;
                ( $($ty.lend_mut1(),)* )
            }

            fn lend_mut2(&'a self, ptr: &Self::Target1) -> Self::Target {
                let ( $($name,)* ) = ptr;
                let ( $($ty,)* ) = self;
                ( $($ty.lend_mut2($name),)* )
            }

            fn lend_mut(&'a self) -> Self::Target {
                let ( $($ty,)* ) = self;
                ( $($ty.lend_mut(),)* )
            }
        }
    };
}

impl<'a> LendMut<'a> for () {
    type Target = ();
    type Target1 = ();
    fn lend_mut(&'a self) -> Self::Target {
        ()
    }

    fn lend_mut1(&'a self) -> Self::Target1 {
        ()
    }

    fn lend_mut2(&'a self, _ptr: &Self::Target) -> Self::Target {
        ()
    }
}

impl<'a> Lend<'a> for () {
    type Target = ();
    type Target1 = ();
    fn lend(&'a self) -> Self::Target {
        ()
    }
    fn lend1(&'a self) -> Self::Target1 {
        ()
    }

    fn lend2(&'a self, _ptr: &Self::Target) -> Self::Target {
        ()
    }
}

impl TypeIds for () {
    fn type_ids() -> Vec<(TypeId, TypeId)> {
        vec![]
    }
}

impl Fetch for () {
    fn fetch(_world: &World) -> Self {
        ()
    }
}

impl_trait!((A), (a));
impl_trait!((A, B), (a, b));
impl_trait!((A, B, C), (a, b, c));
impl_trait!((A, B, C, D), (a, b, c, d));
impl_trait!((A, B, C, D, E), (a, b, c, d, e));
impl_trait!((A, B, C, D, E, F), (a, b, c, d, e, f));
impl_trait!((A, B, C, D, E, F, G), (a, b, c, d, e, f, g));
impl_trait!((A, B, C, D, E, F, G, H), (a, b, c, d, e, f, g, h));
impl_trait!((A, B, C, D, E, F, G, H, I), (a, b, c, d, e, f, g, h, i));
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J),
    (a, b, c, d, e, f, g, h, i, j)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K),
    (a, b, c, d, e, f, g, h, i, j, k)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (a, b, c, d, e, f, g, h, i, j, k, l)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (a, b, c, d, e, f, g, h, i, j, k, l, m)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y)
);
impl_trait!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z),
    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z)
);

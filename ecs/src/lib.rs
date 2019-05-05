#![feature(core_intrinsics)]

extern crate slab;
extern crate atom;
extern crate fnv;
extern crate map;
extern crate listener;
extern crate pointer;

extern crate im;

#[macro_use]
extern crate mopa;

pub mod world;
pub mod system;
pub mod entity;
pub mod compment;

pub mod idtree;
pub mod dispatch;
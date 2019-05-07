#![feature(core_intrinsics)]

extern crate slab;
extern crate atom;
extern crate fnv;
extern crate map;
extern crate listener;
extern crate pointer;

extern crate im;

#[macro_use]
extern crate downcast_rs;

pub mod world;
pub mod system;
pub mod entity;
pub mod component;

pub mod idtree;
pub mod dispatch;
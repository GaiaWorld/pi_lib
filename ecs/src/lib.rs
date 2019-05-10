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

extern crate im;
pub extern crate paste;

pub mod world;
pub mod system;
pub mod entity;
pub mod component;
pub mod dispatch;
pub mod single;
pub mod monitor;

pub mod idtree;
pub mod idtree_sys;
pub mod dirty;

pub trait Share: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Share for T {}

#![feature(weak_counts)]

extern crate deque;
extern crate hash;
extern crate lru;
extern crate share;
extern crate slab;
#[macro_use]
extern crate any;

mod res_map;
mod res_mgr;

pub use res_map::*;
pub use res_mgr::*;

#![feature(weak_counts)]

extern crate slab;
extern crate deque;
extern crate lru;
extern crate share;
extern crate hash;
#[macro_use]
extern crate any;

mod res_map;
mod res_mgr;

pub use res_map::*;
pub use res_mgr::*;

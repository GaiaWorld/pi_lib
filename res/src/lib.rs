
extern crate deque;
extern crate hash;
extern crate lru;

extern crate slab;
#[macro_use]
extern crate any;
extern crate share;
extern crate log;
#[macro_use]
extern crate serde;


// extern crate web_sys;

mod res_map;
mod res_mgr;

pub use res_map::*;
pub use res_mgr::*;

#![feature(asm)]
#![feature(libc)]
#![feature(fnbox)]
#![feature(drain_filter)]
#![feature(rustc_private)]
#![feature(type_ascription)]
#![feature(slice_internals)]
#![feature(integer_atomics)]

extern crate atom;
extern crate fnv;
extern crate rand;
extern crate threadpool;

#[macro_use]
extern crate lazy_static;

pub mod task_pool;
pub mod task;
pub mod worker_pool;
pub mod worker;
pub mod impls;
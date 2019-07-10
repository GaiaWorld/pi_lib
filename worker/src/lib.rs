#![feature(asm)]
#![feature(libc)]
#![feature(drain_filter)]
#![feature(rustc_private)]
#![feature(type_ascription)]
#![feature(slice_internals)]
#![feature(integer_atomics)]
#![feature(proc_macro_hygiene)]
#![feature(wait_timeout_until)]

extern crate fnv;
extern crate rand;
extern crate threadpool;

#[macro_use]
extern crate lazy_static;

extern crate atom;
extern crate apm;
extern crate timer;
extern crate task_pool;

pub mod impls;
pub mod task;
//pub mod task_pool;
pub mod worker;
pub mod worker_pool;

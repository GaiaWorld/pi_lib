#![feature(asm)]
#![feature(libc)]
#![feature(drain_filter)]
#![feature(rustc_private)]
#![feature(type_ascription)]
#![feature(slice_internals)]
#![feature(integer_atomics)]

extern crate npnc;
extern crate notify;
#[macro_use]
extern crate lazy_static;
extern crate atom;
extern crate apm;

pub mod fs_monitor;


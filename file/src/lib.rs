#![feature(asm)]
#![feature(libc)]
#![feature(fnbox)]
#![feature(drain_filter)]
#![feature(rustc_private)]
#![feature(type_ascription)]
#![feature(slice_internals)]
#![feature(integer_atomics)]

extern crate atom;
extern crate worker;
extern crate npnc;
extern crate notify;

pub mod file;
pub mod fs_monitor;



#![crate_type = "rlib"]
#![feature(integer_atomics)]
#![feature(duration_extras)]
#![feature(custom_derive,asm,box_syntax,box_patterns)]
#![feature(pointer_methods)]
#![feature(core_intrinsics)]
#![feature(generators, generator_trait)]
#![feature(associated_type_defaults)]
#![feature(trait_alias)]
#![feature(exclusive_range_pattern)]
#![feature(nll)]
#![feature(box_into_raw_non_null)]
#[allow(dead_code,unused_variables,non_snake_case,unused_parens,unused_assignments,unused_unsafe,unused_imports)]

extern crate core;
extern crate fnv;

pub mod slab;
pub mod rc;
pub mod ordmap;
#[macro_use]
pub mod sbtree;
pub mod asbtree;
pub mod bon;
pub mod data_view;
pub mod atom;
pub mod sinfo;
pub mod util;
pub mod guid;
pub mod time;
pub mod cowlist;

#[macro_use]
extern crate lazy_static;

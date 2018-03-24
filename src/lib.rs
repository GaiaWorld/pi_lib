
#![crate_type = "rlib"]
#![feature(custom_derive,asm,box_syntax,box_patterns)]
#![feature(pointer_methods)]
#[allow(dead_code,unused_variables,non_snake_case,unused_parens,unused_assignments,unused_unsafe,unused_imports)]

pub mod slab;
pub mod rc;
pub mod ordmap;
pub mod sbtree;
pub mod bon;
pub mod data_view;

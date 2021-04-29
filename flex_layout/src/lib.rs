
// #![feature(assoc_int_consts)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]
#[cfg(not(feature = "std"))]
extern crate alloc;

extern crate slab;

extern crate paste;

extern crate idtree;
extern crate dirty;
extern crate map;

#[macro_use]
extern crate debug_info;

#[macro_use]
extern crate serde;

mod geometry;
mod number;
pub mod style;
mod tree;
mod calc;


pub use crate::tree::*;
pub use crate::geometry::*;
pub use crate::number::*;
pub use crate::style::*;
pub use crate::tree::*;
pub use crate::calc::*;

#![feature(core_intrinsics)]
#![feature(proc_macro_hygiene)]

extern crate atom;
extern crate listener;
extern crate map;
extern crate pointer;
extern crate slab;
#[macro_use]
extern crate any;
extern crate hash;
extern crate share;
// #[cfg(feature = "wasm-bindgen")]
// extern crate wasm_bindgen_cross_performance;
// #[cfg(feature = "native")]
// extern crate native_cross_performance;
// extern crate im;
pub extern crate paste;

pub extern crate time;
extern crate log;

// pub extern crate web_sys;

// #[cfg(feature = "wasm-bindgen")]
// pub crate use wasm_bindgen_cross_performance as cross_performance;
// #[cfg(feature = "native")]
// pub crate use native_cross_performance as cross_performance;

pub mod world;
pub mod component;
pub mod entity;
pub mod archetype;
pub mod sys;
pub mod storage;
pub mod query;

pub use world::World;

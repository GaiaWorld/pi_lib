#![feature(core_intrinsics)]
#![feature(proc_macro_hygiene)]

#[cfg(feature = "web")]
extern crate wasm_bindgen;
#[cfg(feature = "web")]
extern crate js_sys;
#[cfg(feature = "web")]
extern crate web_sys;

pub mod time;
#[macro_use]
extern crate lazy_static;

use std::time::Instant;
use std::time::Duration;
lazy_static! {
    static ref START: Instant = Instant::now();
}

#[cfg(not(feature = "native"))]
pub fn now() -> f64 {
    match Instant::now().checked_duration_since(START.clone()) {
        Some(r) => r.as_millis() as f64,
        None => 0.0,
    }
}

#[cfg(feature = "wasm-bindgen")]
pub fn now() -> f64 {
    web_sys::window()
        .expect("should have a Window")
        .performance()
        .expect("should have a Performance")
        .now()
}
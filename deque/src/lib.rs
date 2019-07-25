
/// A double-ended queue implemented with a link and map.
///
/// Support for appending to the back and popping from the back.
/// Support for prepending to the front and popping from the front.
/// Supports quick deletion of specified elements
/// 
/// extern crate slab;
/// 
#[cfg(test)]
extern crate time;

extern crate slab;
extern crate map;
extern crate ver_index;

pub mod deque;
pub mod slab_deque;

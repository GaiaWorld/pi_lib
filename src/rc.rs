/**
 * 计数器，一般内置在结构体中
 */

use std::sync::atomic::{AtomicUsize, Ordering};


pub trait Counter {
	fn new(usize) -> Self;
	fn incr(&mut self, usize) -> usize;
	fn decr(&mut self, usize) -> usize;
	fn count(self) -> usize;
}

pub struct ACounter {
	value: AtomicUsize,
}

pub struct NCounter {
	value: usize,
}

const MAX_REFCOUNT: usize = 0x7fffffff;

impl Counter for ACounter {
	fn new(c: usize) -> Self {
		ACounter { value: AtomicUsize::new(c)}
	}
	fn incr(&mut self, c: usize) -> usize {
		let old_size = self.value.fetch_add(c, Ordering::Relaxed);
		assert!( old_size + c < MAX_REFCOUNT, "count overflow");
		old_size
	}
	fn decr(&mut self, c: usize) -> usize {
		let old_size = self.value.fetch_sub(c, Ordering::Release);
		assert!( old_size - c < MAX_REFCOUNT, "count overflow");
		old_size
	}
	fn count(self) -> usize {
		self.value.into_inner()
	}
}


impl Counter for NCounter {
	fn new(c: usize) -> Self {
		NCounter { value: c}
	}
	fn incr(&mut self, c: usize)-> usize {
		let old_size = self.value;
		self.value+= c;
		assert!( old_size + c < MAX_REFCOUNT, "count overflow");
		old_size
	}
	fn decr(&mut self, c: usize)-> usize {
		let old_size = self.value;
		self.value-= c;
		assert!( old_size - c < MAX_REFCOUNT, "count overflow");
		old_size
	}
	fn count(self) -> usize {
		self.value
	}
}


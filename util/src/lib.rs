/**
 * 通用函数库
 */
extern crate libc;

use std::vec::Vec;
use std::sync::Arc;

use libc::c_void;

/*
* 将box转换为*const c_void
*/
#[inline]
pub fn box2void<T>(ptr_box: Box<T>) -> *const c_void {
    Box::into_raw(ptr_box) as *const c_void
}

/*
* 将*mut c_void转换为box
*/
#[inline]
pub fn void2box<T>(ptr_void: *mut c_void) -> Box<T> {
    unsafe { Box::from_raw(ptr_void as *mut T) }
}

/*
* 将Arc转换为*const c_void
*/
#[inline]
pub fn arc2void<T>(ptr_box: Arc<T>) -> *const c_void {
    Arc::into_raw(ptr_box) as *const c_void
}

/*
* 将*mut c_void转换为Arc
*/
#[inline]
pub fn void2arc<T>(ptr_void: *mut c_void) -> Arc<T> {
    unsafe { Arc::from_raw(ptr_void as *mut T) }
}

/*
* 将*const c_void转换为usize
*/
#[inline]
pub fn void2usize(ptr_void: *const c_void) -> usize {
    ptr_void as usize
}

/*
* 将usize转换为*const c_void
*/
#[inline]
pub fn usize2void(ptr: usize) -> *const c_void {
    ptr as *const c_void
}


// 为Vec增加的新方法
pub trait VecIndex {
	type Item;
	fn index(&self, item: &Self::Item) -> Option<usize>;
	fn swap_delete(&mut self, item: &Self::Item) -> Option<Self::Item>;
}

impl<T: PartialEq> VecIndex for Vec<T> {
	type Item = T;
	#[inline]
	fn index(&self, item: &T) -> Option<usize> {
        for i in 0..self.len() {
            if unsafe {self.get_unchecked(i)} == item {
                return Some(i)
            }
        }
        None
	}
	#[inline]
	fn swap_delete(&mut self, item: &T) -> Option<T> {
		match self.index(item) {
			Some(i) => Some(self.swap_remove(i)),
			_ => None,
		}
	}
}

#[inline]
pub fn err_string<T, E: ToString>(err: Result<T, E>) -> Result<T, String>{
	match err {
		Ok(o) => Ok(o),
		Err(e) => Err(e.to_string())
	}
}

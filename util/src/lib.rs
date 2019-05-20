/**
 * 通用函数库
 */

use std::{
    process,
};

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

// 为Option增加的新方法
pub trait Fetch {
	type Item;
	fn fetch(self) -> Self::Item;
}
impl<T> Fetch for Option<T> {
    type Item = T;
	#[inline]
	fn fetch(self) -> Self::Item{
        match self {
            Some(t) => t,
            _ => process::abort(),
        }
	}
}
// 为Option增加的新方法
pub trait FetchDefault {
	type Item: Default;
	fn fetch_default(self) -> Self::Item;
}
impl<T: Default> FetchDefault for Option<T> {
    type Item = T;
	#[inline]
	fn fetch_default(self) -> Self::Item{
        match self {
            Some(t) => t,
            _ => Self::Item::default(),
        }
	}
}
// 为Option增加的新方法
pub trait FetchClone {
	type Item: Default + Clone;
	fn fetch_clone(self) -> Self::Item;
}
impl<T: Default + Clone> FetchClone for Option<T> {
    type Item = T;
	#[inline]
	fn fetch_clone(self) -> Self::Item{
        match self {
            Some(t) => t.clone(),
            _ => Self::Item::default(),
        }
	}
}

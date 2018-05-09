/**
 * 通用函数库
 */

use std::vec::Vec;
use std::ops::{Generator, GeneratorState};

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
		self.iter().position(|x| *x == *item)
	}
	#[inline]
	fn swap_delete(&mut self, item: &T) -> Option<T> {
		match self.index(item) {
			Some(i) => Some(self.swap_remove(i)),
			_ => None,
		}
	}
}

// 将生成器转成迭代器
pub fn generator_to_iterator<G>(g: G) -> impl Iterator<Item = G::Yield>
where
	G: Generator<Return = ()>
{
	struct It<G>(G);
	impl<G: Generator<Return = ()>> Iterator for It<G> {
		type Item = G::Yield;

		fn next(&mut self) -> Option<Self::Item> {
			match self.0.resume() {
				GeneratorState::Yielded(y) => Some(y),
				GeneratorState::Complete(_) => None,
			}
		}
	}
	It(g)
}

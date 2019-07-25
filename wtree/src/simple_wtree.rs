/**
 * 一个更快速的权重树，不支持索引删除
 */
use std::fmt::{Debug, Formatter, Result};

use map::{Empty};
use wtree::WeightTree;

pub struct SimpleWeightTree<T> {
    empty: Empty<(), usize>,
    wtree: WeightTree<T, ()>,
}
impl<T> Default for SimpleWeightTree<T> {
    fn default() -> Self {
        SimpleWeightTree{
            empty: Empty::default(),
            wtree: WeightTree::default(),
        }
    }
}

impl<T> SimpleWeightTree<T> {

	//创建一颗权重树， 并初始容量
	pub fn with_capacity(capacity: usize) -> Self{
		SimpleWeightTree{
            empty: Empty::default(),
            wtree: WeightTree::with_capacity(capacity),
        }
	}

    #[inline]
	pub fn amount(&self) -> usize{
		self.wtree.amount()
	}

    #[inline]
	pub fn len(&self) -> usize{
		self.wtree.len()
	}

    #[inline]
	pub fn clear(&mut self) {
		self.wtree.clear()
	}

	pub fn push(&mut self, obj: T, weight: usize){
		self.wtree.push(obj, weight, (), &mut self.empty);
	}

	pub unsafe fn pop_unchecked(&mut self, weight: usize) -> (T, usize, ()){
		 self.wtree.pop_unchecked(weight, &mut self.empty)
	}

	pub fn pop(&mut self, weight: usize) -> Option<(T, usize, ())>{
		self.wtree.pop(weight, &mut self.empty)
	}

}

impl<T: Debug> Debug for SimpleWeightTree<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write!(fmt,
               "SimpleWeightTree({:?})",
               self,
        )
    }
}
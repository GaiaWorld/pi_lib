use std::fmt::{Debug, Formatter, Result as FResult};

use dyn_uint::{UintFactory, SlabFactory};
use wtree::WeightTree as Wtree;

pub struct WeightTree<T> {
    index_factory: SlabFactory<(), ()>,
    wtree: Wtree<T>,
}

impl<T> WeightTree<T> {

	//构建一颗权重树
	pub fn new() -> Self{
        WeightTree{
            index_factory: SlabFactory::new(),
            wtree: Wtree::new(),
        }
	}

	//创建一颗权重树， 并初始容量
	pub fn with_capacity(capacity: usize) -> Self{
		WeightTree{
            index_factory: SlabFactory::new(),
            wtree: Wtree::with_capacity(capacity),
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

	pub fn push(&mut self, elem: T, weight: usize){
        let index = self.index_factory.create(0, (), ());
		self.wtree.push(elem, weight, index, &mut self.index_factory);
	}

	pub fn remove(&mut self, index: usize) -> (T, usize, usize){
		let r = unsafe { self.wtree.delete(self.index_factory.load(index), &mut self.index_factory) };
        self.index_factory.destroy(index);
        r
	}

	pub fn try_remove(&mut self, index: usize) -> Option<(T, usize, usize)>{
        match self.index_factory.try_load(index) {
            Some(i) => {
                let r = unsafe { self.wtree.delete(i, &mut self.index_factory) };
                self.index_factory.destroy(index);
                Some(r)
            },
            None => None,
        }
	}

	pub fn pop(&mut self, weight: usize) -> (T, usize, usize){
		unsafe { self.wtree.pop(weight, &mut self.index_factory) }
	}

	pub fn try_pop(&mut self, weight: usize) -> Option<(T, usize, usize)>{
		self.wtree.try_pop(weight, &mut self.index_factory)
	}

    #[inline]
	pub fn get(&self, index: usize) -> Option<&T>{
        match self.index_factory.try_load(index) {
            Some(i) =>  Some(unsafe{self.wtree.get_unchecked(i)}),
            None => None,
        }
	}

    #[inline]
	pub fn get_mut(&mut self, index: usize) -> Option<&mut T>{
		match self.index_factory.try_load(index) {
            Some(i) => Some(unsafe{self.wtree.get_unchecked_mut(i)}),
            None => None,
        }
	}

    #[inline]
	pub fn update_weight(&mut self, weight: usize, index: usize){
		unsafe{self.wtree.update_weight(weight, self.index_factory.load(index), &mut self.index_factory)}
	}

    #[inline]
	pub fn try_update_weight(&mut self, weight: usize, index: usize) -> bool{
        if let Some(i) = self.index_factory.try_load(index) {
            unsafe{self.wtree.update_weight(weight, i, &mut self.index_factory)};
            true
        }else {
            false
        }
	}
}

impl<T: Debug> Debug for WeightTree<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "SlabWeightTree(index_factory: {:?}, wtree: {:?})",
               self.index_factory,
               self.wtree,
        )
    }
}


#[test]
fn test(){
	let mut wtree: WeightTree<u32> = WeightTree::new();
	wtree.push(100, 100);
	wtree.push(2000, 2000);
	wtree.push(50, 50);
	wtree.push(70, 70);
	wtree.push(500, 500);
	wtree.push(20, 20);
	assert_eq!(wtree.amount(), 2740);

	wtree.update_weight(60, 6);
	assert_eq!(wtree.amount(), 2780);

	wtree.update_weight(20, 6);
	assert_eq!(wtree.amount(), 2740);

	assert_eq!(wtree.pop(2739).1, 20);
	assert_eq!(wtree.amount(), 2720);

	assert_eq!(wtree.pop(2000).1, 500);
	assert_eq!(wtree.amount(), 2220);
	
	assert_eq!(wtree.pop(1999).1, 2000);
	assert_eq!(wtree.amount(), 220);

	wtree.push(30, 30);
	wtree.update_weight(80, 7);

	assert_eq!(wtree.pop(140).1, 80);
	assert_eq!(wtree.amount(), 220);

}

#[cfg(test)]
use time::now_millisecond;
#[cfg(test)]
use rand::Rng;
#[cfg(test)]
use std::collections::VecDeque;

#[test]
fn test_effic(){
	let mut weight_tree: WeightTree<u32> = WeightTree::new();
	let max = 100000;
	let now = now_millisecond();
	for i in 0..max{
		weight_tree.push(i, (i+1) as usize);
	}
	println!("slab_wtree push max_heap time{}",  now_millisecond() - now);

	let mut arr = VecDeque::new();
	let now = now_millisecond();
	for i in 0..max{
		arr.push_front(i);
	}
	println!("push VecDeque time{}",  now_millisecond() - now);

	let now = now_millisecond();
	for _ in 0..max{
		rand::thread_rng().gen_range(0, 100000);
	}
	println!("slab_wtree rand time{}",  now_millisecond() - now);


	let now = now_millisecond();
	for _ in 0..max{
		//let r = rand::thread_rng().gen_range(0, weight_tree.amount());
		weight_tree.try_pop(1);
	}
	println!("slab_wtree pop time{}",  now_millisecond() - now);
}

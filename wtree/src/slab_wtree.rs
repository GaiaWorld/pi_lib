use std::fmt::{Debug, Formatter, Result as FResult};

use index_class::{IndexClassFactory};
use ver_index::VerIndex;
use wtree::WeightTree;


pub struct SlabWeightTree<T, I:VerIndex> {
    factory: IndexClassFactory<(), (), I>,
    wtree: WeightTree<T, I::ID>,
}
impl<T, I:VerIndex+Default> Default for SlabWeightTree<T, I> {
    fn default() -> Self {
        SlabWeightTree{
            factory: IndexClassFactory::default(),
            wtree: WeightTree::default(),
        }
    }
}
impl<T, I:VerIndex+Default> SlabWeightTree<T, I> {

	//创建一颗权重树， 并初始容量
	pub fn with_capacity(capacity: usize) -> Self{
		SlabWeightTree{
            factory: IndexClassFactory::default(),
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

	pub fn push(&mut self, obj: T, weight: usize) -> I::ID {
        let obj_id = self.factory.create(0, (), ());
		self.wtree.push(obj, weight, obj_id, &mut self.factory);
		obj_id
	}

	pub fn remove(&mut self, obj_id: I::ID) -> Option<(T, usize, I::ID)>{
        match self.factory.remove(obj_id) {
            Some(i) => {
                let r = unsafe { self.wtree.delete(i.index, &mut self.factory) };
                Some(r)
            },
            None => None,
        }
	}

	pub unsafe fn pop_unchecked(&mut self, weight: usize) -> (T, usize, I::ID){
		 self.wtree.pop_unchecked(weight, &mut self.factory)
	}

	pub fn pop(&mut self, weight: usize) -> Option<(T, usize, I::ID)>{
		self.wtree.pop(weight, &mut self.factory)
	}

    #[inline]
	pub fn get(&self, id: I::ID) -> Option<(&T, usize, I::ID)>{
        match self.factory.get(id) {
            Some(i) => Some(unsafe{self.wtree.get_unchecked(i.index)}),
            None => None,
        }
	}

    #[inline]
	pub fn get_mut(&mut self, id: I::ID) -> Option<(&mut T, usize, I::ID)>{
		match self.factory.get(id) {
            Some(i) => Some(unsafe{self.wtree.get_unchecked_mut(i.index)}),
            None => None,
        }
	}

    #[inline]
	pub unsafe fn update_weight_unchecked(&mut self, id: I::ID, weight: usize){
		self.wtree.update_weight(self.factory.get_unchecked(id).index, weight, &mut self.factory)
	}

    #[inline]
	pub fn update_weight(&mut self, id: I::ID, weight: usize) -> bool{
        if let Some(i) = self.factory.get(id) {
            unsafe{self.wtree.update_weight(i.index, weight, &mut self.factory)};
            true
        }else {
            false
        }
	}
}

impl<T: Debug, I:VerIndex> Debug for SlabWeightTree<T, I> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "SlabSlabWeightTree(factory: {:?}, wtree: {:?})",
               self.factory,
               self.wtree,
        )
    }
}


#[test]
fn test(){
	use ver_index::ver::U32Index;
	let mut wtree: SlabWeightTree<u32, U32Index> = SlabWeightTree::default();
	wtree.push(100, 100);
	wtree.push(2000, 2000);
	wtree.push(50, 50);
	wtree.push(70, 70);
	wtree.push(500, 500);
	let r6=wtree.push(20, 20);
	println!("u0------------{:?}", wtree);
	assert_eq!(wtree.amount(), 2740);

	wtree.update_weight(r6, 60);
	println!("u1------------{:?}", wtree);
	assert_eq!(wtree.amount(), 2780);

	wtree.update_weight(r6, 20);
	assert_eq!(wtree.amount(), 2740);

	assert_eq!(wtree.pop(2739).unwrap().1, 20);
	assert_eq!(wtree.amount(), 2720);

	assert_eq!(wtree.pop(2000).unwrap().1, 500);
	assert_eq!(wtree.amount(), 2220);
	
	assert_eq!(wtree.pop(1999).unwrap().1, 2000);
	assert_eq!(wtree.amount(), 220);

	let r7 = wtree.push(30, 30);
	wtree.update_weight(r7, 80);

	assert_eq!(wtree.pop(140).unwrap().1, 80);
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
	use ver_index::ver::U32Index;
	let mut weight_tree: SlabWeightTree<u32, U32Index> = SlabWeightTree::default();
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
		weight_tree.pop(1);
	}
	println!("slab_wtree pop time{}",  now_millisecond() - now);
}

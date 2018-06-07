/**
 * 定长分配器，支持单线程或多线程安全，采用动态分配的桶。利用位索引进行查找加速。
 */


// use std::rc::Rc;
// use std::sync::Arc;
// use std::sync::atomic::AtomicUsize;
// use std::vec;
// use std::mem::size_of;
// use std::usize::MAX;
// use std::marker::PhantomData;
//use std::marker::Copy;

/**
 * 默认4兆为1个块
 */
pub const LIMIT_MEM_SIZE: usize = 0x400000;

pub trait Slab<T:Copy + Clone> {
	fn new(init_block_size: usize, max_block_size: usize) -> Self;
	fn capity(&self) -> usize;
	fn count(&self) -> usize;
	fn get(&self, usize) -> &T;
	fn get_mut(&mut self, usize) -> &mut T;
	fn alloc(&mut self) -> usize;
	fn free(&mut self, usize);
	fn collect(&mut self);
}

// pub struct ASlab<T:Copy + Clone> {
// 	root: Arc<NSlab<T>>,
// }
// pub struct NSlab<T:Copy + Clone> {
// 	size: usize,
// 	limit: usize,
// 	item_size: usize,
// 	split_size: usize,
// 	capity: usize,
// 	count: usize,
// 	used: Vec<usize>,
// 	arr: Vec<Vec<T>>,
// 	//_marker: PhantomData<T>,
// }

// impl<T:Copy + Clone> Slab<T> for NSlab<T> {
// 	fn new(init_block_size: usize, max_block_size: usize) -> Self {
// 		let c = size_of::<T>();
// 		NSlab {
// 			size: c,
// 			limit: max_block_size,
// 			item_size: max_block_size / c,
// 			split_size: find_one(max_block_size / c) << 1,
// 			capity: 0,
// 			count: 0,
// 			used: Vec::with_capacity(6),
// 			arr: Vec::new(),
// 		}
// 	}
// 	fn capity(&self) -> usize {
// 		self.capity
// 	}
// 	fn count(&self) -> usize {
// 		self.count
// 	}
// 	fn get(&self, i: usize) -> &T {
// 		&self.arr[0][0]

// 	}
// 	fn get_mut(&mut self, i: usize) -> &mut T {
// 		&mut self.arr[0][0]

// 	}
// 	fn alloc(&mut self) -> usize {
// 		0

// 	}
// 	fn free(&mut self, i: usize) {

// 	}
// 	fn collect(&mut self) {

// 	}
// }

// // impl Alloc<T> for NAlloc {
// // 	fn new(c: usize) -> Self {
// // 		ACounter { value: AtomicUsize::new(c)}
// // 	}
// // }
// // #[derive(Copy, Clone)]
// // struct Item<T:Copy + Clone> {
// // 	arr: Vec<T>,
// // }
// const START_SIZE: usize = 64;

// //impl<T> Clone for 

// // 返回指定的数字中低位第一个0的位置
// pub fn find_zero(i:usize) -> usize {
// 	let a = !i;
// 	one_index(a - (a & (a - 1)))
// }
// // 找到指定的数字中低位第一个0的位置，将其改为1，返回位置
// pub fn zero2one(i: &mut usize) -> usize {
// 	let a = !(*i);
// 	let c = a - (a & (a - 1));
// 	*i = a | c;
// 	one_index(c)
// }
// // 返回指定的数字中高位第一个1的位置
// pub fn find_one(i:usize) -> usize {
// 	let a = !i;
// 	let c = a - (a & (a - 1));
// 	one_index(c)
// }

// #[inline]
// fn one_index(i: usize) -> usize {
// 	match i {
// 		0b1 => 0,
// 		0b10 => 1,
// 		4 => 2,
// 		8 => 3,
// 		16 => 4,
// 		32 => 5,
// 		64 => 6,
// 		128 => 7,
// 		256 => 8,
// 		_ => 0,
// 	}
// }
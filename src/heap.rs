/**
 * 线程不安全的堆，支持根据索引快速调整位置或删除
 */

use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::Arc;
use std::cmp::{Ord, Ordering};
use std::mem::{swap};

pub struct Heap<T>(Vec<(T, Arc<AtomicUsize>)>, Ordering);

impl<T: Ord> Heap<T> {

	//构建一个堆, 如果ord为Ordering::Less, 将创建一个小堆, 如果为Ordering::Greater，将创建一个大堆, 不应该使用Ordering::Equal创建一个堆
	pub fn new(ord: Ordering) -> Self{
		match ord {
			Ordering::Equal => {panic!("You can't create a heap with Ordering::Equal");},
			_ => Heap(Vec::new(), ord),
		}
	}

	//创建一个堆， 并初始容量
	pub fn with_capacity(capacity: usize, ord: Ordering) -> Self{
		match ord {
			Ordering::Equal => {panic!("You can't create a heap with Ordering::Equal");},
			_ => Heap(Vec::with_capacity(capacity), ord),
		}
	}

	//插入元素，返回该元素的位置
	pub fn push(&mut self, elem: T) -> Arc<AtomicUsize>{
		let len = self.0.len();
		self.0.push((elem, Arc::new(new_index(len + 1))));
		self.up(len)
	}

	//remove a element by index, Panics if index is out of bounds.
	pub fn remove(&mut self, index: &Arc<AtomicUsize>) -> T{
		let i = load_index(&index);
		self.delete((i - 1) as usize, self.0.len()).0
	}

	pub fn remove_top(&mut self) -> (T, Arc<AtomicUsize>){
		self.delete(0, self.0.len())
	}

	//remove a element by index; returns it, or None if it is not exist;
	pub fn try_remove(&mut self, index: &Arc<AtomicUsize>) -> Option<T>{
		let i = load_index(&index);
		if i == 0{
			return None;
		}
		self.try_delete((i - 1) as usize)
	}

	//Removes the top element from the pile and returns it, or None if it is empty 
	pub fn pop(&mut self) -> Option<T>{
		self.try_delete(0)
	}

	

	pub fn get_top(&mut self) -> Option<&T>{
		match self.0.get(0){
			Some(v) => Some(&v.0),
			None => {None}
		}
	}

	pub fn get(&self, index: &Arc<AtomicUsize>) -> Option<&T>{
		let i = load_index(&index);
		match self.0.get((i-1) as usize){
			Some(v) => Some(&v.0),
			None => {None}
		}
	}

	pub fn get_mut(&mut self, index: &Arc<AtomicUsize>) -> Option<&mut T>{
		let i = load_index(&index);
		match self.0.get_mut((i-1) as usize){
			Some(v) => Some(&mut v.0),
			None => None
		}
	}

	pub fn len(&self) -> usize{
		self.0.len()
	}

	//清空
	pub fn clear(&mut self) {
		self.0.clear();
	}

	#[inline]
	fn delete(&mut self, index: usize, len: usize) -> (T, Arc<AtomicUsize>){
		let mut elem = self.0.pop().unwrap();
		if index + 1 < len{//如果需要移除的元素不是堆底元素， 需要将该元素位置设置为栈底元素并下沉
			swap(&mut elem, &mut self.0[index]);
			self.down(index);
		}
		store_index(0, &mut elem.1);
		elem
	}

	#[inline]
	fn try_delete(&mut self, index: usize) -> Option<T>{
		let arr = &mut self.0;
		let len = arr.len();
		if index >= len {
			return None;
		}
		Some(self.delete(index, len).0)
	}

	//上朔， 使用时应该保证index不会溢出
	fn up(&mut self, mut cur: usize) -> Arc<AtomicUsize>{
		let arr = &mut self.0;
		if cur > 1{
			let mut parent = (cur - 1) >> 1;
			while arr[cur].0.cmp(&arr[parent].0) == self.1{
				store_index(cur + 1, &arr[parent].1);
				arr.swap(cur, parent);
				if parent == 0{
					break;
				}
				// 往上迭代
				cur = parent;
				parent = (cur - 1) >> 1;
			}
			store_index(cur+ 1, &mut arr[cur].1);
		}
		arr[cur].1.clone()
	}

	/**
	 * 下沉
	 * Panics if index is out of bounds.
	 */
	fn down(&mut self, index: usize) {
		let arr = &mut self.0;
		let mut cur = index;
		let mut left = (cur << 1) + 1;
		let mut right = left + 1;
		let len = arr.len();

		while left < len {
			// 选择左右孩子的最小值作为比较
			let mut child = left;
			if right < len && arr[right].0.cmp(&arr[left].0) == self.1 {
				child = right;
			}
			match arr[cur].0.cmp(&arr[child].0) == self.1 {
				true => break,
				false => {
					store_index(cur+ 1, &arr[child].1);
					arr.swap(cur, child);

					// 往下迭代
					cur = child;
					left = (cur << 1) + 1;
					right = left + 1;
				}
			}
		}
		store_index(cur+ 1, &mut arr[cur].1);
	}
}

//store index in AtomicUsize, The last two bytes represent the type, and 0 means the heap. 
#[inline]
fn store_index(index: usize, dst: &Arc<AtomicUsize>){
	dst.store(index << 2, AOrd::Relaxed);
}

//load index from AtomicUsize, The last two bytes represent the type, and 0 means the heap.

#[inline]
fn load_index(index: &Arc<AtomicUsize>) -> usize{
	index.load(AOrd::Relaxed) >> 2
}

//new index in AtomicUsize, The last two bytes represent the type, and 0 means the heap.
#[inline]
fn new_index(index: usize) -> AtomicUsize{
	AtomicUsize::new(index << 2)
}

#[test]
fn test(){
	let mut min_heap: Heap<u32> = Heap::new(Ordering::Less);
	let index1 = min_heap.push(1);
	assert_eq!(load_index(&index1), 1);
	let index2 = min_heap.push(10);
	assert_eq!(load_index(&index2), 2);
	let index3 = min_heap.push(6);
	assert_eq!(load_index(&index3), 3);
	let index4 = min_heap.push(5);
	assert_eq!(load_index(&index4), 2);
	assert_eq!(load_index(&index2), 4);
	let index5 = min_heap.push(9);
	assert_eq!(load_index(&index5), 5);
	let index6 = min_heap.push(4);
	assert_eq!(load_index(&index6), 3);
	assert_eq!(load_index(&index3), 6);
	let index7 = min_heap.push(4);
	assert_eq!(load_index(&index7), 7);
	let index8 = min_heap.push(4);
	assert_eq!(load_index(&index8), 2);
	assert_eq!(load_index(&index2), 8);
	assert_eq!(load_index(&index4), 4);
	min_heap.push(3);
	min_heap.push(7);
	min_heap.push(100);
	min_heap.push(90);
	min_heap.push(2);
	min_heap.push(15);
	let index15 = min_heap.push(8);
	assert_eq!(load_index(&index15), 15);

	//堆内元素[1, 3, 2, 4, 7, 4, 4, 10, 5, 9, 100, 90, 6, 15, 8]
	let len = min_heap.len();
	assert_eq!(len, 15);

	let mut e = min_heap.remove(&Arc::new(new_index(8))); // [1, 3, 2, 4, 7, 4, 4, 8, 5, 9, 100, 90, 6, 15]
	assert_eq!(e, 10);
	e = min_heap.remove(&Arc::new(new_index(6))); //[1, 3, 2, 4, 7, 6, 4, 8, 5, 9, 100, 90, 15]
	assert_eq!(e, 4);
	e = min_heap.pop().unwrap(); //[2, 3, 4, 4, 7, 6, 15, 8, 5, 9, 100, 90]
	assert_eq!(e, 1); 
	e = min_heap.remove(&Arc::new(new_index(6))); //[2, 3, 4, 4, 7, 90, 15, 8, 5, 9, 100]
	assert_eq!(e, 6); 
	e = min_heap.remove(&Arc::new(new_index(7))); //[2, 3, 4, 4, 7, 90, 100, 8, 5, 9]
	assert_eq!(e, 15);
}

#[cfg(test)]
use time::now_millis;

#[test]
fn test_effic(){
	let mut max_heap: Heap<u32> = Heap::new(Ordering::Greater);
	let now = now_millis();
	for i in 0..100000{
		max_heap.push(i);
	}
	println!("push max_heap time{}",  now_millis() - now);
	
	let mut min_heap: Heap<u32> = Heap::new(Ordering::Less);
	let now = now_millis();
	for i in 0..100000{
		min_heap.push(i);
	}
	println!("push max_heap time{}",  now_millis() - now);
}
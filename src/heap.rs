/**
 * 线程不安全的堆，支持根据索引快速调整位置或删除
 */

use std::sync::atomic::{AtomicIsize, Ordering as AOrd};
use std::sync::Arc;
use std::cmp::{Ord, Ordering};

pub struct Heap<T>(Vec<(T, Arc<AtomicIsize>)>, Ordering);

impl<T: Clone + Ord> Heap<T> {

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
	pub fn push(&mut self, elem: T) -> Arc<AtomicIsize>{
		let len = self.0.len();
		self.0.push((elem, Arc::new(AtomicIsize::new((len as isize) + 1))));
		self.up(len)
	}

	//remove a element by index, Panics if index is out of bounds.
	pub fn remove(&mut self, index: Arc<AtomicIsize>) -> T{
		let i = index.load(AOrd::Relaxed);
		self.delete((i - 1) as usize, self.0.len()).0
	}

	//remove a element by index; returns it, or None if it is not exist;
	pub fn try_remove(&mut self, index: Arc<AtomicIsize>) -> Option<T>{
		let i = index.load(AOrd::Relaxed);
		if i == 0{
			return None;
		}
		self.try_delete((i - 1) as usize)
	}

	//Removes the top element from the pile and returns it, or None if it is empty 
	pub fn pop(&mut self) -> Option<T>{
		self.try_delete(0)
	}

	pub fn get_top(&mut self) -> (T, Arc<AtomicIsize>){
		self.delete(0, self.0.len())
	}

	pub fn get(&self, index: usize) -> Option<&T>{
		match self.0.get(index){
			Some(v) => Some(&v.0),
			None => {None}
		}
	}

	pub fn get_mut(&mut self, index: usize) -> Option<&mut T>{
		match self.0.get_mut(index){
			Some(v) => Some(&mut v.0),
			None => {None}
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
	fn delete(&mut self, index: usize, len: usize) -> (T, Arc<AtomicIsize>){
		let mut elem = self.0.pop().unwrap();
		if index + 1 < len{//如果需要移除的元素不是堆底元素， 需要将该元素位置设置为栈底元素并下沉
			let temp = self.0[index].clone();
			self.0[index] = elem;
			elem = temp;
			self.down(index);
		}
		elem.1.store(0, AOrd::Relaxed);
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
	fn up(&mut self, mut cur: usize) -> Arc<AtomicIsize>{
		let arr = &mut self.0;
		if cur > 1{
			let element = arr[cur].clone();
			let mut parent = (cur - 1) >> 1;
			while element.0.cmp(&arr[parent].0) == self.1{
				let c = arr[parent].clone();
				c.1.store((cur as isize) + 1, AOrd::Relaxed);
				arr[cur] = c;
				if parent == 0{
					break;
				}
				// 往上迭代
				cur = parent;
				parent = (cur - 1) >> 1;
			}
			element.1.store((cur as isize) + 1, AOrd::Relaxed);
			arr[cur] = element;
		}
		arr[cur].1.clone()
	}

	/**
	 * 下沉
	 * Panics if index is out of bounds.
	 */
	fn down(&mut self, index: usize) {
		let arr = &mut self.0;
		let element = arr[index].clone();
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
			
			if element.0.cmp(&arr[child].0) == self.1 {
				break;
			} else {
				let c = arr[child].clone();
				c.1.store((cur as isize) + 1, AOrd::Relaxed);
				arr[cur] = c;
				
				// 往下迭代
				cur = child;
				left = (cur << 1) + 1;
				right = left + 1;
			}
		}
		element.1.store((cur as isize) + 1, AOrd::Relaxed);
		arr[cur] = element;
	}
}


#[test]
fn test(){
	let mut min_heap: Heap<u32> = Heap::new(Ordering::Less);
	let index1 = min_heap.push(1);
	assert_eq!(index1.load(AOrd::Relaxed), 1);
	let index2 = min_heap.push(10);
	assert_eq!(index2.load(AOrd::Relaxed), 2);
	let index3 = min_heap.push(6);
	assert_eq!(index3.load(AOrd::Relaxed), 3);
	let index4 = min_heap.push(5);
	assert_eq!(index4.load(AOrd::Relaxed), 2);
	assert_eq!(index2.load(AOrd::Relaxed), 4);
	let index5 = min_heap.push(9);
	assert_eq!(index5.load(AOrd::Relaxed), 5);
	let index6 = min_heap.push(4);
	assert_eq!(index6.load(AOrd::Relaxed), 3);
	assert_eq!(index3.load(AOrd::Relaxed), 6);
	let index7 = min_heap.push(4);
	assert_eq!(index7.load(AOrd::Relaxed), 7);
	let index8 = min_heap.push(4);
	assert_eq!(index8.load(AOrd::Relaxed), 2);
	assert_eq!(index2.load(AOrd::Relaxed), 8);
	assert_eq!(index4.load(AOrd::Relaxed), 4);
	min_heap.push(3);
	min_heap.push(7);
	min_heap.push(100);
	min_heap.push(90);
	min_heap.push(2);
	min_heap.push(15);
	let index15 = min_heap.push(8);
	assert_eq!(index15.load(AOrd::Relaxed), 15);

	//堆内元素[1, 3, 2, 4, 7, 4, 4, 10, 5, 9, 100, 90, 6, 15, 8]
	let len = min_heap.len();
	assert_eq!(len, 15);

	let mut e = min_heap.remove(Arc::new(AtomicIsize::new(8))); // [1, 3, 2, 4, 7, 4, 4, 8, 5, 9, 100, 90, 6, 15]
	assert_eq!(e, 10);
	e = min_heap.remove(Arc::new(AtomicIsize::new(6))); //[1, 3, 2, 4, 7, 6, 4, 8, 5, 9, 100, 90, 15]
	assert_eq!(e, 4);
	e = min_heap.pop().unwrap(); //[2, 3, 4, 4, 7, 6, 15, 8, 5, 9, 100, 90]
	assert_eq!(e, 1); 
	e = min_heap.remove(Arc::new(AtomicIsize::new(6))); //[2, 3, 4, 4, 7, 90, 15, 8, 5, 9, 100]
	assert_eq!(e, 6); 
	e = min_heap.remove(Arc::new(AtomicIsize::new(7))); //[2, 3, 4, 4, 7, 90, 100, 8, 5, 9]
	assert_eq!(e, 15);
}
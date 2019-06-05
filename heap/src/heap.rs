/**
 * 线程不安全的堆，支持根据索引快速调整位置或删除
 */

use std::cmp::{Ord, Ordering};
use std::mem::transmute_copy;
use std::fmt::{Debug, Formatter, Result as FResult};
use std::ptr::write;

use dyn_uint::{UintFactory};

pub struct Heap<T>(Vec<(T, usize)>, Ordering);

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

    #[inline]
	pub fn len(&self) -> usize{
		self.0.len()
	}

	//清空
	#[inline]
	pub fn clear(&mut self) {
		self.0.clear();
	}

	//插入元素，返回该元素的位置
	pub fn push< F:UintFactory>(&mut self, elem: T, index: usize, index_factor: &mut F ){
		let len = self.0.len();
		index_factor.store(index, len);
		self.0.push((elem, index));
		self.up(len, index_factor);
	}

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T{
        &self.0.get_unchecked(index).0
	}
    
    #[inline]
	pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T{
        &mut self.0.get_unchecked_mut(index).0
	}

	#[inline]
	pub unsafe fn delete< F:UintFactory>(&mut self, index: usize, index_factory: &mut F) -> (T, usize){
		let len = self.0.len();
		let last_elem = self.0.remove(len - 1);
		//如果需要移除的元素不是堆底元素， 需要将该元素位置设置为栈底元素并下沉, 否则直接返回堆底元素
		if index < self.0.len() {
			let cur_elem = transmute_copy(&mut self.0[index]);
			self.down(index, last_elem, index_factory);
			cur_elem
		}else {
			last_elem
		}
	}

	//上朔， 使用时应该保证cur不会溢出
	#[inline]
	fn up< F:UintFactory>(&mut self, mut cur: usize, index_factory: &mut F){
		if cur >= 1{
			let arr = &mut self.0;
			let mut parent = (cur - 1) >> 1;
			if arr[cur].0.cmp(&arr[parent].0) != self.1 { return;}
			let elem: (T, usize) = unsafe{ transmute_copy(&arr[cur])};
			// 往上迭代
			loop {
				index_factory.store(arr[parent].1, cur);
				let src = arr.as_mut_ptr();
				unsafe{src.wrapping_offset(parent as isize).copy_to(src.wrapping_offset(cur as isize), 1)};
				cur = parent;
				if parent == 0 { break; }
				parent = (cur - 1) >> 1;
				if elem.0.cmp(&arr[parent].0) != self.1 { break; }
			}
			unsafe{write(arr.as_mut_ptr().wrapping_offset(cur as isize), elem)};
			index_factory.store(arr[cur].1, cur);
		}
	}

	/**
	 * 下沉
	* Panics if index is out of bounds.
	*/
	#[inline]
	fn down< F:UintFactory>(&mut self, mut cur: usize, elem: (T, usize), index_factory: &mut F) {
		let arr = &mut self.0;
		let mut left = (cur << 1) + 1;
		let mut right = left + 1;
		let len = arr.len();
        
		while left < len {
			// 选择左右孩子的较小值（或较大值， 根据堆的类型而定）作为比较对象
			let (child, child_index) = if right < len && arr[right].0.cmp(&arr[left].0) == self.1 {
				(&mut arr[right], right)
			}else {
				(&mut arr[left], left)
			};
			
			// 往下迭代
			match elem.0.cmp(&child.0) == self.1 {
				true => break,
				false => {
					index_factory.store(child.1, cur);
					let src = arr.as_mut_ptr();
					unsafe{src.wrapping_offset(child_index as isize).copy_to(src.wrapping_offset(cur as isize), 1)};

					cur = child_index;
					left = (cur << 1) + 1;
					right = left + 1;
				}
			}
		}
		unsafe{write(arr.as_mut_ptr().wrapping_offset(cur as isize), elem)};
		index_factory.store(arr[cur].1, cur);
	}
}

impl<T: Debug> Debug for Heap<T> where T: Debug {
	fn fmt(&self, fmt: &mut Formatter) -> FResult {
		write!(fmt,
			"Heap({:?}, {:?})",
			self.0,
			self.1
		)
	}
}
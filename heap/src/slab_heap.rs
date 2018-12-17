use std::cmp::{Ord, Ordering};
use std::fmt::{Debug, Formatter, Result as FResult};

use dyn_uint::{ SlabFactory, UintFactory };
use heap::Heap;

pub struct SlabHeap<T> {
    index_factory: SlabFactory<(), ()>,
    heap: Heap<T>,
}

impl<T: Ord> SlabHeap<T> {

	//构建一个堆, 如果ord为Ordering::Less, 将创建一个小堆, 如果为Ordering::Greater，将创建一个大堆, 不应该使用Ordering::Equal创建一个堆
	pub fn new(ord: Ordering) -> Self{
		SlabHeap{
            index_factory: SlabFactory::new(),
            heap: Heap::new(ord),
        }
	}

	//创建一个堆， 并初始容量
	pub fn with_capacity(capacity: usize, ord: Ordering) -> Self{
        SlabHeap{
            index_factory: SlabFactory::new(),
            heap: Heap::with_capacity(capacity, ord),
        }
	}

	//插入元素，返回该元素的位置
	pub fn push(&mut self, elem: T) -> usize{
        let index = self.index_factory.create(0, (), ());
		self.heap.push(elem, index, &mut self.index_factory);
        index
	}

	//remove a element by index, Panics if index is out of bounds.
	pub fn remove(&mut self, index: usize) -> T{
        let (elem, _) = unsafe { self.heap.delete(self.index_factory.load(index), &mut self.index_factory) };
        self.index_factory.destroy(index);
        elem
	}

	//remove a element by index; returns it, or None if it is not exist;
	pub fn try_remove(&mut self, index: usize) -> Option<T>{
        match self.index_factory.try_load(index) {
            Some(i) => {
                let r = Some(unsafe{ self.heap.delete(i, &mut self.index_factory).0 });
                self.index_factory.destroy(index);
                r
            },
            None => None,
        }
	}

	//Removes the top element from the pile and returns it, or None if it is empty 
	pub fn pop(&mut self) -> Option<T>{
        match self.heap.len() > 0 {
            true => {
                let r = unsafe{ self.heap.delete(0, &mut self.index_factory) };
                self.index_factory.destroy(r.1);
                Some(r.0)
            },
            false => None,
        }
	}

	pub fn get_top(&self) -> Option<&T>{
        match self.heap.len() > 0 {
            true => Some(unsafe{ self.heap.get_unchecked(0) } ),
            false => None,
        }
	}

    pub fn get_top_mut(&mut self) -> Option<&mut T>{
        match self.heap.len() > 0 {
            true => Some(unsafe{ self.heap.get_unchecked_mut(0) } ),
            false => None,
        }
	}

	pub fn get(&self, index: usize) -> Option<&T>{
		match self.index_factory.try_load(index) {
            Some(i) => Some(unsafe{ self.heap.get_unchecked(i) }),
            None => None,
        }
	}

	pub fn get_mut(&mut self, index: usize) -> Option<&mut T>{
		match self.index_factory.try_load(index) {
            Some(i) => Some(unsafe{ self.heap.get_unchecked_mut(i) } ),
            None => None,
        }
	}

    pub fn get_unchecked(&self, index: usize) -> &T{
		unsafe{ self.heap.get_unchecked(self.index_factory.load(index)) }
	}

	pub fn get_unchecked_mut(&mut self, index: usize) -> &mut T{
		unsafe{ self.heap.get_unchecked_mut(self.index_factory.load(index))}
	}

	#[inline]
	pub fn len(&self) -> usize{
		self.heap.len()
	}

	//清空
	#[inline]
	pub fn clear(&mut self) {
		self.heap.clear();
	}
}

impl<T: Debug> Debug for SlabHeap<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "SlabHeap({:?}, {:?})",
               self.index_factory,
               self.heap
        )
    }
}

#[test]
fn test(){
	let mut min_heap: SlabHeap<u32> = SlabHeap::new(Ordering::Less);
    
    assert_eq!([
        min_heap.push(1),
        min_heap.push(10),
        min_heap.push(6),
        min_heap.push(5),
        min_heap.push(9),
        min_heap.push(4),
        min_heap.push(4),
        min_heap.push(4),
        min_heap.push(3),
        min_heap.push(7),
        min_heap.push(100),
        min_heap.push(90),
        min_heap.push(2),
        min_heap.push(15),
        min_heap.push(8)],
        [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]
    );
    println!("{:?}", min_heap);

	let mut e = min_heap.remove(2);
	assert_eq!(e, 10);
	e = min_heap.remove(8); //[1, 3, 2, 4, 7, 6, 4, 8, 5, 9, 100, 90, 15]
	assert_eq!(e, 4);
	e = min_heap.remove(1); //[2, 3, 4, 4, 7, 6, 15, 8, 5, 9, 100, 90]
	assert_eq!(e, 1); 
	e = min_heap.remove(3); //[2, 3, 4, 4, 7, 90, 15, 8, 5, 9, 100]
	assert_eq!(e, 6); 
	e = min_heap.remove(14); //[2, 3, 4, 4, 7, 90, 100, 8, 5, 9]
	assert_eq!(e, 15);

    println!("{:?}", min_heap);
}


#[cfg(test)]
use time::now_millis;

#[test]
fn test_effic(){
	let mut max_heap: SlabHeap<u32> = SlabHeap::new(Ordering::Greater);

	let now = now_millis();
	for i in 0..100000{
		max_heap.push(i);
	}
	println!("push max_heap time{}",  now_millis() - now);
	
	let mut min_heap: SlabHeap<u32> = SlabHeap::new(Ordering::Less);
	let now = now_millis();
	for i in 0..100000{
		min_heap.push(i);
	}
	println!("push max_heap time{}",  now_millis() - now);
}
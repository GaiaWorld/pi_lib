use std::cmp::{Ord, Ordering};
use std::fmt::{Debug, Formatter, Result as FResult};

use dyn_uint::{ UintFactory };
use crate::heap::Heap;

pub struct IndexFactory;

impl UintFactory for IndexFactory {
    #[inline]
    fn load(&self, _index: usize) -> usize{
        0
    }
    #[inline]
    fn try_load(&self, _index: usize) -> Option<usize>{
        None
    }
    #[inline]
	fn store(&mut self, _index: usize, _value: usize){}
    #[inline]
    fn try_store(&mut self, _index: usize, _value: usize) -> bool{
        false
    }
}

pub struct SimpleHeap<T> {
    index_factory: IndexFactory,
    heap: Heap<T>,
}

impl<T: Ord> SimpleHeap<T> {

	//构建一个堆, 如果ord为Ordering::Less, 将创建一个小堆, 如果为Ordering::Greater，将创建一个大堆, 不应该使用Ordering::Equal创建一个堆
	pub fn new(ord: Ordering) -> Self{
		SimpleHeap{
            index_factory: IndexFactory,
            heap: Heap::new(ord),
        }
	}

	//创建一个堆， 并初始容量
	pub fn with_capacity(capacity: usize, ord: Ordering) -> Self{
        SimpleHeap{
            index_factory: IndexFactory,
            heap: Heap::with_capacity(capacity, ord),
        }
	}

	//插入元素，返回该元素的位置
	pub fn push(&mut self, elem: T){
		self.heap.push(elem, 0, &mut self.index_factory);
	}

	//Removes the top element from the pile and returns it, or None if it is empty 
	pub fn pop(&mut self) -> Option<T>{
        match self.heap.len() > 0 {
            true => {
                let r = unsafe{ self.heap.delete(0, &mut self.index_factory) };
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

impl<T: Debug> Debug for SimpleHeap<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "SimpleHeap({:?})",
               self.heap
        )
    }
}

#[test]
fn test(){
	let mut min_heap: SimpleHeap<u32> = SimpleHeap::new(Ordering::Less);

    min_heap.push(1);
    min_heap.push(10);
    min_heap.push(6);
    min_heap.push(5);
    min_heap.push(9);
    min_heap.push(4);
    min_heap.push(4);
    min_heap.push(4);
    min_heap.push(3);
    min_heap.push(7);
    min_heap.push(100);
    min_heap.push(90);
    min_heap.push(2);
    min_heap.push(15);
    min_heap.push(8);

	assert_eq!(min_heap.pop().unwrap(), 1);
    assert_eq!(min_heap.pop().unwrap(), 2);
    assert_eq!(min_heap.pop().unwrap(), 3);
    assert_eq!(min_heap.pop().unwrap(), 4);
    assert_eq!(min_heap.pop().unwrap(), 4);
    assert_eq!(min_heap.pop().unwrap(), 4);
    assert_eq!(min_heap.pop().unwrap(), 5);
    assert_eq!(min_heap.pop().unwrap(), 6);
    assert_eq!(min_heap.pop().unwrap(), 7);
    assert_eq!(min_heap.pop().unwrap(), 8);
    assert_eq!(min_heap.pop().unwrap(), 9);
    assert_eq!(min_heap.pop().unwrap(), 10);
    assert_eq!(min_heap.pop().unwrap(), 15);
    assert_eq!(min_heap.pop().unwrap(), 90);
    assert_eq!(min_heap.pop().unwrap(), 100);
}

#[cfg(test)]
#[derive(PartialEq, PartialOrd, Debug, Eq, Ord)]
struct AA(isize, usize);

#[test]
fn test1(){
	let mut min_heap: SimpleHeap<AA> = SimpleHeap::new(Ordering::Less);

    min_heap.push(AA(3, 2));

    println!("{:?}", min_heap);
    min_heap.push(AA(2, 3));
    println!("{:?}", min_heap);
    min_heap.push(AA(1, 6));
    println!("{:?}", min_heap);
    // min_heap.push(AA(5, 8));
    // min_heap.push(AA(10, 9));
    // min_heap.push(AA(15, 20));

    println!("{:?}, {:?}, {:?}", min_heap.pop(), min_heap.pop(), min_heap.pop());
}
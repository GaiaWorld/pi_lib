use std::cmp::{Ord, Ordering};
use std::fmt::{Debug, Formatter, Result as FResult};

use heap::Heap;
use index_class::IndexClassFactory;
use ver_index::VerIndex;


pub struct SlabHeap<T, I:VerIndex> {
    factory: IndexClassFactory<(), (), I>,
    heap: Heap<T, I::ID>,
}

impl<T: Ord, I:VerIndex+Default> SlabHeap<T, I> {

	//构建一个堆, 如果ord为Ordering::Less, 将创建一个小堆, 如果为Ordering::Greater，将创建一个大堆, 不应该使用Ordering::Equal创建一个堆
	pub fn new(ord: Ordering) -> Self{
		SlabHeap{
            factory: IndexClassFactory::default(),
            heap: Heap::new(ord),
        }
	}

	//创建一个堆， 并初始容量
	pub fn with_capacity(capacity: usize, ord: Ordering) -> Self{
        let mut f = IndexClassFactory::default();
        f.reserve(capacity);
        SlabHeap{
            factory: f,
            heap: Heap::with_capacity(capacity, ord),
        }
	}

	//插入元素，返回该元素的id
	pub fn push(&mut self, elem: T) -> I::ID{
        let id = self.factory.create(0, (), ());
		self.heap.push(elem, id, &mut self.factory);
        id
	}

	//remove a element by index; returns it, or None if it is not exist;
	pub fn remove(&mut self, id: I::ID) -> Option<T>{
        match self.factory.remove (id) {
            Some(i) => Some(unsafe{ self.heap.delete(i.index, &mut self.factory).0 }),
            None => None,
        }
	}

	//Removes the top element from the pile and returns it, or None if it is empty 
	pub fn pop(&mut self) -> Option<T>{
        match self.heap.len() > 0 {
            true => {
                let r = unsafe{ self.heap.delete(0, &mut self.factory) };
                self.factory.remove(r.1);
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

	pub fn get(&self, id: I::ID) -> Option<&T>{
		match self.factory.get(id) {
            Some(i) => Some(unsafe{ self.heap.get_unchecked(i.index) }),
            None => None,
        }
	}

	pub fn get_mut(&mut self, id: I::ID) -> Option<&mut T>{
		match self.factory.get(id) {
            Some(i) => Some(unsafe{ self.heap.get_unchecked_mut(i.index) } ),
            None => None,
        }
	}

    pub fn get_unchecked(&self, id: I::ID) -> &T{
		unsafe{ self.heap.get_unchecked(self.factory.get_unchecked(id).index) }
	}

	pub fn get_unchecked_mut(&mut self, id: I::ID) -> &mut T{
		unsafe{ self.heap.get_unchecked_mut(self.factory.get_unchecked(id).index)}
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

impl<T: Debug, I: VerIndex> Debug for SlabHeap<T, I> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "SlabHeap({:?}, {:?})",
               self.factory,
               self.heap
        )
    }
}

#[test]
fn test(){
    use ver_index::bit::BitIndex;
	let mut min_heap: SlabHeap<u32, BitIndex> = SlabHeap::new(Ordering::Less);
    
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

	let mut e = min_heap.remove(2).unwrap();
	assert_eq!(e, 10);
	e = min_heap.remove(8).unwrap(); //[1, 3, 2, 4, 7, 6, 4, 8, 5, 9, 100, 90, 15]
	assert_eq!(e, 4);
	e = min_heap.remove(1).unwrap(); //[2, 3, 4, 4, 7, 6, 15, 8, 5, 9, 100, 90]
	assert_eq!(e, 1); 
	e = min_heap.remove(3).unwrap(); //[2, 3, 4, 4, 7, 90, 15, 8, 5, 9, 100]
	assert_eq!(e, 6); 
	e = min_heap.remove(14).unwrap(); //[2, 3, 4, 4, 7, 90, 100, 8, 5, 9]
	assert_eq!(e, 15);

    println!("{:?}", min_heap);
}


#[cfg(test)]
use time::now_millisecond;

#[test]
fn test_effic(){
    use ver_index::bit::BitIndex;
	let mut min_heap: SlabHeap<u32, BitIndex> = SlabHeap::new(Ordering::Less);

	let now = now_millisecond();
	for i in 0..100000{
		min_heap.push(i);
	}
	println!("push max_heap time{}",  now_millisecond() - now);
	
	let mut max_heap: SlabHeap<u32, BitIndex> = SlabHeap::new(Ordering::Less);
	let now = now_millisecond();
	for i in 0..100000{
		max_heap.push(i);
	}
	println!("push max_heap time{}",  now_millisecond() - now);
}
//! deque模块通常不单独使用，slab_deque模块deque提供一个Slab实现的索引工厂
//! 当不需要与其它数据结构结合时，你可以使用slab_deque为你提供的双端队列。
//! 如果你不满意使用Slab作为索引工厂，你也可以参照本模块，重新封装deque模块

use std::fmt::{Debug, Formatter, Result as FResult};

use slab::Slab;
use deque::{ Deque, Node, Iter as DIter };

/// 一个用slab作为索引工厂的双端队列
pub struct SlabDeque<T>{
    deque: Deque<T, Slab<Node<T>>>,
    slab: Slab<Node<T>>,
}

impl<T> Default for SlabDeque<T> {
    fn default() -> Self {
        SlabDeque::new()
    }
}

impl<T> SlabDeque<T> {
    pub fn new() -> Self {
        Self {
            deque: Deque::new(),
            slab: Slab::new(),
        }
    }

    /// Append an element to the SlabDeque. return a index
    /// 在尾部压入一个元素，返回元素索引
    #[inline]
    pub fn push_back(&mut self, elem: T) -> usize {
        self.deque.push_back(elem, &mut self.slab)
    }

    /// Prepend an element to the SlabDeque. return a index
    /// 在头部压入一个元素，返回元素索引
    pub fn push_front(&mut self, elem: T) -> usize{
        self.deque.push_front(elem, &mut self.slab)
    }

    /// Removes the first element from the SlabDeque and returns it, or None if it is empty.
    /// 从队列头部弹出一个元素，并返回弹出的头部元素，如果队列中没有元素，则返回None
    pub fn pop_front(&mut self) -> Option<T> {
        self.deque.pop_front(&mut self.slab)
    }

    /// Removes the last element from the SlabDeque and returns it, or None if it is empty.
    /// 从队列尾部弹出一个元素，并返回弹出的尾部元素，如果队列中没有元素，则返回None
    pub fn pop_back(&mut self) -> Option<T> {
        self.deque.pop_back(&mut self.slab)
    }

    /// Removes and returns the element at index from the SlabDeque.
    /// 删除索引对应的元素，并返回该元素；如果没有对应元素，将panic
    pub fn remove(&mut self, index: usize) -> T {
        self.deque.remove(index, &mut self.slab)
    }

    /// Removes and returns the element at index from the SlabDeque.
    /// 尝试删除索引对应的元素，并返回该元素；如果没有对应元素，则返回None
    pub fn try_remove(&mut self, index: usize) -> Option<T> {
        self.deque.try_remove(index, &mut self.slab)
    }

    /// clear SlabDeque
    /// 清空队列
    pub fn clear(&mut self) {
        self.deque.clear(&mut self.slab)
    }

    /// 取到队列元素个数
    pub fn len(&self) -> usize {
        self.slab.len()
    }

    /// 创建队列的迭代器
    pub fn iter(&mut self) -> Iter<T> {
        Iter{
            d_iter: self.deque.iter(&self.slab),
        }
    }
}

impl<T: Debug> Debug for SlabDeque<T> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.debug_struct("SlabDeque")
            .field("slab", &self.slab)
            .field("first", &self.deque)
            .finish()
    }
}

pub struct Iter<'a, T: 'a> {
    d_iter: DIter<'a, T, Slab<Node<T>>>,
}


impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.d_iter.next()
    }
}



#[cfg(test)]
use time::now_millisecond;

#[cfg(test)]
use std::collections::{VecDeque, HashMap};

#[test]
fn test(){
	let mut fast_deque: SlabDeque<u32> = SlabDeque::new();
   
    let i = fast_deque.push_back(1);
    fast_deque.remove(i);
    println!("-----{}", fast_deque.len());

}


#[test]
fn test_effict(){
	let mut fast_deque: SlabDeque<u32> = SlabDeque::new();
    let max = 100000;

    let now = now_millisecond();
    for i in 0..max {
        fast_deque.push_back(i);
    }

    println!("push back time{}",  now_millisecond() - now);

    let now = now_millisecond();
    for _ in 0..max {
        fast_deque.pop_back().unwrap();
        //println!("i---------------------{}", i);
        // let index: usize = ((5- i)/2) as usize;
        // println!("index---------------------{}", index);
        // assert_eq!(fast_SlabDeque.remove(&(index + 1)).unwrap(), index as u32);
        //assert_eq!(fast_SlabDeque.pop_front().unwrap(), i);
    }
    println!("pop_back time{}",  now_millisecond() - now);

    let mut vec_deque = VecDeque::new();
    let now = now_millisecond();
    for i in 0..max {
        vec_deque.push_back(i);
    }
    println!("push vec front time{}",  now_millisecond() - now);

    let now = now_millisecond();
    for _ in 0..max{
        vec_deque.pop_back();
    }
    println!("pop vec front time{}",  now_millisecond() - now);

    let mut map = HashMap::new();
    let now = now_millisecond();
    for i in 0..max {
        map.insert(i, i);
    }
    println!("insert HashMap front time{}",  now_millisecond() - now);

    let now = now_millisecond();
    for i in 0..max {
        assert_eq!(map.remove(&i).unwrap(), i);
    }
    println!("remove HashMap front time{}",  now_millisecond() - now);

}
/// A double-ended queue implemented with a link and map.
///
/// Support for appending to the back and popping from the back.
/// Support for prepending to the front and popping from the front.
/// Supports quick deletion of specified elements
/// 

use std::ptr::read;
use std::ptr::NonNull;

use fnv::FnvHashMap;

pub struct FastDeque<T>{
    first :Option<NonNull<Node<T>>>,
    last :Option<NonNull<Node<T>>>,
    map: FnvHashMap<usize, NonNull<Node<T>>>,
    count: usize,
}

impl<T> FastDeque<T> {
    pub fn new() -> Self {
        Self {
            first: None,
            last: None,
            map: FnvHashMap::default(),
            count: 0,
        }
    }

    /// Append an element to the FastDeque. return a index
    pub fn push_back(&mut self, elem: T) -> usize {
        self.count = to_ring_usize(self.count);
        let ptr = match self.last {
            Some(mut n) => {
                let ptr = Box::into_raw_non_null(Box::new(Node::new(elem, Some(n), None, self.count)));
                unsafe {n.as_mut()}.next = Some(ptr);
                self.last = Some(ptr);
                ptr
            }
            None => {
                let ptr = Box::into_raw_non_null(Box::new(Node::new(elem, None, None, self.count)));
                self.last = Some(ptr);
                self.first = Some(ptr);
                ptr
            }
        };
        self.map.insert(self.count, ptr);
        self.count
    }

    /// Prepend an element to the FastDeque. return a index
    pub fn push_front(&mut self, elem: T) -> usize{
        self.count += to_ring_usize(1);
        let ptr = match self.first {
            Some(mut n) => {
                let ptr = unsafe{ NonNull::new_unchecked(&mut Node::new(elem, None, Some(n), self.count) as *mut Node<T>) };
                unsafe { n.as_mut() }.pre = Some(ptr);
                ptr
            }
            None => {
                let ptr = unsafe{ NonNull::new_unchecked(&mut Node::new(elem, None, None, self.count) as *mut Node<T>) };
                self.last = Some(ptr);
                self.first = Some(ptr);
                ptr
            }
        };
        self.map.insert(self.count, ptr);
        self.count
    }

    /// Removes the first element from the FastDeque and returns it, or None if it is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        let node = match self.first {
            Some(n) => {
                let node = unsafe{ read(n.as_ptr()) };
                self.first = node.next;
                if self.first.is_none(){
                    self.last = None;
                }
                node
            },
            None => {
                return None;
            }
        };
        self.map.remove(&node.index);
        Some(node.elem)
    }

    /// Removes the last element from the FastDeque and returns it, or None if it is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        let node = match self.last {
            Some(n) => {
                let node = unsafe { read(n.as_ptr()) };
                self.last = node.pre;
                if self.last.is_none(){
                    self.last = None;
                }
                node
            },
            None => {
                return None;
            }
        };
        self.map.remove(&node.index);
        Some(node.elem)
    }

    ///Removes and returns the element at index from the FastDeque.
    pub fn remove(&mut self, index: &usize) -> Option<T> {
        match self.map.remove(&index){
            Some(ptr) => {
                let node = unsafe { read(ptr.as_ptr()) };
                match (node.pre, node.next) {
                    (Some(mut pre_ptr), Some(mut next_ptr)) => {
                        //如果该元素既存在上一个元素，也存在下一个元素， 则将上一个元素的下一个元素设置为本元素的下一个元素, 下一个元素的上一个元素设置为本元素的上一个元素
                        unsafe { pre_ptr.as_mut() }.next = Some(next_ptr);
                        unsafe { next_ptr.as_mut() }.pre = Some(pre_ptr);
                    },
                    (Some(mut pre_ptr), None) => {
                        //如果该元素存在上一个元素，不存在下一个元素， 则将上一个元素的下一个元素设置为None, 并设置队列的尾部为该元素的上一个元素
                        unsafe { pre_ptr.as_mut() }.next = None;
                        self.last = Some(pre_ptr);
                    },
                    (None, Some(mut next_ptr)) => {
                        //如果该元素不存在上一个元素，但存在下一个元素， 则将下一个元素的上一个元素设置为None, 并设置队列的头部为该元素的下一个元素
                        unsafe{ next_ptr.as_mut() }.pre = None;
                        self.first = Some(next_ptr);
                    },
                    (None, None) => {
                        //如果该元素既不存在上一个元素，也不存在下一个元素， 则设置队列的头部None， 则设置队列的尾部None
                        self.first = None;
                        self.last = None;
                    },
                }
                Some(node.elem)
            },
            None => None,
        }
    }

    //clear FastDeque
    pub fn clear(&mut self) {
        self.count = 0;
        loop {
            match self.first {
                Some(node) => {
                    self.first = unsafe { read(node.as_ptr()) }.next;
                },
                None => break,
            }
        }
        self.last = None;
    }
}

unsafe impl<T: Send> Send for FastDeque<T> {}

struct Node<T>{
    pub elem: T,
    pub next: Option<NonNull<Node<T>>>,
    pub pre: Option<NonNull<Node<T>>>,
    pub index: usize
}

impl<T> Node<T>{
    fn new(elem: T, pre: Option<NonNull<Node<T>>>, next: Option<NonNull<Node<T>>>, index: usize) -> Node<T>{
        Node{
            elem,
            pre,
            next,
            index,
        }
    }
}

fn to_ring_usize(id: usize) -> usize{
    if id == <usize>::max_value(){
        return 1;
    }else {
        return id + 1;
    }
}

#[cfg(test)]
use time::now_millis;

#[cfg(test)]
use std::collections::{VecDeque, HashMap};


#[test]
fn test(){
	let mut fast_deque: FastDeque<u32> = FastDeque::new();
    let max = 100000;

    let now = now_millis();
    for i in 0..max {
        fast_deque.push_back(i);
    }

    println!("push back time{}",  now_millis() - now);

    let now = now_millis();
    for i in 0..max {
        fast_deque.remove(&((i + 1) as usize)).unwrap();
        // println!("i---------------------{}", i);
        // let index: usize = ((5- i)/2) as usize;
        // println!("index---------------------{}", index);
        // assert_eq!(fast_deque.remove(&(index + 1)).unwrap(), index as u32);
        //assert_eq!(fast_deque.pop_front().unwrap(), i);
    }
    println!("pop front time{}",  now_millis() - now);

    let mut vec_deque = VecDeque::new();
    let now = now_millis();
    for i in 0..max {
        vec_deque.push_back(i);
    }
    println!("push vec front time{}",  now_millis() - now);

    let now = now_millis();
    for i in 0..max {
        let index: usize = ((max- i)/2) as usize;
        vec_deque.remove(index);
        //assert_eq!(vec_deque.remove(index).unwrap(), index as u32);
        //assert_eq!(vec_deque.pop_front().unwrap(), i);
    }
    println!("pop vec front time{}",  now_millis() - now);

    let mut map = HashMap::new();
    let now = now_millis();
    for i in 0..max {
        map.insert(i, i);
    }
    println!("insert HashMap front time{}",  now_millis() - now);

    let now = now_millis();
    for i in 0..max {
        assert_eq!(map.remove(&i).unwrap(), i);
    }
    println!("remove HashMap front time{}",  now_millis() - now);

    let mut map = FnvHashMap::default();
    let now = now_millis();
    for i in 0..max {
        map.insert(i, i);
    }
    println!("insert FnvHashMap front time{}",  now_millis() - now);

    let now = now_millis();
    for i in 0..max {
        assert_eq!(map.remove(&i).unwrap(), i);
    }
    println!("remove FnvHashMap front time{}",  now_millis() - now);

}
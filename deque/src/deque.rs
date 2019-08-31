/// A double-ended queue implemented with a link and map.
///
/// Support for appending to the back and popping from the back.
/// Support for prepending to the front and popping from the front.
/// Supports quick deletion of specified elements
/// 
/// 
use std::fmt::{Debug, Formatter, Result as FResult};
use std::marker::PhantomData;
use std::mem::replace;
use std::iter::Iterator;

use slab::IndexMap;

pub struct Deque<T, C: IndexMap<Node<T>>>{
    first : usize,
    last :usize,
    len: usize,
    mark: PhantomData<(T, C)>,
}

impl<T, C: IndexMap<Node<T>>> Default for Deque<T, C> {
    fn default() -> Self {
        Deque::new()
    }
}

impl<T, C: IndexMap<Node<T>>> Deque<T, C> {
    pub fn new() -> Self {
        Self {
            first: 0,
            last: 0,
            len: 0,
            mark: PhantomData,
        }
    }

    pub fn get_first(&self) -> usize{
        self.first
    }

    pub fn get_last(&self) -> usize{
        self.last
    }

    /// Append an element to the Deque. return a index
    pub fn push_back(&mut self, elem: T, index_map: &mut C) -> usize {
        self.len += 1;
        if self.last == 0 {
            let index = index_map.insert(Node::new(elem, 0, 0));
            self.last = index;
            self.first = index;
            index
        }else {
            let index = index_map.insert(Node::new(elem, self.last, 0));
            unsafe{index_map.get_unchecked_mut(self.last).next = index;}
            self.last = index;
            index
        }
    }

    /// Prepend an element to the Deque. return a index
    pub fn push_front(&mut self, elem: T, index_map: &mut C) -> usize{
        self.len += 1;
        if self.first == 0 {
            let index = index_map.insert(Node::new(elem, 0, 0));
            self.last = index;
            self.first = index;
            index
        }else {
            let index = index_map.insert(Node::new(elem, 0, self.first));
            unsafe{index_map.get_unchecked_mut(self.first).pre = index;}
            self.first = index;
            index
        }
    }

    /// Prepend an element to the Deque. return a index
    pub unsafe fn push_to_back(&mut self, elem: T, index: usize, index_map: &mut C) -> usize{
        self.len += 1;
        let i = index_map.insert(Node::new(elem, index, 0));

        let next = {
            let e = index_map.get_unchecked_mut(index);
            replace(&mut e.next, i)
        };

        index_map.get_unchecked_mut(next).pre = i;
        let e = index_map.get_unchecked_mut(i);
        e.pre = index;
        e.next = next;
        i
    }

    /// Prepend an element to the Deque. return a index
    pub unsafe fn push_to_front(&mut self, elem: T, index: usize, index_map: &mut C) -> usize{
        self.len += 1;
        let i = index_map.insert(Node::new(elem, index, 0));

        let pre = {
            let e = index_map.get_unchecked_mut(index);
            replace(&mut e.pre, i)
        };

        index_map.get_unchecked_mut(pre).next = i;
        let e = index_map.get_unchecked_mut(i);
        e.pre = pre;
        e.next = index;
        i
    }
    /// Removes the first element from the Deque and returns it, or None if it is empty.
    pub unsafe fn pop_front_unchecked(&mut self, index_map: &mut C) -> T {
        self.len -= 1;
        let node = index_map.remove(self.first);
        self.first = node.next;
        if self.first == 0 {
            self.last = 0;
        }
        node.elem
    }
    /// Removes the first element from the Deque and returns it, or None if it is empty.
    pub fn pop_front(&mut self, index_map: &mut C) -> Option<T> {
        if self.first == 0{
            None
        } else {
            self.len -= 1;
            let node = index_map.remove(self.first);
            self.first = node.next;
            if self.first == 0 {
                self.last = 0;
            }
            Some(node.elem)
        }
    }

    /// Removes the last element from the Deque and returns it, or None if it is empty.
    pub fn pop_back(&mut self, index_map: &mut C) -> Option<T> {
        if self.last == 0{
            None
        } else {
            self.len -= 1;
            let node = index_map.remove(self.last);
            self.last = node.pre;
            if self.last == 0 {
                self.first = 0;
            }
            Some(node.elem)
        }
    }

    ///Removes and returns the element at index from the Deque.
    pub fn remove(&mut self, index: usize, index_map: &mut C) -> T {
        let node = index_map.remove(index);
        match (node.pre, node.next) {
            (0, 0) => {
                //如果该元素既不存在上一个元素，也不存在下一个元素， 则设置队列的头部None， 则设置队列的尾部None
                self.first = 0;
                self.last = 0;
            },
            
            (_, 0) => {
                //如果该元素存在上一个元素，不存在下一个元素， 则将上一个元素的下一个元素设置为None, 并设置队列的尾部为该元素的上一个元素
                unsafe{ index_map.get_unchecked_mut(node.pre).next = 0};
                self.last = node.pre;
            },
            (0, _) => {
                //如果该元素不存在上一个元素，但存在下一个元素， 则将下一个元素的上一个元素设置为None, 并设置队列的头部为该元素的下一个元素
                unsafe{ index_map.get_unchecked_mut(node.next).pre = 0};
                self.first = node.next;
            },
            (_, _) => {
                //如果该元素既存在上一个元素，也存在下一个元素， 则将上一个元素的下一个元素设置为本元素的下一个元素, 下一个元素的上一个元素设置为本元素的上一个元素
                unsafe{ index_map.get_unchecked_mut(node.pre).next = node.next};
                unsafe{ index_map.get_unchecked_mut(node.next).pre = node.pre};
            },
            
        }
        self.len -= 1;
        node.elem
    }

    ///Removes and returns the element at index from the Deque.
    pub fn try_remove(&mut self, index: usize, index_map: &mut C) -> Option<T> {
        match index_map.contains(index){
            true => {
                Some(self.remove(index, index_map))
            },
            false => None,
        }
    }

    //clear Deque
    pub fn clear(&mut self, index_map: &mut C) {
        loop {
            if self.first == 0 {
                self.last = 0;
                break;
            }
            let node = index_map.remove(self.first);
            self.first = node.next;
        }
        self.len = 0;
    }

    //clear Deque
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter<'a>(&self, container: &'a C) -> Iter<'a, T, C> {
        Iter{
            next: self.first,
            container: container,
            mark: PhantomData,
        }
    }

}

impl<T, C: IndexMap<Node<T>>> Clone for Deque<T, C>{
    fn clone(&self) -> Deque<T, C>{
        Deque {
            first: self.first,
            last: self.last,
            len: self.len,
            mark: PhantomData
        }
    }
}


pub struct Iter<'a, T: 'a, C: 'a + IndexMap<Node<T>>> {
    next: usize,
    container: &'a C,
    mark: PhantomData<T>
}

impl<'a, T, C: IndexMap<Node<T>>> Iterator for Iter<'a, T, C> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.next == 0 {
            return None;
        }
        
        let node = unsafe{self.container.get_unchecked(self.next)};
        self.next = node.next;
        Some(&node.elem)
    }
}

impl<T, C: IndexMap<Node<T>>> Debug for Deque<T, C> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.debug_struct("Deque")
            .field("first", &self.first)
            .field("last", &self.last)
            .finish()
    }
}

pub struct Node<T>{
    pub elem: T,
    pub next: usize,
    pub pre: usize,
}

impl<T> Node<T>{
    fn new(elem: T, pre: usize, next: usize) -> Node<T>{
        Node{
            elem,
            pre,
            next,
        }
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.debug_struct("Node")
            .field("elem", &self.elem)
            .field("pre", &self.pre)
            .field("next", &self.next)
            .finish()
    }
}
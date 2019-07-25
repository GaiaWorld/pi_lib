/// A double-ended queue implemented with a link and map.
///
/// Support for appending to the back and popping from the back.
/// Support for prepending to the front and popping from the front.
/// Supports quick deletion of specified elements
/// 
/// 
use std::fmt::{Debug, Formatter, Result as FResult};
use std::marker::PhantomData;
use std::iter::Iterator;

use slab::IdAllocater;
use map::Map;

#[derive(Debug)]
pub enum Direction{
    Back,
    Front,
}

pub struct Deque<T, C: Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>, ID: Copy + Debug + PartialEq + Default + Send + Sync>{
    first: ID,
    last: ID,
    len: usize,
    mark: PhantomData<(T, C)>,
}

impl<T, C: Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Default for Deque<T, C, ID> {
    fn default() -> Self {
        Deque::new()
    }
}

impl<T, C: Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Deque<T, C, ID> {
    pub fn new() -> Self {
        Self {
            first: ID::default(),
            last: ID::default(),
            len: 0,
            mark: PhantomData,
        }
    }

    pub fn get_first(&self) -> ID{
        self.first
    }

    pub fn get_last(&self) -> ID{
        self.last
    }
    /// Append an element to the Deque. return a index
    pub fn push_id(&mut self, id: ID, direct: Direction, id_map: &mut C) {
        self.len += 1;
        if self.first == ID::default() {
            self.last = id;
            self.first = id;
            return
        }
        match direct {
            Direction::Back => {
                unsafe{id_map.get_unchecked_mut(&id).prev = self.last;}
                unsafe{id_map.get_unchecked_mut(&self.last).next = id;}
                self.last = id;
            },
            Direction::Front =>{
                unsafe{id_map.get_unchecked_mut(&id).next = self.first;}
                unsafe{id_map.get_unchecked_mut(&self.first).prev = id;}
                self.first = id;
            }
        }
    }
    /// Append an element to the Deque. return a index
    pub fn push(&mut self, elem: T, direct: Direction, id_map: &mut C) -> ID {
        self.len += 1;
        if self.last == ID::default() {
            let id = id_map.alloc(Node::new(elem, ID::default(), ID::default()));
            self.last = id;
            self.first = id;
            return id
        }
        match direct {
            Direction::Back =>{
                let id = id_map.alloc(Node::new(elem, self.last, ID::default()));
                unsafe{id_map.get_unchecked_mut(&self.last).next = id;}
                self.last = id;
                id
            },
            Direction::Front =>{
                let id = id_map.alloc(Node::new(elem, ID::default(), self.first));
                unsafe{id_map.get_unchecked_mut(&self.first).prev = id;}
                self.first = id;
                id
            }
        }
    }
    /// Append an element to the Deque. return a index
    pub fn push_back(&mut self, elem: T, id_map: &mut C) -> ID {
        self.len += 1;
        if self.last == ID::default() {
            let id = id_map.alloc(Node::new(elem, ID::default(), ID::default()));
            self.last = id;
            self.first = id;
            id
        }else {
            let id = id_map.alloc(Node::new(elem, self.last, ID::default()));
            unsafe{id_map.get_unchecked_mut(&self.last).next = id;}
            self.last = id;
            id
        }
    }

    /// Prepend an element to the Deque. return a index
    pub fn push_front(&mut self, elem: T, id_map: &mut C) -> ID{
        self.len += 1;
        if self.first == ID::default() {
            let id = id_map.alloc(Node::new(elem, ID::default(), ID::default()));
            self.last = id;
            self.first = id;
            id
        }else {
            let id = id_map.alloc(Node::new(elem, ID::default(), self.first));
            unsafe{id_map.get_unchecked_mut(&self.first).prev = id;}
            self.first = id;
            id
        }
    }
    /// Removes the first or last element from the Deque and returns it, or None if it is empty.
    pub fn pop(&mut self, direct: Direction, id_map: &mut C) -> Option<T> {
        match direct {
            Direction::Back => self.pop_back(id_map),
            Direction::Front => self.pop_front(id_map),
        }
    }
    /// Removes the first element from the Deque and returns it, or None if it is empty.
    pub fn pop_front(&mut self, id_map: &mut C) -> Option<T> {
        if self.first == ID::default() {
            return None
        }
        let node = match id_map.remove(&self.first) {
            Some(r) => r,
            _ => return None
        };
        self.len -= 1;
        self.first = node.next;
        if self.first == ID::default() {
            self.last = ID::default();
        }
        Some(node.elem)
    }

    /// Removes the last element from the Deque and returns it, or None if it is empty.
    pub fn pop_back(&mut self, id_map: &mut C) -> Option<T> {
        if self.last == ID::default() {
            return None
        }
        let node = match id_map.remove(&self.last) {
            Some(r) => r,
            _ => return None
        };
        self.len -= 1;
        self.last = node.prev;
        if self.last == ID::default() {
            self.first = ID::default();
        }
        Some(node.elem)
    }

    ///Removes and returns the element at index from the Deque.
    pub fn remove(&mut self, id: ID, id_map: &mut C) -> Option<T> {
        let node = match id_map.remove(&id) {
            Some(r) => r,
            _ => return None
        };
        if node.prev == ID::default() {
            if node.next == ID::default() {
                //如果该元素既不存在上一个元素，也不存在下一个元素， 则设置队列的头部None， 则设置队列的尾部None
                self.first = ID::default();
                self.last = ID::default();
            }else{
                //如果该元素不存在上一个元素，但存在下一个元素， 则将下一个元素的上一个元素设置为None, 并设置队列的头部为该元素的下一个元素
                unsafe{ id_map.get_unchecked_mut(&node.next).prev = ID::default()};
                self.first = node.next;
            }
        }else if node.next == ID::default() {
            //如果该元素存在上一个元素，不存在下一个元素， 则将上一个元素的下一个元素设置为None, 并设置队列的尾部为该元素的上一个元素
            unsafe{ id_map.get_unchecked_mut(&node.prev).next = ID::default()};
            self.last = node.prev;
        }else{
            //如果该元素既存在上一个元素，也存在下一个元素， 则将上一个元素的下一个元素设置为本元素的下一个元素, 下一个元素的上一个元素设置为本元素的上一个元素
            unsafe{ id_map.get_unchecked_mut(&node.prev).next = node.next};
            unsafe{ id_map.get_unchecked_mut(&node.next).prev = node.prev};
        }
        self.len -= 1;
        Some(node.elem)
    }

    //clear Deque
    pub fn clear(&mut self, id_map: &mut C) {
        while self.first != ID::default() {
            match id_map.remove(&self.first) {
                Some(node) => self.first = node.next,
                _ => break
            }
        }
        self.first = ID::default();
        self.last = ID::default();
        self.len = 0;
    }

    //clear Deque
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter<'a>(&self, container: &'a C) -> Iter<'a, T, C, ID> {
        Iter{
            next: self.first,
            container: container,
            mark: PhantomData,
        }
    }

}

impl<T, C: Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Clone for Deque<T, C, ID> {
    fn clone(&self) -> Self{
        Deque {
            first: self.first,
            last: self.last,
            len: self.len,
            mark: PhantomData
        }
    }
}


pub struct Iter<'a, T: 'a, C: 'a + Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> {
    next: ID,
    container: &'a C,
    mark: PhantomData<T>
}

impl<'a, T, C: Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>, ID: 'a + Copy + Debug + PartialEq + Default + Send + Sync> Iterator for Iter<'a, T, C, ID> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.next == ID::default() {
            return None;
        }
        let node = unsafe{self.container.get_unchecked(&self.next)};
        self.next = node.next;
        Some(&node.elem)
    }
}

impl<T, C: Map<Key=ID, Val=Node<T, ID>> + IdAllocater<Node<T, ID>, ID=ID>, ID: Copy + Debug + PartialEq + Default + Send + Sync> Debug for Deque<T, C, ID> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.debug_struct("Deque")
            .field("first", &self.first)
            .field("last", &self.last)
            .finish()
    }
}

pub struct Node<T, ID: Copy + Debug + PartialEq + Default + Send + Sync>{
    pub elem: T,
    pub prev: ID,
    pub next: ID,
}

impl<T,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Node<T, ID>{
    pub fn new(elem: T, prev: ID, next: ID) -> Self {
        Node{
            elem,
            prev,
            next,
        }
    }
}

impl<T: Debug,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Debug for Node<T, ID> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.debug_struct("Node")
            .field("elem", &self.elem)
            .field("prev", &self.prev)
            .field("next", &self.next)
            .finish()
    }
}
//! 基于slotmap的双端队列
//! 支持从队列头部添加或弹出
//! 支持从队列尾部添加或弹出
//! 与标准库的双端队列相比，本双端队列还支持根据索引快速从任意位置删除和查询，一些时候，可快速删除的双端队列十分有用（例如pi_lib中的task_pool）
//!
//! 特色： 将双端队列本身的逻辑和索引（删除就需要依赖索引）分离，因此，十分容易和其它需要索引的数据结构共享索引。
//! 关于共享索引的意义，请参考：https://github.com/GaiaWorld/pi_lib/tree/master/dyn_uint
//!
//! 选择:
//! - 当你需要使用双端队列，并且你不需要快速从任意位置删除和查询，标准库中的双端队列是一个不错的选择
//! - 当你的部分功能需要使用从任意位置删除和查询，部分功能不需要时，不太建议你同时依赖标准库与本库的双端队列，毕竟会增减应用程序的尺寸
//! 但如果你不在意，你可以这么做！这种情况下，
//! 建议的做法是，总是使用本库或其它的代替品,本库的双端队列性能仅比标准库略低（删除功能也需要一定成本）

use std::fmt::{Debug, Formatter, Result};
use std::iter::Iterator;


use slotmap::{Key, SlotMap};

pub type Slot<K, T> = SlotMap<K, LinkedNode<K, T>>;

/// 双端队列
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Deque<K: Key> {
    head: K,
    tail: K,
}

impl<K: Key> Default for Deque<K> {
    fn default() -> Self {
        Deque::new()
    }
}

impl<K: Key> Deque<K> {
    pub fn new() -> Self {
        Self {
            head: K::null(),
            tail: K::null(),
        }
    }
    /// Get head key
    pub fn head(&self) -> K {
        self.head
    }
    /// Get tail key
    pub fn tail(&self) -> K {
        self.tail
    }

    /// Append an element to the Deque. return a key
    pub fn push_back<T>(&mut self, el: T, slot: &mut Slot<K, T>) -> K {
        if self.tail.is_null() {
            let key = slot.insert(LinkedNode::new(el, K::null(), K::null()));
            self.head = key;
            self.tail = key;
            key
        } else {
            let key = slot.insert(LinkedNode::new(el, self.tail, K::null()));
            unsafe {
                slot.get_unchecked_mut(self.tail).next = key;
            }
            self.tail = key;
            key
        }
    }

    /// Prepend an element to the Deque. return a key
    pub fn push_front<T>(&mut self, el: T, slot: &mut Slot<K, T>) -> K {
        if self.head.is_null() {
            let key = slot.insert(LinkedNode::new(el, K::null(), K::null()));
            self.head = key;
            self.tail = key;
            key
        } else {
            let key = slot.insert(LinkedNode::new(el, K::null(), self.head));
            unsafe {
                slot.get_unchecked_mut(self.head).prev = key;
            }
            self.head = key;
            key
        }
    }
    /// Append an key to the Deque
    pub fn push_key_back<T>(&mut self, key: K, slot: &mut Slot<K, T>) {
        let node = unsafe { slot.get_unchecked_mut(key) };
        if self.tail.is_null() {
            node.prev = K::null();
            node.next = K::null();
            self.tail = key;
            self.head = key;
        } else {
            node.prev = self.tail;
            node.next = K::null();
            unsafe {
                slot.get_unchecked_mut(self.tail).next = key;
            }
            self.tail = key;
        }
    }
    /// Append an key to the Deque
    pub fn push_key_front<T>(&mut self, key: K, slot: &mut Slot<K, T>) {
        let node = unsafe { slot.get_unchecked_mut(key) };
        if self.head.is_null() {
            node.prev = K::null();
            node.next = K::null();
            self.head = key;
            self.tail = key;
        } else {
            node.prev = K::null();
            node.next = self.head;
            unsafe {
                slot.get_unchecked_mut(self.tail).prev = key;
            }
            self.head = key;
        }
    }
    /// Removes the last element from the Deque and returns it, or None if it is empty.
    pub fn pop_back<T>(&mut self, slot: &mut Slot<K, T>) -> Option<T> {
        if let Some(node) = slot.remove(self.tail) {
            self.tail = node.prev;
            if self.tail.is_null() {
                self.head = K::null();
            } else {
                unsafe { slot.get_unchecked_mut(self.tail).next = K::null() };
            }
            Some(node.el)
        } else {
            None
        }
    }

    /// Removes the first element from the Deque and returns it, or None if it is empty.
    pub fn pop_front<T>(&mut self, slot: &mut Slot<K, T>) -> Option<T> {
        if let Some(node) = slot.remove(self.head) {
            self.head = node.next;
            if self.head.is_null() {
                self.tail = K::null();
            } else {
                unsafe { slot.get_unchecked_mut(self.head).prev = K::null() };
            }
            Some(node.el)
        } else {
            None
        }
    }

    ///Removes and returns the element at key from the Deque.
    pub fn remove<T>(
        &mut self,
        key: K,
        slot: &mut Slot<K, T>,
    ) -> Option<T> {
        if let Some(node) = slot.remove(key) {
            self.repair(node.prev, node.next, slot);
            Some(node.el)
        } else {
            None
        }
    }
    ///repair Deque.
    pub fn repair<T>(
        &mut self,
        prev: K,
        next: K,
        slot: &mut Slot<K, T>,
    ) {
        if prev.is_null() {
            if next.is_null() {
                //如果该元素既不存在上一个元素，也不存在下一个元素， 则设置队列的头部None， 则设置队列的尾部None
                self.head = K::null();
                self.tail = K::null();
            } else {
                //如果该元素不存在上一个元素，但存在下一个元素， 则将下一个元素的上一个元素设置为None, 并设置队列的头部为该元素的下一个元素
                unsafe { slot.get_unchecked_mut(next).prev = K::null() };
                self.head = next;
            }
        } else if next.is_null() {
            //如果该元素存在上一个元素，不存在下一个元素， 则将上一个元素的下一个元素设置为None, 并设置队列的尾部为该元素的上一个元素
            unsafe { slot.get_unchecked_mut(prev).next = K::null() };
            self.tail = prev;
        } else {
            //如果该元素既存在上一个元素，也存在下一个元素， 则将上一个元素的下一个元素设置为本元素的下一个元素, 下一个元素的上一个元素设置为本元素的上一个元素
            unsafe { slot.get_unchecked_mut(prev).next = next };
            unsafe { slot.get_unchecked_mut(next).prev = prev };
        }
    }
    //clear Deque
    pub fn clear<T>(&mut self, slot: &mut Slot<K, T>) {
        while !self.head.is_null() {
            let node = slot.remove(self.head).unwrap();
            self.head = node.next;
        }
        self.tail = K::null();
    }
    pub fn iter<'a, T>(&self, container: &'a SlotMap<K, LinkedNode<K, T>>) -> Iter<'a, K, T> {
        Iter {
            next: self.head,
            container: container,
        }
    }
}

pub struct Iter<'a, K: Key, T: 'a> {
    next: K,
    container: &'a SlotMap<K, LinkedNode<K, T>>,
}

impl<'a, K: Key, T> Iterator for Iter<'a, K, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.next.is_null() {
            return None;
        }
        let node = unsafe { self.container.get_unchecked(self.next) };
        self.next = node.next;
        Some(&node.el)
    }
}

impl<K: Key> Debug for Deque<K> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Deque")
            .field("first", &self.head)
            .field("last", &self.tail)
            .finish()
    }
}

pub struct LinkedNode<K: Key, T> {
    pub el: T,
    prev: K,
    next: K,
}

impl<K: Key, T> LinkedNode<K, T> {
    pub fn new(el: T, prev: K, next: K) -> Self {
        LinkedNode {
            el,
            prev,
            next,
        }
    }
    /// Get prev key
    pub fn prev(&self) -> K {
        self.prev
    }
    /// Get next key
    pub fn next(&self) -> K {
        self.next
    }
}

impl<K: Key, T: Debug> Debug for LinkedNode<K, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Node")
            .field("el", &self.el)
            .field("prev", &self.prev)
            .field("next", &self.next)
            .finish()
    }
}

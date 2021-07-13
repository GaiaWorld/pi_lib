//! 支持删除的堆，当堆内元素移动时，会调用回调函数，根据所在位置维护删除索引


#![warn(type_alias_bounds)]
#![allow(missing_docs)]


extern crate alloc;

extern crate slotmap;
extern crate index_slotmap;
extern crate ext_heap;


use std::{cmp::{Ordering, Ord, PartialOrd}, fmt};

use slotmap::*;
use index_slotmap::*;
use ext_heap::*;

/// 带反向位置索引键的条目
pub struct KeyItem<T> {
    pub el: T,
    key: DefaultKey,
}
impl<T: fmt::Debug> fmt::Debug for KeyItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyItem")
            .field("el", &self.el)
            .field("key", &self.key)
            .finish()
    }
}
impl<T: Clone> Clone for KeyItem<T> {
    fn clone(&self) -> Self {
        KeyItem{
            el: self.el.clone(),
            key: self.key.clone(),
        }
    }
}
// Ord trait所需
impl<T: Ord> Eq for KeyItem<T> {}
// Ord trait所需
impl<T: Ord> PartialEq for KeyItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.el.eq(&other.el)
    }
}
impl<T: Ord> PartialOrd for KeyItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.el.partial_cmp(&other.el)
    }
}
impl<T: Ord> Ord for KeyItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.el.cmp(&other.el)
    }
}

impl<T> KeyItem<T> {
    pub fn new(el: T, key: DefaultKey) -> Self {
        KeyItem{
            el,
            key,
        }
    }
    pub fn get_key(&self) -> &DefaultKey {
        &self.key
    }
}
pub type DeletableHeap<T> = ExtHeap<KeyItem<T>>;

/// 增加可删除堆的操作接口
pub trait HeapAction<T: Ord, I> {
    /// 弹出最优先的元素并维护反向位置索引
    fn pop_index(&mut self, slotmap: &mut IndexSlotMap<I>) -> Option<KeyItem<T>>;
    /// 移除指定位置的元素并维护反向位置索引
    fn remove_index(&mut self, index: usize, slotmap: &mut IndexSlotMap<I>) -> KeyItem<T>;
    /// 修复指定位置的元素并维护反向位置索引
    fn repair_index(&mut self, index: usize, ord: Ordering, slotmap: &mut IndexSlotMap<I>) -> usize;
    /// 放入元素并维护反向位置索引
    fn push_index(&mut self, item: KeyItem<T>, slotmap: &mut IndexSlotMap<I>) -> usize;
}

impl<T: Ord, I> HeapAction<T, I> for ExtHeap<KeyItem<T>> {
    fn pop_index(&mut self, slotmap: &mut IndexSlotMap<I>) -> Option<KeyItem<T>> {
        match self.pop(slotmap, set_index) {
            Some(item) => {
                slotmap.remove(item.key);
                Some(item)
            }
            _ => None
        }
    }

    fn remove_index(&mut self, index: usize, slotmap: &mut IndexSlotMap<I>) -> KeyItem<T> {
        let item = self.remove(index, slotmap, set_index);
        slotmap.remove(item.key);
        item
    }

    fn repair_index(&mut self, index: usize, ord: Ordering, slotmap: &mut IndexSlotMap<I>) -> usize {
        self.repair(index, ord, slotmap, set_index)
    }

    fn push_index(&mut self, item: KeyItem<T>, slotmap: &mut IndexSlotMap<I>) -> usize {
        self.push(item, slotmap, set_index)
    }
}
pub fn set_index<I, T: Ord>(slotmap: &mut IndexSlotMap<I>, arr: &mut [KeyItem<T>], loc: usize) {
    let i = &arr[loc];
    slotmap[i.key].index = loc;
}
pub fn push_item<T:Ord, I>(heap: &mut DeletableHeap<T>, el: T, index_value: I, slotmap: &mut IndexSlotMap<I>) -> DefaultKey {
    let key = slotmap.insert(IndexEntry{index: heap.len(), value: index_value});
    heap.push_index(KeyItem{el, key}, slotmap);
    key
}

#[test]
fn test(){
    use crate::*;
    let mut slot = IndexSlotMap::new();
	let mut heap: DeletableHeap<u32> = DeletableHeap::new();
    let vec = vec![1,10,6,5,9,4,4,4,3,7,100,90,2,15,8];
    //let vec = vec![1,10,6];
    let mut result = Vec::new();
    for i in vec.clone() {
        result.push(push_item(&mut heap, i, (), &mut slot));
    }
    println!("{:?}", heap);
    println!("{:?}", slot);
    let arr = heap.as_slice();
    for i in 0..arr.len() {
        assert_eq!(slot[arr[i].key].index, i);
    }
    let mut sorted = vec.clone();
    sorted.sort();
    sorted.reverse();
    for i in sorted {
        assert_eq!(heap.pop_index(&mut slot).unwrap().el, i);
        let arr = heap.as_slice();
        for j in 0..arr.len() {
            assert_eq!(slot[arr[j].key].index, j);
        }
    }
}

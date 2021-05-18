//! FIFO（first in first out，先进先出）缓存区， 容量满时总是淘汰先进入的数据， 提供最大最小容量和超时处理。
//! 常用的用法就是将不被使用的资源放入FIFO缓存区，如果该资源被使用了，则需要从该缓存区中移除。
//! 算法逻辑：当放入资源后，如果缓存区大小超过最大容量，则把最旧的资源依次移除，直到缓存区大小小于最大容量或最少保留1个资源。
//! 定时整理，依次超时的资源移除，直到达到最小容量。
//! 内部数据结构为一个slab队列，支持快速从中间删除。 一般被res模块使用，资源id依赖res模块的slab分配。

extern crate deque;
extern crate slab;

use std::ops::Deref;

use deque::deque::{Deque, Node};
use slab::Slab;

pub static MIN: usize = 64 * 1024;
pub static MAX: usize = 1024 * 1024;
pub static TIMEOUT: usize = 3 * 60 * 1000;

pub struct Entry<T> {
    value: T,
    pub cost: usize,
    pub timeout: usize,
}

impl<T> Deref for Entry<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

/// FIFO缓存区
#[derive(Clone)]
pub struct LruCache<T> {
    deque: Deque<Entry<T>, Slab<Node<Entry<T>>>>,
    min_capacity: usize,
    max_capacity: usize,
    timeout: usize,
    size: usize,
}

impl<T> Default for LruCache<T> {
    fn default() -> Self {
        LruCache::with_config(MIN, MAX, TIMEOUT)
    }
}

impl<T> LruCache<T> {
    /// 根据配置新建FIFO缓冲区
    pub fn with_config(min_capacity: usize, max_capacity: usize, timeout: usize) -> Self {
        Self {
            deque: Deque::new(),
            min_capacity,
            max_capacity: if max_capacity > min_capacity {
                max_capacity
            } else {
                min_capacity
            },
            timeout,
            size: 0,
        }
    }
    /// 获得配置
    pub fn get_config(&self) -> (usize, usize, usize) {
        (self.min_capacity, self.max_capacity, self.timeout)
    }
    /// 获得最大容量
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// 获得最大容量
    pub fn min_capacity(&self) -> usize {
        self.min_capacity
    }

    /// 获得当前资源大小
    pub fn size(&self) -> usize {
        self.size
    }
    /// 获得当前资源数量
    pub fn len(&self) -> usize {
        self.deque.len()
    }
    /// 设置配置
    pub fn set_config(&mut self, min_capacity: usize, max_capacity: usize, timeout: usize) {
        self.min_capacity = min_capacity;
        self.max_capacity = if max_capacity > self.min_capacity {
            max_capacity
        } else {
            self.min_capacity
        };
        self.timeout = timeout;
    }
    /// 设置最大容量
    pub fn set_max_capacity(&mut self, max_capacity: usize) {
        self.max_capacity = if max_capacity > self.min_capacity {
            max_capacity
        } else {
            self.min_capacity
        };
    }
    /// 添加一个新资源，返回该资源的id。调用后，一般应该调用capacity_collect方法
    pub fn add(
        &mut self,
        value: T,
        cost: usize,
        now: usize,
        slab: &mut Slab<Node<Entry<T>>>,
    ) -> usize {
        self.size += cost;
        self.deque.push_back(
            Entry {
                value,
                cost,
                timeout: now + self.timeout,
            },
            slab,
        )
    }
    /// 移除资源并返回
    pub fn remove(&mut self, id: usize, slab: &mut Slab<Node<Entry<T>>>) -> Option<(T, usize)> {
        match self.deque.try_remove(id, slab) {
            Some(r) => {
                self.size -= r.cost;
                Some((r.value, r.cost))
            }
            _ => None,
        }
    }
    /// 清空
    pub fn clear(&mut self, slab: &mut Slab<Node<Entry<T>>>) {
        self.deque.clear(slab);
        self.size = 0;
    }
    /// 根据容量进行整理。如果缓冲满，根据FIFO原则，返回被移除的资源。需要循环调用，直到返回None
    pub fn capacity_collect(&mut self, slab: &mut Slab<Node<Entry<T>>>) -> Option<(T, usize)> {
        if self.size <= self.max_capacity {
            return None;
        }
        let r = unsafe { self.deque.pop_front_unchecked(slab) };
        self.size -= r.cost;
        Some((r.value, r.cost))
    }
    /// 根据超时进行整理，返回超时的资源。需要循环调用，直到返回None
    pub fn timeout_collect(
        &mut self,
        now: usize,
        slab: &mut Slab<Node<Entry<T>>>,
    ) -> Option<(T, usize)> {
        if self.size <= self.min_capacity {
            return None;
        }
        let id = self.deque.get_first();
        if id == 0 {
            return None;
        }
        if unsafe { slab.get_unchecked(id) }.elem.timeout > now {
            return None;
        }
        let r = unsafe { self.deque.pop_front_unchecked(slab) };
        self.size -= r.cost;
        Some((r.value, r.cost))
    }
}

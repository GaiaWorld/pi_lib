extern crate deque;
extern crate slab;

use deque::deque::{Deque, Node};
use slab::Slab;

pub static MIN: usize = 64 * 1024;
pub static MAX: usize = 1024 * 1024;
pub static TIMEOUT: usize = 3 * 60 * 1000;

pub struct Entry<T> {
    value: T,
    pub cost: usize,
    timeout: usize,
}
/**
 * LRU 最近最少使用 缓冲
 */
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
    /**
     * 根据配置新建LRU缓冲
     */
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
    /**
     * 更改配置
     */
    pub fn modify_config(&mut self, min_capacity: usize, max_capacity: usize, timeout: usize) {
        self.min_capacity = min_capacity;
        self.timeout = timeout;
        self.max_capacity = if max_capacity > min_capacity {
            max_capacity
        } else {
            min_capacity
        };
    }
    /**
     * 获得配置
     */
    pub fn get_config(&self) -> (usize, usize, usize) {
        (self.min_capacity, self.max_capacity, self.timeout)
    }
    /**
     * 获得最大容量
     */
    pub fn get_max_capacity(&self) -> usize {
        self.max_capacity
    }
    /**
     * 获得当前大小
     */
    pub fn size(&self) -> usize {
        self.size
    }
    /**
     * 获得配置
     */
    pub fn len(&self) -> usize {
        self.deque.len()
    }
    /**
     * 配置
     */
    pub fn set_config(&mut self, min_capacity: usize, max_capacity: usize, timeout: usize) {
        self.min_capacity = min_capacity;
        self.max_capacity = if max_capacity > self.min_capacity {
            max_capacity
        } else {
            self.min_capacity
        };
        self.timeout = timeout;
    }
    /**
     * 设置最大容量
     */
    pub fn set_max_capacity(&mut self, max_capacity: usize) {
        self.max_capacity = if max_capacity > self.min_capacity {
            max_capacity
        } else {
            self.min_capacity
        };
    }
    /**
     * 添加一个新元素，返回该元素的id
     * 注：如果缓冲满，根据LRU原则，移除旧元素并返回
     */
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
    /**
     * 移除元素并返回
     */
    pub fn remove(&mut self, id: usize, slab: &mut Slab<Node<Entry<T>>>) -> Option<(T, usize)> {
        match self.deque.try_remove(id, slab) {
            Some(r) => {
                self.size -= r.cost;
                Some((r.value, r.cost))
            }
            _ => None,
        }
    }
    /**
     * 清空原有的缓冲
     */
    pub fn clear(&mut self, slab: &mut Slab<Node<Entry<T>>>) {
        self.deque.clear(slab);
        self.size = 0;
    }
    /**
     * 根据容量进行整理
     */
    pub fn capacity_collect(&mut self, slab: &mut Slab<Node<Entry<T>>>) -> Option<(T, usize)> {
        if self.size <= self.max_capacity {
            return None;
        }
        let r = unsafe { self.deque.pop_front_unchecked(slab) };
        self.size -= r.cost;
        Some((r.value, r.cost))
    }
    /**
     * 根据超时进行整理
     */
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

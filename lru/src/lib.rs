extern crate deque;

use deque::slab_deque::{SlabDeque};

/**
 * LRU 最近最少使用 缓冲
 */

pub struct LruCache<T> {
    capacity: usize,  // 缓冲的容量
    values: SlabDeque<T>,
}

impl<T> LruCache<T> {

    /** 
     * 根据容量新建LRU缓冲
     */
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            values: SlabDeque::new(),
        }
    }
    
    /** 
     * 清空原有的缓冲
     */
    pub fn reset(&mut self) {
        self.values.clear();
    }

    /** 
     * 允许迭代放到缓冲的元素
     */
    pub fn iter(&mut self) -> impl Iterator<Item=&T> {
        self.values.iter()
    }

    /** 
     * 添加一个新元素，返回该元素的id
     * 注：如果缓冲满，根据LRU原则，移除旧元素并返回
     */
    pub fn add(&mut self, value: T) -> (usize, Option<T>) {
        let old = if self.capacity >= self.values.len() {
            self.values.pop_back()
        } else {
            None
        };
        
        let id = self.values.push_front(value);
        (id, old)
    }

    /** 
     * 尝试命中元素，并返回引用
     * 注：内部实现中，如果命中了元素，必须将该元素移动到队列的头部
     */
    pub fn get(&mut self, _id: usize) -> Option<&T> {
        None
    }

    /** 
     * 尝试命中元素，并返回可写引用
     */
    pub fn get_mut(&mut self, _id: usize) -> Option<&mut T> {
        None
    }

    /** 
     * 移除元素并返回
     */
    pub fn remove(&mut self, id: usize) -> T {
        self.values.remove(id)
    }

    /** 
     * 移除元素并返回
     */
    pub fn try_remove(&mut self, id: usize) -> Option<T> {
        self.values.try_remove(id)
    }
}
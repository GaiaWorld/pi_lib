//! 资源表， 为一类资源提供资源管理.
//! 资源如果真正被使用，则用引用计数来管理。
//! 如果不被使用了，则放入Cache中， 根据最大最小缓存和超时时间来决定释放。
//! 在Cache清理中，先将在最小容量外的所有超时的资源释放， 然后再将超出容量的最早资源释放。
//! TODO 以后Rc或Arc支持设置内存分配器， 可以使用自定义的内存分配器， 这样在资源的引用计数减为0时， 将其放入到特定位置， 可以避免现在通过遍历资源表来发现未引用的资源。

use std::collections::hash_map::{Entry, Iter};
use std::hash::Hash;

use any::{impl_downcast_box, BoxAny};
use hash::XHashMap;
use share::Share;
use slot_deque::{Deque, Slot};
use slotmap::{new_key_type, Key};

/// 资源，放入资源表的资源必须实现该trait
pub trait Res {
    /// 关联键的类型
    type Key: Hash + Eq + Clone + std::fmt::Debug;
    /// 资源的关联键
    fn key(&self) -> Self::Key;
    /// 资源的大小
    fn size(&self) -> usize;
    /// 所在的分组
    fn group(&self) -> usize {
        0
    }
}

// 定义缓冲键类型
new_key_type! {
    pub struct CacheKey;
}

/// 缓存资源队列表
#[derive(Clone, Debug, Default)]
pub(crate) struct ResCache {
    /// 当前未使用的缓存资源队列
    pub deque: Deque<CacheKey>,
    /// 缓存表内全部资源的内存大小
    pub size: usize,
    // 缓存表内全部资源的数量
    pub len: usize,
    /// 最小容量
    pub min_capacity: usize,
    /// 最大容量
    pub max_capacity: usize,
    /// 当前容量
    pub cur_capacity: usize,
    /// 缓存超时时间
    pub timeout: usize,
    /// 名字
    pub name: Share<String>,
}
impl ResCache {
    /// 配置最小最大容量和超时时间
    pub fn config(
        &mut self,
        min_capacity: usize,
        max_capacity: usize,
        timeout: usize,
        name: Share<String>,
    ) {
        self.min_capacity = min_capacity;
        self.max_capacity = max_capacity;
        self.cur_capacity = max_capacity;
        self.timeout = timeout;
        self.name = name;
    }
}

/// 资源整理接口
pub(crate) trait ResCollect: BoxAny {
    /// 获得资源的内存占用
    fn size(&self) -> usize;
    /// 获得资源缓存表的数组
    fn caches(&mut self) -> &mut [ResCache];
    /// 回收清除，将无人使用的资源放入到Cache
    fn garbage(&mut self, now: usize);
    /// 超时整理方法， 清理最小容量外的超时资源
    fn timeout_collect(&mut self, now: usize);
    /// 超量整理方法， 按照先进先出的原则，清理超出容量的资源
    fn capacity_collect(&mut self);
}
impl_downcast_box!(ResCollect);

/// 资源表
#[derive(Debug)]
pub struct ResMap<T: Res> {
    /// 资源表
    map: XHashMap<<T as Res>::Key, (Share<T>, CacheKey)>,
    /// 缓存资源表
    slot: Slot<CacheKey, (Share<T>, usize)>,
    /// 分组的缓存资源队列表
    caches: [ResCache; 4],
    /// 资源表内全部资源的内存大小
    size: usize,
}
impl<T: Res> Default for ResMap<T> {
    fn default() -> Self {
        ResMap {
            map: Default::default(),
            slot: Default::default(),
            caches: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            size: 0,
        }
    }
}
impl<T: Res> ResMap<T> {
    /// 返回资源表的资源引用
    pub fn map_iter(&self) -> Iter<<T as Res>::Key, (Share<T>, CacheKey)> {
        self.map.iter()
    }
    /// 获得指定键的资源
    #[inline]
    pub fn get(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        match self.map.get_mut(key) {
            Some(r) => {
                if !r.1.is_null() {
                    let c = &mut self.caches[r.0.group()];
                    c.deque.remove(r.1, &mut self.slot);
                    c.size -= r.0.size();
                    c.len -= 1;
                    r.1 = CacheKey::null();
                }
                Some(r.0.clone())
            }
            _ => None,
        }
    }
    /// 放入资源
    #[inline]
    pub fn insert(&mut self, res: Share<T>) {
        self.size += res.size();
        match self.map.entry(res.key()) {
            Entry::Occupied(mut e) => {
                let r = e.get_mut();
                self.size -= r.0.size();
                r.0 = res;
                if !r.1.is_null() {
                    let c = &mut self.caches[r.0.group()];
                    c.deque.remove(r.1, &mut self.slot);
                    c.size -= r.0.size();
                    c.len -= 1;
                    r.1 = CacheKey::null();
                }
            }
            Entry::Vacant(e) => {
                e.insert((res, CacheKey::null()));
            }
        }
    }
    /// 移除一个指定键的资源
    #[inline]
    pub fn remove(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        if let Some(r) = self.map.remove(key) {
            self.size -= r.0.size();
            if !r.1.is_null() {
                let c = &mut self.caches[r.0.group()];
                c.deque.remove(r.1, &mut self.slot);
                c.size -= r.0.size();
                c.len -= 1;
            }
            return Some(r.0);
        }
        None
    }
}

impl<T: Res + 'static> ResCollect for ResMap<T> {
    /// 计算资源的内存占用
    fn size(&self) -> usize {
        self.size
    }
    /// 计算资源的内存占用
    fn caches(&mut self) -> &mut [ResCache] {
        self.caches.as_mut_slice()
    }
    /// 回收清除，将无人使用的资源放入到Cache
    fn garbage(&mut self, now: usize) {
        // 将无人使用的资源放入到Cache
        for v in self.map.values_mut() {
            if Share::<T>::strong_count(&v.0) > 1 {
                continue;
            }
            let c = &mut self.caches[v.0.group()];
            c.size += v.0.size();
            c.len += 1;
            c.deque.push_back((v.0.clone(), now), &mut self.slot);
        }
    }
    /// 超时整理方法， 清理最小容量外的超时资源
    fn timeout_collect(&mut self, now: usize) {
        for c in &mut self.caches {
            while c.size > c.min_capacity {
                // 清理过时的资源
                let node = unsafe { self.slot.get_unchecked(c.deque.head()) };
                if node.el.1 + c.timeout < now {
                    break;
                }
                let key = node.el.0.key();
                let size = node.el.0.size();
                c.deque.pop_front(&mut self.slot);
                self.map.remove(&key);
                self.size -= size;
                c.size -= size;
                c.len -= 1;
            }
        }
    }
    // 超量整理方法， 按照先进先出的原则，清理超出容量的资源
    fn capacity_collect(&mut self) {
        for c in &mut self.caches {
            while c.size > c.cur_capacity {
                let (res, _) = c.deque.pop_front(&mut self.slot).unwrap();
                self.map.remove(&res.key());
                self.size -= res.size();
                c.size -= res.size();
                c.len -= 1;
            }
        }
    }
}

//! 资源表， 为一类资源提供资源管理.
//! 资源如果真正被使用，则用引用计数来管理。
//! 如果不被使用了，则放入FifoCache中， 根据最大最小缓存和超时时间来决定释放。

use std::hash::Hash;

use any::RcAny;
use deque::deque::Node;
use hash::XHashMap;
use lru::{Entry, LruCache};
use share::{Share, ShareWeak};
use slab::Slab;

/// 资源，放入资源表的资源必须实现该trait
pub trait Res {
    /// 关联键的类型
    type Key: Hash + Eq + Clone + std::fmt::Debug;
}

/// 资源整理接口
pub trait ResCollect: RcAny {
    /// 计算资源的内存占用
    fn mem_size(&self) -> usize;
    /// 计算资源的内存占用
    fn set_max_capacity(&mut self, max_capacity: usize);
    /// 整理方法， 将无人使用的资源放入到LruCache， 清理超时的资源
    fn collect(&mut self, now: usize) -> StateInfo;
    /// 整理容量，删除超出最大容量的资源
    fn capacity_collect(&mut self);
}
impl_downcast_rc!(ResCollect);

///资源表的状态信息
#[derive(Debug)]
pub enum StateInfo {
    /// 不可用，没有放资源
    None,
    /// 数值为当前最小最大容量。 容量已满， 指大小大于最大容量减平均资源大小的2倍
    Full(usize, usize),
    /// 数值为当前最小最大容量。容量合适， 指大小小于最大容量减平均资源大小的2倍，大于最大容量减平均资源大小的4倍
    Ok(usize, usize),
    /// 数值为当前最小最大容量和可以释放出来的大小。空闲状态。等于最大容量减平均资源大小的4倍
    Free(usize, usize, usize),
}

/// 资源表
pub struct ResMap<T: Res + 'static> {
    map: XHashMap<<T as Res>::Key, ResEntry<T>>,
    array: Vec<(KeyRes<T>, usize, usize)>,
    slab: Slab<Node<Entry<KeyRes<T>>>>,
    pub cache: LruCache<KeyRes<T>>,
    // 调试使用，稳定后去除
    _name: String,
}
impl<T: Res + 'static> Default for ResMap<T> {
    fn default() -> Self {
        ResMap {
            map: XHashMap::default(),
            array: Vec::new(),
            slab: Slab::default(),
            cache: LruCache::default(),
            _name: "".to_string(),
        }
    }
}
impl<T: Res + 'static> ResMap<T> {
    /// 返回所有资源的引用（lru 和 正在使用得）
    pub fn all_res(
        &self,
    ) -> (
        &Vec<(KeyRes<T>, usize, usize)>,
        &Slab<Node<Entry<KeyRes<T>>>>,
    ) {
        (&self.array, &self.slab)
    }
    /// 用指定的名称，最大最小容量，超时时间来创建一个资源表
    pub fn with_config(name: String, min_capacity: usize, max_capacity: usize, timeout: usize) -> Self {
        ResMap {
            map: XHashMap::default(),
            array: Vec::new(),
            slab: Slab::default(),
            cache: LruCache::with_config(min_capacity, max_capacity, timeout),
            _name: name,
        }
    }
    /// 修改最大最小容量和超时时间
    pub fn modify_config(&mut self, min_capacity: usize, max_capacity: usize, timeout: usize) -> (usize, usize, usize) {
        let old = self.cache.get_config();
        self.cache.set_config(min_capacity, max_capacity, timeout);
        old
    }
    /// 获得指定键的资源
    #[inline]
    pub fn get(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        match self.map.get_mut(key) {
            Some(r) => {
                if r.id > 0 {
                    // 将lru中缓存的数据放回到array中
                    let e = self.cache.remove(r.id, &mut self.slab).unwrap();
                    self.array.push((e.0, e.1, r.group_i));
                    r.id = 0;
                }
                Some(r.res.clone())
            }
            None => None,
        }
    }
    /// 创建资源，用指定的键，内容，内存大小，所在分组
    #[inline]
    pub fn create(&mut self, key: T::Key, res: T, cost: usize, group_i: usize) -> Share<T> {
        // println!(
        //     "create res================, cost:{}, resName:{}, group_i: {}, key:{:?}",
        //     cost, &self.name, group_i, key
        // );
        let res = Share::new(res);
        self.map.insert(
            key.clone(),
            ResEntry {
                res: res.clone(),
                group_i,
                id: 0,
            },
        );
        self.array.push((
            KeyRes {
                key,
                res: Share::downgrade(&res),
            },
            cost,
            group_i,
        ));
        res
    }
    /// 移除一个指定键的资源
    #[inline]
    pub fn remove(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        // println!("remove res================, key:{:?}", key);
        match self.map.remove(key) {
            Some(r) => Some(r.res),
            None => None,
        }
    }
}

impl<T: Res + 'static> ResCollect for ResMap<T> {
    fn mem_size(&self) -> usize {
        let mut r = 0;
        r += self.map.capacity()
            * (std::mem::size_of::<<T as Res>::Key>() + std::mem::size_of::<ResEntry<T>>());

        r += self.array.capacity() * std::mem::size_of::<(KeyRes<T>, usize, usize)>();
        r += self.slab.mem_size();
        r
    }

    // 设置指定lru的最大容量
    #[inline]
    fn set_max_capacity(&mut self, max_capacity: usize) {
        self.cache.set_max_capacity(max_capacity);
    }

    // 整理方法， 将无人使用的资源放入到LruCache， 清理超时的资源
    fn collect(&mut self, now: usize) -> StateInfo {
        // 将无人使用的资源放入到LruCache
        let mut i = 0;
        while i < self.array.len() {
            let j = self.array.len() - i - 1;
            i += 1;
            let el = unsafe { self.array.get_unchecked(j) };
            if el.0.res.strong_count() > 1 {
                continue;
            }
            let el = self.array.swap_remove(j);
            // if el.0.res.strong_count() == 0 {
            //     println!("strong_count=============={}", 0);
            //     continue;
            // }
            let k = el.0.key.clone();
            match self.map.get_mut(&k) {
                Some(r) => {
                    r.id = self.cache.add(el.0, el.1, now, &mut self.slab);
                }
                _ => (),
            }
        }

        let mut state_info = StateInfo::None;
        // let mut sizeqq = Vec::<[usize; 5]>::new();
        let c = &mut self.cache;
        // 清理过时的资源
        loop {
            match c.timeout_collect(now, &mut self.slab) {
                Some((r, _)) => self.map.remove(&r.key),
                _ => break,
            };
        }
        let len = c.len();
        if len != 0 {
            let (min, max, _) = c.get_config();
            let size2 = c.size() + c.size() / len;
            // sizeqq.push([min, max, c.size(), len, size2]);
            if c.size() + size2 >= max {
                state_info = StateInfo::Full(min, max);
            } else if c.size() + size2 + size2 >= max {
                state_info = StateInfo::Ok(min, max);
            } else if c.size() + size2 + size2 > min {
                state_info = StateInfo::Free(min, max, c.size() + size2 + size2);
            } else {
                state_info = StateInfo::Free(min, max, min);
            }
        }
        state_info
        // println!(
        //     "map collect==========name: {}, carr:{:?}, sizeqq: {:?}",
        //     self._name, carr, sizeqq
        // );
    }

    fn capacity_collect(&mut self) {
        loop {
            match self.cache.capacity_collect(&mut self.slab) {
                Some((r, _)) => self.map.remove(&r.key),
                _ => break,
            };
        }
    }
}

pub struct KeyRes<T: Res + 'static> {
    key: T::Key,
    res: ShareWeak<T>,
}
pub struct ResEntry<T: Res + 'static> {
    res: Share<T>,
    group_i: usize,
    id: usize,
}

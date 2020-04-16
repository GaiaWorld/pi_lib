// 资源表

use std::hash::Hash;

use any::RcAny;
use deque::deque::Node;
use hash::XHashMap;
use lru::{Entry, LruCache};
use share::{Share, ShareWeak};
use slab::Slab;

pub trait Res {
    type Key: Hash + Eq + Clone + std::fmt::Debug;
}

pub trait ResCollect: RcAny {
    fn mem_size(&self) -> usize;

    fn set_max_capacity(&mut self, index: usize, max_capacity: usize);
    // 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
    fn collect(&mut self, now: usize) -> [StateInfo; 3];

    // 整理容量，删除超出最大容量的资源
    fn capacity_collect(&mut self);
}
impl_downcast_rc!(ResCollect);

#[derive(Debug)]
pub enum StateInfo {
    None,                      // 不可用，没有放资源
    Full(usize, usize), // 数值为当前最小最大容量。 容量已满， 指大小大于最大容量减平均资源大小的2倍
    Ok(usize, usize), // 数值为当前最小最大容量。容量合适， 指大小小于最大容量减平均资源大小的2倍，大于最大容量减平均资源大小的4倍
    Free(usize, usize, usize), // 数值为当前最小最大容量和可以释放出来的大小。空闲状态。等于最大容量减平均资源大小的4倍
}

//资源表
pub struct ResMap<T: Res + 'static> {
    map: XHashMap<<T as Res>::Key, ResEntry<T>>,
    array: Vec<(KeyRes<T>, usize, usize)>,
    slab: Slab<Node<Entry<KeyRes<T>>>>,
    pub caches: [LruCache<KeyRes<T>>; 3],
    // 调试使用，稳定后去除
    _name: String,
}
impl<T: Res + 'static> Default for ResMap<T> {
    fn default() -> Self {
        ResMap {
            map: XHashMap::default(),
            array: Vec::new(),
            slab: Slab::default(),
            caches: [
                LruCache::default(),
                LruCache::default(),
                LruCache::default(),
            ],
            _name: "".to_string(),
        }
    }
}
impl<T: Res + 'static> ResMap<T> {
    // 所有资源（lru 和 正在使用得）
    pub fn all_res(
        &self,
    ) -> (
        &Vec<(KeyRes<T>, usize, usize)>,
        &Slab<Node<Entry<KeyRes<T>>>>,
    ) {
        (&self.array, &self.slab)
    }

    pub fn with_config(configs: &[usize; 9], name: String) -> Self {
        ResMap {
            map: XHashMap::default(),
            array: Vec::new(),
            slab: Slab::default(),
            caches: [
                LruCache::with_config(configs[0], configs[1], configs[2]),
                LruCache::with_config(configs[3], configs[4], configs[5]),
                LruCache::with_config(configs[6], configs[7], configs[8]),
            ],
            _name: name,
        }
    }
    pub fn modify_config(&mut self, configs: &[usize; 9]) -> [(usize, usize, usize); 3] {
        let old = [
            self.caches[0].get_config(),
            self.caches[1].get_config(),
            self.caches[2].get_config(),
        ];
        self.caches[0].modify_config(configs[0], configs[1], configs[2]);
        self.caches[0].modify_config(configs[3], configs[4], configs[5]);
        self.caches[0].modify_config(configs[6], configs[7], configs[8]);
        old
    }
    // 获得指定键的资源
    #[inline]
    pub fn get(&mut self, key: &<T as Res>::Key) -> Option<Share<T>> {
        match self.map.get_mut(key) {
            Some(r) => {
                if r.id > 0 {
                    // 将lru中缓存的数据放回到array中
                    let e = self.caches[r.rtype].remove(r.id, &mut self.slab).unwrap();
                    self.array.push((e.0, e.1, r.rtype));
                    r.id = 0;
                }
                Some(r.res.clone())
            }
            None => None,
        }
    }
    // 创建资源
    #[inline]
    pub fn create(&mut self, key: T::Key, res: T, cost: usize, rtype: usize) -> Share<T> {
        if rtype >= self.caches.len() {
            panic!("invalid rtype: {}", rtype)
        }
        // println!(
        //     "create res================, cost:{}, resName:{}, rtype: {}, key:{:?}",
        //     cost, &self.name, rtype, key
        // );
        let res = Share::new(res);
        self.map.insert(
            key.clone(),
            ResEntry {
                res: res.clone(),
                rtype,
                id: 0,
            },
        );
        self.array.push((
            KeyRes {
                key,
                res: Share::downgrade(&res),
            },
            cost,
            rtype,
        ));
        res
    }

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
    fn set_max_capacity(&mut self, index: usize, max_capacity: usize) {
        self.caches[index].set_max_capacity(max_capacity);
    }

    // 整理方法， 将无人使用的资源放入到LruCache， 清理过时的资源
    fn collect(&mut self, now: usize) -> [StateInfo; 3] {
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
            let id = {
                let c = &mut self.caches[el.2];
                let id = c.add(el.0, el.1, now, &mut self.slab);
                // if (self._name == "TextureRes") {
                //     println!("add_texture=============={}", c.len());
                // }
                // loop {
                //     match c.capacity_collect(&mut self.slab) {
                //         Some((r, _)) => self.map.remove(&r.key),
                //         _ => break,
                //     };
                // }
                id
            };
            match self.map.get_mut(&k) {
                Some(r) => {
                    r.id = id;
                }
                _ => (),
            }
        }

        let mut carr = [StateInfo::None, StateInfo::None, StateInfo::None];
        // let mut sizeqq = Vec::<[usize; 5]>::new();
        i = 0;
        // 清理过时的资源
        for c in self.caches.iter_mut() {
            loop {
                match c.timeout_collect(now, &mut self.slab) {
                    Some((r, _)) => self.map.remove(&r.key),
                    _ => break,
                };
            }
            let len = c.len();
            if len == 0 {
                continue;
            }
            let (min, max, _) = c.get_config();
            let size2 = c.size() + c.size() / len;
            // sizeqq.push([min, max, c.size(), len, size2]);
            if c.size() + size2 >= max {
                carr[i] = StateInfo::Full(min, max);
            } else if c.size() + size2 + size2 >= max {
                carr[i] = StateInfo::Ok(min, max);
            } else if c.size() + size2 + size2 > min {
                carr[i] = StateInfo::Free(min, max, c.size() + size2 + size2);
            } else {
                carr[i] = StateInfo::Free(min, max, min);
            }
            i += 1;
        }
        // println!(
        //     "map collect==========name: {}, carr:{:?}, sizeqq: {:?}",
        //     self._name, carr, sizeqq
        // );
        carr
    }

    fn capacity_collect(&mut self) {
        for c in self.caches.iter_mut() {
            loop {
                match c.capacity_collect(&mut self.slab) {
                    Some((r, _)) => self.map.remove(&r.key),
                    _ => break,
                };
            }
        }
    }
}

pub struct KeyRes<T: Res + 'static> {
    key: T::Key,
    res: ShareWeak<T>,
}
pub struct ResEntry<T: Res + 'static> {
    res: Share<T>,
    rtype: usize,
    id: usize,
}

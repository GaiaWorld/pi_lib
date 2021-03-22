//! # 用于性能采集的计数器
//!

use std::time;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use lazy_static;
use fnv::FnvHashMap;
use parking_lot::RwLock;
use crossbeam_queue::ArrayQueue;

use atom::Atom;

/*
* 最小动态计数器容量
*/
const MIN_DYNAMIC_COUNTER_CAPACITY: usize = 10;

/*
* 默认动态计数器容量
*/
const DEFAULT_DYNAMIC_COUNTER_CAPACITY: usize = 1000000;

/*
* 最小静态计数器容量
*/
const MIN_STATIC_COUNTER_CAPACITY: usize = 1;

/*
* 默认静态计数器容量
*/
const DEFAULT_STATIC_COUNTER_CAPACITY: usize = 1000;

///
/// 全局并发性能采集
///
lazy_static! {
    pub static ref GLOBAL_PREF_COLLECT: PrefCollect = PrefCollect::new(DEFAULT_DYNAMIC_COUNTER_CAPACITY, DEFAULT_STATIC_COUNTER_CAPACITY);
}

///
/// 检查指定计数器名称与计数器id是否匹配
///
pub fn check_counter(name: &str, cid: u64) -> bool {
    Atom::from(name).get_hash() as u64 == cid 
}

///
/// 并发性能计数器
///
#[derive(Debug, Clone)]
pub struct PrefCounter(Arc<AtomicUsize>);

unsafe impl Send for PrefCounter {}
unsafe impl Sync for PrefCounter {}

impl PrefCounter {
    /// 获取
    pub fn get(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    /// 重置
    pub fn set(&self, count: usize) {
        self.0.store(count, Ordering::SeqCst);
    }

    /// 计数
    pub fn sum(&self, count: usize) {
        self.0.fetch_add(count, Ordering::Relaxed);
    }
}

///
/// 并发性能计时器
///
type StartTime = time::Instant;

///
/// 性能定时器
///
#[derive(Debug, Clone)]
pub struct PrefTimer(Arc<AtomicUsize>);

unsafe impl Send for PrefTimer {}
unsafe impl Sync for PrefTimer {}

impl PrefTimer {
    /// 获取
    pub fn get(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }

    /// 开始计时
    pub fn start(&self) -> StartTime {
        StartTime::now()
    }

    /// 计时
    pub fn timing(&self, start: StartTime) {
        self.0.fetch_add((StartTime::now() - start).as_micros() as usize, Ordering::Relaxed);
    }
}

///
/// 动态计数器队列迭代器
///
pub struct DynamicIterator {
    inner: Arc<InnerCollect>,
    cache: Vec<(u64, Arc<AtomicUsize>)>,
}

impl Drop for DynamicIterator {
    fn drop(&mut self) {
        for _ in 0..self.cache.len() {
            //返还缓存的动态计数器
            self.inner.dynamic_collect.push(self.cache.remove(0));
        }
    }
}

impl Iterator for DynamicIterator {
    type Item = (u64, Arc<AtomicUsize>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(counter) = self.inner.dynamic_collect.pop() {
            if Arc::strong_count(&(counter.1)) == 2 {
                //外部没有使用当前动态计数器，则从动态计数器表中移出指定动态计数器，并直接返回
                self.inner.dynamic_table.write().remove(&(counter.0));
                return Some(counter);
            }

            //外部还在使用当前动态计数器，则返回复制
            let r = Some((counter.0, counter.1.clone()));
            self.cache.push(counter);
            return r;
        }

        None
    }
}

///
/// 静态计数器队列迭代器
///
pub struct StaticIterator {
    inner: Arc<InnerCollect>,
    cache: Vec<(u64, Arc<AtomicUsize>)>,
}

impl Drop for StaticIterator {
    fn drop(&mut self) {
        for _ in 0..self.cache.len() {
            //返还缓存的静态计数器
            self.inner.static_collect.push(self.cache.remove(0));
        }
    }
}

impl Iterator for StaticIterator {
    type Item = (u64, Arc<AtomicUsize>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(counter) = self.inner.static_collect.pop() {
            let r = Some((counter.0, counter.1.clone()));
            self.cache.push(counter);
            return r;
        }

        None
    }
}

///
/// 并发性能采集
///
#[derive(Clone)]
pub struct PrefCollect(Arc<InnerCollect>);

unsafe impl Send for PrefCollect {}
unsafe impl Sync for PrefCollect {}

struct InnerCollect {
    dynamic_table:      RwLock<FnvHashMap<u64, Arc<AtomicUsize>>>,  //动态计数器表，用于随机查询指定的动态计数器
    dynamic_collect:    ArrayQueue<(u64, Arc<AtomicUsize>)>,        //动态计数器队列，运行时可以动态增删计数器
    static_init:        AtomicBool,                                 //静态计数器队列初始化标记
    static_collect:     ArrayQueue<(u64, Arc<AtomicUsize>)>,        //静态计数器队列，初始化后无法增删计数器
}

impl PrefCollect {
    /// 构建并发性能采集
    pub fn new(dynamic_capacity: usize, static_capacity: usize) -> Self {
        if dynamic_capacity < MIN_DYNAMIC_COUNTER_CAPACITY {
            panic!("invalid dynamic capacity");
        }
        if static_capacity < MIN_STATIC_COUNTER_CAPACITY {
            panic!("invalid static capacity");
        }

        PrefCollect(Arc::new(InnerCollect {
            dynamic_table: RwLock::new(FnvHashMap::default()),
            dynamic_collect: ArrayQueue::new(dynamic_capacity),
            static_init: AtomicBool::new(false),
            static_collect: ArrayQueue::new(static_capacity),
        }))
    }

    /// 动态计数器队列是否已满
    pub fn dynamic_is_full(&self) -> bool {
        self.0.dynamic_collect.is_full()
    }

    /// 动态计数器数量
    pub fn dynamic_size(&self) -> usize {
        self.0.dynamic_collect.len()
    }

    /// 构建指定性能指标和初始值的动态计数器
    pub fn new_dynamic_counter(&self, target: Atom, init: usize) -> Option<PrefCounter> {
        if self.0.dynamic_collect.is_full() {
            //动态计数器队列已满
            return None;
        }

        let cid = target.get_hash();
        if let Some(counter) = self.0.dynamic_table.read().get(&(cid as u64)) {
            //存在指定的动态计数器
            return Some(PrefCounter(counter.clone()));
        }

        //不存在指定的动态计数器
        let counter = Arc::new(AtomicUsize::new(init));
        self.0.dynamic_collect.push((cid as u64, counter.clone()));
        self.0.dynamic_table.write().insert(cid as u64, counter.clone());
        Some(PrefCounter(counter))
    }

    /// 构建指定性能指标和初始值的动态计时器
    pub fn new_dynamic_timer(&self, target: Atom, init: usize) -> Option<PrefTimer> {
        if self.0.dynamic_collect.is_full() {
            //动态计数器队列已满
            return None;
        }

        let cid = target.get_hash();
        if let Some(counter) = self.0.dynamic_table.read().get(&(cid as u64)) {
            //存在指定的动态计时器
            return Some(PrefTimer(counter.clone()));
        }

        //不存在指定的动态计时器
        let counter = Arc::new(AtomicUsize::new(init));
        self.0.dynamic_collect.push((cid as u64, counter.clone()));
        self.0.dynamic_table.write().insert(cid as u64, counter.clone());
        Some(PrefTimer(counter))
    }

    /// 获取动态计数器的迭代器
    pub fn dynamic_iter(&self) -> DynamicIterator {
        DynamicIterator {
            inner: self.0.clone(),
            cache: Vec::with_capacity(self.0.dynamic_collect.len()),
        }
    }

    /// 静态计数器队列是否已满
    pub fn static_is_full(&self) -> bool {
        self.0.static_collect.is_full()
    }

    /// 静态计数器数量
    pub fn static_size(&self) -> usize {
        self.0.static_collect.len()
    }

    /// 构建指定性能指标和初始值的静态计数器
    pub fn new_static_counter(&self, target: Atom, init: usize) -> Option<PrefCounter> {
        if self.0.static_init.load(Ordering::SeqCst) {
            //初始化已完成，则忽略
            return None;
        }

        let cid = target.get_hash();
        let counter = Arc::new(AtomicUsize::new(init));
        self.0.static_collect.push((cid as u64, counter.clone()));
        Some(PrefCounter(counter))
    }

    /// 构建指定性能指标和初始值的静态计时器
    pub fn new_static_timer(&self, target: Atom, init: usize) -> Option<PrefTimer> {
        if self.0.static_init.load(Ordering::SeqCst) {
            //初始化已完成，则忽略
            return None;
        }

        let cid = target.get_hash();
        let counter = Arc::new(AtomicUsize::new(init));
        self.0.static_collect.push((cid as u64, counter.clone()));
        Some(PrefTimer(counter))
    }

    /// 初始化静态计数器队列完成，返回静态计数器队列长度
    pub fn static_init_ok(&self) -> usize {
        self.0.static_init.compare_and_swap(false, true, Ordering::SeqCst);
        self.0.static_collect.len()
    }

    /// 获取静态计数器的迭代器
    pub fn static_iter(&self) -> StaticIterator {
        StaticIterator {
            inner: self.0.clone(),
            cache: Vec::with_capacity(self.0.static_collect.len()),
        }
    }
}
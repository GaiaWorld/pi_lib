//! 任务池
//! 可以向任务池中插入不同优先级的任务，任务池提供弹出功能，任务池大概率会弹出优先级高的任务。
//! 任务池支持的任务可以大致分为两类：
//!     1. 队列任务：插入队列任务需要先创建队列，插入到同一个队列的任务，会按顺序弹出。即便一个优先级很高的任务，
//!        如果它所在的队列头部还存在任务，也需要等待这些任务弹出后才能被弹出。尽管队列任务的优先级在本队列中并不生效，
//!        但是可以提高整个队列的优先级。如果向一个队列插入一个优先级很高的任务，接下来，弹出该队列头部的任务的概率会变高。
//!     2. 单例任务：在任务池中，如果不是队列任务，那一定是一个单例任务。
//!        单列任务与队列任务的区别是，单例任务不需要排队，单例任务的优先级越高，弹出的概率越大。
//! 尽管任务池中的任务仅分为两类（队列任务，单例任务），但每类任务又可以分为可删除的任务、和不可删除的任务。
//! 一些任务在弹出前，如果不会被取消，推荐插入不可删任务。不可删除的任务在任务池内部使用了更高效的数据结构。
//! 
//! 除此以外，任务池还可以插入一个延时的任务，该任务先被缓存在定时器中，超时后，才能有机会被弹出
//!

#![feature(proc_macro_hygiene)]
extern crate rand;

extern crate flame;
#[allow(unused_imports)]
#[macro_use]
extern crate flamer;

extern crate wtree;
extern crate timer;
extern crate dyn_uint;
extern crate deque;
extern crate slab;

pub mod enums;
mod static_pool;
mod dyn_pool  ;

use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::{Arc, Mutex};
use std::marker::Send;
use std::fmt;
use std::ptr::NonNull;

use rand::prelude::*;
use rand::{Rng};
use rand::rngs::SmallRng;

use timer::{Timer, Runer};
use dyn_uint::{SlabFactory, UintFactory, ClassFactory};

use enums:: {QueueType, IndexType, Direction, Task, FreeSign};

/// 任务池
///
/// # Examples
/// ```
/// let task_pool: TaskPool<usize> = TaskPool::new(Timer::new(10), Arc::new(|_ty, _n| {}));//创建任务池实例（任务池需要一个定时器，来缓存定时任务）
/// struct Task(pub String);
///
/// let asyncIndex1 = task_pool.push_dyn_async(Task(String::from("可删除的单例任务1")), 5); // 在任务池中插入一个可删除的单例任务
/// task_pool.push_static_async(Task(String::from("不可删除的单例任务1")), 7); // 在任务池中插入一个不可删除的单例任务
///
/// let dyn_queue = task_pool.create_dyn_queue(3); // 创建一个任务队列，该队列中的任务可删除；该队列中的任务，默认优先级为3
/// let static_queue = task_pool.create_dyn_queue(4); // 创建一个任务队列，该队列中的任务不可删除；该队列中的任务，默认优先级为4
///
/// let syncIndex1 = task_pool.push_dyn_back(Task(String::from("可删除的队列任务1")), dyn_queue); // 在队列尾部插入一个可删除的队列任务
/// task_pool.push_static_back(Task(String::from("不可删除的队列任务1")), static_queue);// 在队列尾部插入一个不可删除的队列任务
///
/// assert!(task_pool.remove_async(asyncIndex1).0 == Task(String::from("可删除的单例任务1"))) ; // 移除单例任务
/// assert!(task_pool.remove_sync(dyn_queue, syncIndex1) == Task(String::from("可删除的队列任务1")));// 移除队列任务
///
/// println!("弹出任务：{:?}",  task_pool.pop().0); // 弹出任务
/// println!("弹出任务：{:?}",  task_pool.pop().0); // 弹出任务
pub struct TaskPool<T: Debug + 'static>{
    // 不可删除的队列任务
    static_sync_pool: Arc<(AtomicUsize, Mutex<static_pool::SyncPool<T>>)>,
    // static_lock_queues: Arc<Mutex<Slab<WeightQueue<T>>>>,

    // 可删除的队列任务
    sync_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>)>)>,
    //lock_queues: Arc<Mutex<Slab<WeightQueueD<T>>>>,

    // 不可删除的单例任务
    static_async_pool: Arc<(AtomicUsize, Mutex<static_pool::AsyncPool<T>>)>,

    /// 可删除的单例任务
    async_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>)>)>,

    // 延时任务
    delay_queue: Timer<DelayTask<T>>,

    // 
    handler: Arc<dyn Fn(QueueType, usize)>,
    // 当前任务总数
    count: AtomicUsize,
    // 
    rng: Arc<Mutex<SmallRng>>,
}

impl<T: Debug + 'static> TaskPool<T> {
    /// 创建任务池实例
    pub fn new(timer: Timer<DelayTask<T>>, handler: Arc<dyn Fn(QueueType, usize)>) -> Self {
        // let timer = Timer::new(10);
        // timer.run();
        TaskPool {
            static_sync_pool: Arc::new((AtomicUsize::new(0), Mutex::new(static_pool::SyncPool::new()))),

            //static_lock_queues: Arc::new(Mutex::new(Slab::new())),

            sync_pool: Arc::new((AtomicUsize::new(0), Mutex::new((dyn_pool  ::SyncPool::new(), SlabFactory::new())))),
            //lock_queues: Arc::new(Mutex::new(Slab::new())),

            static_async_pool: Arc::new((AtomicUsize::new(0), Mutex::new(static_pool::AsyncPool::new()))),
            async_pool: Arc::new((AtomicUsize::new(0), Mutex::new((dyn_pool  ::AsyncPool::new(), SlabFactory::new())))),

            //index_factory: SlabFactory::new(),
            delay_queue: timer,
            count: AtomicUsize::new(0),
            handler,
            rng: Arc::new(Mutex::new(SmallRng::from_entropy())),
        }
    }

    /// 
    pub fn set_count(&self, count: usize) {
        self.count.store(count, AOrd::Relaxed);
    }

    // 创建可删除任务的任务队列，返回队列id
    pub fn create_dyn_queue(&self, weight: usize) -> isize {
        to_queue_id(self.sync_pool.1.lock().unwrap().0.create_queue(weight))
    }

    // 创建不可删除任务的任务队列，返回队列id
    pub fn create_static_queue(&self, weight: usize) -> isize {
        to_static_queue_id(self.static_sync_pool.1.lock().unwrap().create_queue(weight))
    }

    // 删除一个任务队列，如果删除成功，返回true， 否则返回false
    pub fn delete_queue(&self, id: isize) -> bool {
        if is_queue(id) {
            self.sync_pool.1.lock().unwrap().0.try_remove_queue(from_queue_id(id));
            true
        } else if is_static_queue(id) {
            self.static_sync_pool.1.lock().unwrap().try_remove_queue(from_static_queue_id(id));
            true
        } else {
            false
        }
    }

    /// 指定一个队列，在该队列尾部插入一个可删除的任务，返回该任务在任务池中的索引。
    /// 如果指定的队列不是一个可删除任务的队列，将panic。
    pub fn push_dyn_back(&self, task: T, queue_id: isize) -> isize {
        let (id, opt) = {
            let mut sync_pool = self.sync_pool.1.lock().unwrap();
            let id = sync_pool.1.create(0, IndexType::Sync, ());
            let index = sync_pool.0.push_back(task, from_queue_id(queue_id), id);
            self.sync_pool.0.store(sync_pool.0.get_weight(), AOrd::Relaxed);
            sync_pool.1.store(id, index);
//            println!("!!!!!!push dyn sync push back, weight:{}, len: {}", sync_pool.0.get_weight(), sync_pool.0.queue_len());
            if sync_pool.0.is_locked(queue_id as usize) {
                (id, None)
            } else {
                (id, Some(sync_pool.0.queue_len()))
            }
        };
//        println!("!!!!!!push dyn sync queue start");
        if let Some(queue_len) = opt {
            self.notify(QueueType::DynSync, queue_len);
        }
//        println!("!!!!!!push dyn sync queue finish");
        to_sync_id(id)
    }

    /// 指定一个队列，在该队列头部插入一个可删除的任务，返回该任务在任务池中的索引。
    /// 如果指定的队列不是一个可删除任务的队列，将panic。
    pub fn push_dyn_front(&self, task: T, queue_id: isize) -> isize {
        let (id, opt) = {
            let mut sync_pool = self.sync_pool.1.lock().unwrap();
            let id = sync_pool.1.create(0, IndexType::Sync, ());
            let index = sync_pool.0.push_front(task, from_queue_id(queue_id), id);
            self.sync_pool.0.store(sync_pool.0.get_weight(), AOrd::Relaxed);
            sync_pool.1.store(id, index);
            if sync_pool.0.is_locked(queue_id as usize) {
                (id, None)
            } else {
                (id, Some(sync_pool.0.queue_len()))
            }
        };
        if let Some(queue_len) = opt {
            self.notify(QueueType::DynSync, queue_len);
        }
        to_sync_id(id)
    }

    /// 指定一个队列，在该队列尾部插入一个不可删除的任务。
    /// 如果指定的队列不是一个不可删除任务的队列，将panic。
    pub fn push_static_back(&self, task: T, queue_id: isize) {
        let opt = {
            let id = from_static_queue_id(queue_id);
            let mut sync_pool = self.static_sync_pool.1.lock().unwrap();
            sync_pool.push_back(task, id);
            self.static_sync_pool.0.store(sync_pool.get_weight(), AOrd::Relaxed);
            if sync_pool.is_locked(id) {
                None
            } else {
                Some(sync_pool.queue_len())
            }
        };
//        println!("!!!!!!push static sync queue start");
        if let Some(len) = opt {
            self.notify(QueueType::StaticSync, len);
        }
//        println!("!!!!!!push static sync queue start");
    }

    /// 指定一个队列，在该队列头部插入一个不可删除的任务。
    /// 如果指定的队列不是一个不可删除任务的队列，将panic。
    pub fn push_static_front(&self, task: T, queue_id: isize) {
        let opt = {
            let id = from_static_queue_id(queue_id);
            let mut sync_pool = self.static_sync_pool.1.lock().unwrap();
            sync_pool.push_front(task, id);
            self.static_sync_pool.0.store(sync_pool.get_weight(), AOrd::Relaxed);
            if sync_pool.is_locked(id) {
                None
            } else {
                Some(sync_pool.queue_len())
            }
        };
        if let Some(len) = opt {
            self.notify(QueueType::StaticSync, len);
        }
    }

    /// 插入一个可删除的单例任务，并指定任务优先级，返回该任务在任务池中的索引。
    pub fn push_dyn_async(&self, task: T, priority: usize) -> isize {
        let (index, len) = {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, indexs): &mut (wtree::wtree::WeightTree<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let index = indexs.create(0, IndexType::Async, ());
            pool.push(task, priority, index, indexs);
            self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
            (index, pool.len())
        };
//        println!("!!!!!!push dyn async queue start");
        self.notify(QueueType::DynAsync, len);
//        println!("!!!!!!push dyn async queue finish");
        to_async_id(index)
    }

    /// 插入一个不可删除的单例任务，并指定任务优先级。
    pub fn push_static_async(&self, task: T, priority: usize) {
        let len = {
            let mut lock = self.static_async_pool.1.lock().unwrap();
            lock.push(task, priority);
            self.static_async_pool.0.store(lock.amount(), AOrd::Relaxed);
            lock.len()
        };
//        println!("!!!!!!push static async queue start");
        self.notify(QueueType::StaticAsync, len);
//        println!("!!!!!!push dyn async queue finish");
    }

    /// 插入一个可删除的延时队列任务， 返回该任务在任务池中的索引
    /// task: 任务实例，queue_id：队列id， direc：表明时插入到头部还是尾部， ms：延时时长
    pub fn push_sync_delay(&self, task: T, queue_id: isize, direc: Direction, ms: u32) -> isize{
        let index = self.sync_pool.1.lock().unwrap().1.create(0, IndexType::Delay, ());

        let task = DelayTask::Sync {
            queue_id: from_queue_id(queue_id),
            direc: direc,
            index: index,
            sync_pool: self.sync_pool.clone(),
            task: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(task))) },
            handler: self.handler.clone(),
        };
        let index1 = self.delay_queue.set_timeout(task, ms);
        self.sync_pool.1.lock().unwrap().1.store(index, index1);
        to_sync_id(index)
    }

    /// 插入一个可删除的延时单例任务， 返回该任务在任务池中的索引
    pub fn push_async_delay(&self, task: T, priority: usize, ms: u32) -> isize{
        let index = self.sync_pool.1.lock().unwrap().1.create(0, IndexType::Delay, ());
        let task = DelayTask::Async {
            priority: priority,
            index: index,
            async_pool: self.async_pool.clone(),
            task: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(task))) },
            handler: self.handler.clone(),
        };
        let index1 = self.delay_queue.set_timeout(task, ms);
        self.sync_pool.1.lock().unwrap().1.store(index, index1);
        to_async_id(index)
    }

    /// 弹出一个任务，如果任务存在，返回Some(Task), 否则返回None
    /// 如果该任务是一个队列任务，也不对该任务所在的队列加锁
    /// 使用该方法，无法严格保证队列任务的执行顺序。外部应该确保，弹出任务执行完毕后，再弹出下一个任务
    pub fn pop_unlock(&self) -> Option<T>{
        let (async_w, sync_w, static_async_w, static_sync_w, r, mut w) = self.weight_rng();

        //println!("w--------------{:?}", (async_w, sync_w, static_async_w, static_sync_w, r, w));
        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let w = pool.get_weight();
            if w != 0 {
                let r = pool.pop_front(r%w).unwrap();
                self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                indexs.destroy(r.1);
                return Some(r.0);
            }
        } else {
            w = w - sync_w;
        }

        if w < async_w {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let w = pool.amount();
            if w != 0 {
                let r = unsafe{pool.pop(r%w, indexs)};
                self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
                indexs.destroy(r.2);
                return Some(r.0);
            }
        } else {
            w = w - async_w;
        }

        if w < static_async_w {
            let mut pool = self.static_async_pool.1.lock().unwrap();
            let w = pool.amount();
            if w != 0 {
                let r = Some(pool.pop(r%w).0);
                self.static_async_pool.0.store(pool.amount(), AOrd::Relaxed);
                return r;
            }
        } else {
            w = w - static_async_w;
        }

        if w < static_sync_w {
            let mut pool = self.static_sync_pool.1.lock().unwrap();
            let w = pool.get_weight();
            if w != 0 {
                let r = pool.pop_front(r%w);
                self.static_sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                return r;
            }
        }
        None
    }

    /// 弹出一个任务，如果任务存在，返回任务, 否则返回None
    /// 如果该任务是一个队列任务，会对该任务所在的队列加锁，此后，该队列的任务无法弹出，
    /// 直到外部调用free_queue方法解锁该队列，该队列的任务在后续的弹出过程中才有机会被弹出
    pub fn pop(&self) -> Option<Task<T>>{
        let (async_w, sync_w, static_async_w, static_sync_w, r, mut w) = self.weight_rng();
//        println!("w--------------{:?}", (async_w, sync_w, static_async_w, static_sync_w, r, w));
        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let weight = pool.get_weight();
            if weight != 0 {
                let r = pool.pop_front_with_lock(r%weight);
                if let Some(elem) = r.0 {
                    self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                    indexs.destroy(elem.1);
//                println!("w---dyn_sync_pop");
                    return Some(Task::Sync(elem.0, to_sync_id(r.1) ));
                } else {
                    w = w - sync_w;
                }
            }
        } else {
            w = w - sync_w;
        }

        if w < async_w {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let weight = pool.amount();
            if weight != 0 {
                let r = unsafe{pool.pop(r%weight, indexs)};
                self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
                indexs.destroy(r.2);
//                println!("w---dyn_async_pop");
                return Some(Task::Async(r.0));
            }
        } else {
            w = w - async_w;
        }


        if w < static_async_w {
            let mut pool = self.static_async_pool.1.lock().unwrap();
            let weight = pool.amount();
            if weight != 0 {
                let r = Some(Task::Async(pool.pop(r%weight).0));
                self.static_async_pool.0.store(pool.amount(), AOrd::Relaxed);
//                println!("w---static_async_pop");
                return r;
            }
        } else {
            w = w - static_async_w;
        }

        if w < static_sync_w {
            let mut pool = self.static_sync_pool.1.lock().unwrap();
            let weight = pool.get_weight();
            if weight != 0 {
                let r = pool.pop_front_with_lock(r%weight);
                if let Some(elem) = r.0 {
                    self.static_sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
//                println!("w---static_sync_pop");
                    return Some(Task::Sync(elem, to_static_queue_id(r.1)));
                }
            }
        }
//        println!("w---empty_pop");
        None
    }

    /// 只弹出可删除的队列任务和所有单例任务
    pub fn pop_inner(&self) -> Option<Task<T>>{
        let (async_w, sync_w, static_async_w, r, mut w) = self.weight_rng_inner();

        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let weight = pool.get_weight();
            if weight != 0 {
                let r = pool.pop_front_with_lock(r%weight);
                if let Some(elem) = r.0 {
                    self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                    indexs.destroy(elem.1);
                    return Some(Task::Sync(elem.0, to_sync_id(r.1) ));
                } else {
                    w = w - sync_w;
                }
            }
        } else {
            w = w - sync_w;
        }

        if w < async_w {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let weight = pool.amount();
            if weight != 0 {
                let r = unsafe{pool.pop(r%weight, indexs)};
                self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
                indexs.destroy(r.2);
                return Some(Task::Async(r.0));
            }
        } else {
            w = w - async_w;
        }


        if w < static_async_w {
            let mut pool = self.static_async_pool.1.lock().unwrap();
            let weight = pool.amount();
            if weight != 0 {
                let r = Some(Task::Async(pool.pop(r%weight).0));
                self.static_async_pool.0.store(pool.amount(), AOrd::Relaxed);
                return r;
            }
        }

        None
    }

    /// 移除一个队列任务， 返回被移除的任务，如果该任务不存在，则panic
    pub fn remove_sync(&self, queue_id: isize, id: isize) -> T {
        let mut lock = self.sync_pool.1.lock().unwrap();
        let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
        let (elem, index) = pool.remove_elem(from_queue_id(queue_id) , indexs.load(from_sync_id(id)));
        indexs.destroy(index);
        self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
        elem
    }

    /// 尝试移除一个队列任务，返回被移除的任务，如果任务不存在，则返回None
    pub fn try_remove_sync(&self, queue_id: isize, id: isize) -> Option<T> {
        if is_queue(queue_id) && is_sync(id){
            let mut lock = self.sync_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let r = pool.try_remove_elem(from_queue_id(queue_id), indexs.load(from_sync_id(id)));
            match r {
                Some((elem, index)) => {
                    indexs.destroy(index);
                    self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                    return Some(elem)
                },
                None => return None,
            }
        }
        None
    }

    /// 移除一个单例任务， 返回被移除的任务，如果该任务不存在，则panic
    pub fn remove_async(&self, id: isize) -> T {
        let mut lock = self.async_pool.1.lock().unwrap();
        let (pool, indexs): &mut(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
        let (elem, _, i) = unsafe{pool.delete(indexs.load(from_async_id(id)), indexs)};
        indexs.destroy(i);
        self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
        elem
    }

    /// 尝试移除一个单例任务，返回被移除的任务，如果该任务不存在，则返回None
    pub fn try_remove_async(&self, id: isize) -> Option<T> {
        if is_async(id){
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let index  = from_async_id(id);
            match indexs.try_load(index) {
                Some(i) => {
                    let (elem, _, _) = unsafe{pool.delete(i, indexs)};
                    indexs.destroy(index);
                    self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
                    return Some(elem)
                },
                None => return None,
            }
        }
        None
    }

    /// 检查指定队列是否被锁住
    pub fn is_locked(&self, id: isize) -> bool {
        if is_queue(id) {
            self.sync_pool.1.lock().unwrap().0.is_locked(id as usize)
        } else if is_static_queue(id) {
            self.static_sync_pool.1.lock().unwrap().is_locked(from_static_queue_id(id))
        } else {
            false
        }
    }

    /// 为指定队列加锁
    pub fn lock_queue(&self, id: isize) -> bool {
        if is_queue(id){
            let mut lock = self.sync_pool.1.lock().unwrap();
            let r = lock.0.lock_queue(from_queue_id(id));
            if r {
                self.sync_pool.0.store(lock.0.get_weight(), AOrd::Relaxed);
            }
            r
        }else if is_static_queue(id) {
            let mut lock = self.static_sync_pool.1.lock().unwrap();
            let r = lock.lock_queue(from_static_queue_id(id));
            if r {
                self.static_sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
            }
            r
        }else {
            false
        }
    }

    /// 释放队列的锁，成功释放，则但会true， 否则返回false
    pub fn free_queue(&self, id: isize) -> bool{
        if is_queue(id){
            let (r, len) = {
                let mut lock = self.sync_pool.1.lock().unwrap();
                let r = lock.0.free_queue(from_queue_id(id));
                match r {
                    FreeSign::Success => {
                        self.sync_pool.0.store(lock.0.get_weight(), AOrd::Relaxed);
                        (true, lock.0.queue_len())
                    },
                    FreeSign::Ignore => (true, 0),
                    _ => (false, 0)
                }
            };
            self.notify(QueueType::DynSync, len);
            r
        }else if is_static_queue(id) {
            let (r, len) = {let mut lock = self.static_sync_pool.1.lock().unwrap();
                let r = lock.free_queue(from_static_queue_id(id));
                match r {
                    FreeSign::Success => {
                        self.static_sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                        (true, lock.queue_len())
                    },
                    FreeSign::Ignore => (true, 0),
                    _ => (false, 0)
                }
            };
            self.notify(QueueType::StaticSync, len);
            r
        }else {
            false
        }
    }

    /// 清空任务池
    pub fn clear(&self) {
        let mut sync_pool = self.sync_pool.1.lock().unwrap();
        sync_pool.0.clear();
        sync_pool.1.clear();
        self.static_sync_pool.1.lock().unwrap().clear();

        let mut async_pool = self.async_pool.1.lock().unwrap();
        async_pool.0.clear();
        async_pool.1.clear();
        self.static_async_pool.1.lock().unwrap().clear();

        self.sync_pool.0.store(0, AOrd::Relaxed);
        self.async_pool.0.store(0, AOrd::Relaxed);
        self.static_sync_pool.0.store(0, AOrd::Relaxed);
        self.static_async_pool.0.store(0, AOrd::Relaxed);
        self.delay_queue.clear();
    }

    /// 取到可删除的队列任务个数
    pub fn dyn_sync_len(&self) -> usize {
        self.sync_pool.1.lock().unwrap().0.len()
    }

    /// 取到不可删除的队列任务个数
    pub fn static_sync_len(&self) -> usize {
        self.static_sync_pool.1.lock().unwrap().len()
    }

    /// 取到可删除的单例任务个数
    pub fn dyn_async_len(&self) -> usize {
        self.async_pool.1.lock().unwrap().0.len()
    }

    /// 取到不可删除的单例任务个数
    pub fn static_async_len(&self) -> usize {
        self.static_async_pool.1.lock().unwrap().len()
    }

    /// 取到所有任务的个数
    pub fn len(&self) -> usize {
        let len1 = self.sync_pool.1.lock().unwrap().0.len();
        let len2 = self.static_async_pool.1.lock().unwrap().len();
        let len3 = self.async_pool.1.lock().unwrap().0.len();
        let len4 = self.static_sync_pool.1.lock().unwrap().len();
        len1 + len2 + len3 + len4
    }

    // 当某类任务的数量改变，则发出通知
    fn notify(&self, task_type: QueueType, task_size: usize) {
        if task_size <= self.count.load(AOrd::Relaxed) {
            (self.handler)(task_type, task_size)
        }
    }

    fn weight_rng_inner(&self) -> (usize, usize, usize, usize, usize){
        let async_w = self.async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let sync_w = self.sync_pool.0.load(AOrd::Relaxed);  //同步池总权重
        let static_async_w = self.static_async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let r: usize = self.rng.lock().unwrap().gen(); // 由外部实现随机生成器， TODO
        let amount = async_w + sync_w + static_async_w;
        let w = if amount == 0 {
            0
        }else {
            r%amount
        };
        (async_w, sync_w, static_async_w, r, w)
    }

    fn weight_rng(&self) -> (usize, usize, usize, usize, usize, usize){
        let async_w = self.async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let sync_w = self.sync_pool.0.load(AOrd::Relaxed);  //同步池总权重
        let static_async_w = self.static_async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let static_sync_w = self.static_sync_pool.0.load(AOrd::Relaxed);  //同步池总权重
        let r: usize = self.rng.lock().unwrap().gen(); // 由外部实现随机生成器， TODO
        let amount = async_w + sync_w + static_async_w + static_sync_w;
        let w = if amount == 0 {
            0
        }else {
            r%amount
        };
        (async_w, sync_w, static_async_w, static_sync_w, r, w)
    }
}

unsafe impl<T: Debug + Send> Send for TaskPool<T> {}
unsafe impl<T: Debug + Send> Sync for TaskPool<T> {}

impl<T: fmt::Debug> fmt::Debug for TaskPool<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sync_pool = self.sync_pool.1.lock().unwrap();
        let async_pool = self.async_pool.1.lock().unwrap();
        let static_sync_pool = self.static_sync_pool.1.lock().unwrap();
        let static_async_pool = self.static_async_pool.1.lock().unwrap();
        write!(f, r##"TaskPool (
sync_pool: ({:?},{:?}),
static_sync_pool: ({:?},{:?}),
async_pool: ({:?},{:?}),
static_async_pool: ({:?},{:?}),
        )"##, self.sync_pool.0, sync_pool.0, self.static_sync_pool.0, static_sync_pool, self.async_pool.0, async_pool.0, self.static_async_pool.0, static_async_pool)
    }
}

// pub struct QueueLock<T: 'static>{
//     sync_pool: Arc<(AtomicUsize, Mutex<SyncPool<T>>)>,
//     index: Arc<AtomicUsize>,
//     weight: usize,
// }

// impl<T: 'static> Drop for QueueLock<T> {
//     fn drop(&mut self){
//         println!("drop--------------------------------{:?}", self.index);
//         let mut lock = self.sync_pool.1.lock().unwrap();
//         lock.free_lock(&self.index, self.weight);
//         self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
//     }
// }








/// 延时任务
pub enum DelayTask<T: 'static> {
    Async{
        priority: usize,
        index: usize,
        async_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>)>)>,
        task:  NonNull<T>,
        handler: Arc<dyn Fn(QueueType, usize)>,
    },//异步任务
    Sync{
        queue_id: usize,
        index: usize,
        direc: Direction,
        sync_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>)>)>,
        task:  NonNull<T>,
        handler: Arc<dyn Fn(QueueType, usize)>,
    }//同步任务Sync(队列id, push方向)
}

/// 为延时任务实现Runer
impl<T: 'static> Runer for DelayTask<T> {
    fn run(self, _key: usize){
        match self {
            DelayTask::Async { priority,index, async_pool,task , handler} => {
                let pool_len;
                {
                    let mut lock = async_pool.1.lock().unwrap();
                    let (pool, indexs): &mut (dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
                    pool.push(unsafe {task.as_ptr().read()} , priority, index, indexs);
                    async_pool.0.store(pool.amount(), AOrd::Relaxed);
                    pool_len = pool.len();
                }
                handler(QueueType::DynAsync, pool_len);
            },
            DelayTask::Sync { queue_id, index, direc, sync_pool, task , handler} => {
                let queue_len;
                {
                    let mut lock = sync_pool.1.lock().unwrap();
                    let (pool, indexs): &mut (dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
                    let id = match direc {
                        Direction::Front => pool.push_front(unsafe {task.as_ptr().read()}, queue_id, index),
                        Direction::Back => pool.push_back(unsafe {task.as_ptr().read()}, queue_id, index)
                    };
                    sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                    indexs.store(index, id);
                    indexs.set_class(index, IndexType::Sync);
                    queue_len = pool.queue_len()
                }
                handler(QueueType::DynSync, queue_len);
            }

        }
    }
}

unsafe impl<T> Send for DelayTask<T> {}



#[inline]
fn to_static_queue_id(id: usize) -> isize{
    -(id as isize)
}

#[inline]
fn from_static_queue_id(id: isize) -> usize{
    (-id) as usize
}

#[inline]
fn is_static_queue(id: isize) -> bool{
    id < 0
}

#[inline]
fn to_queue_id(id: usize) -> isize{
    id as isize
}

#[inline]
fn from_queue_id(id: isize) -> usize{
    id as usize
}

#[inline]
fn is_queue(id: isize) -> bool{
    id > 0
}

#[inline]
fn to_sync_id(id: usize) -> isize{
    -(id as isize)
}

#[inline]
fn from_sync_id(id: isize) -> usize{
    -id as usize
}

#[inline]
fn is_sync(id: isize) -> bool{
    id < 0
}

#[inline]
fn to_async_id(id: usize) -> isize{
    id as isize
}

#[inline]
fn from_async_id(id: isize) -> usize{
    id as usize
}

#[inline]
fn is_async(id: isize) -> bool{
    id > 0
}

#[cfg(test)]
extern crate time;
#[cfg(test)]
use time::run_millis;
// #[cfg(test)]
// use std::thread;
// #[cfg(test)]
// use std::time::{Duration};

#[test]
fn test(){
    let task_pool: TaskPool<u32> = TaskPool::new(Timer::new(10), Arc::new(|_ty, _n| {}));

    let queue1 = task_pool.create_dyn_queue(1);
    let queue2 = task_pool.create_dyn_queue(2);
    let queue3 = task_pool.create_static_queue(1);
    let queue4 = task_pool.create_static_queue(2);
    let queue5 = task_pool.create_static_queue(3);

    /***************************************插入和弹出， 验证接口的正确性*************************************************** */
    task_pool.push_dyn_back(1, queue1);
    task_pool.push_dyn_back(2, queue1);
    task_pool.push_dyn_back(3, queue2);
    task_pool.push_dyn_back(4, queue2);
    task_pool.push_static_back(5, queue3);
    task_pool.push_static_back(6, queue3);
    task_pool.push_static_back(7, queue4);
    task_pool.push_static_back(8, queue4);
    task_pool.push_static_back(9, queue5);
    task_pool.push_static_back(10, queue5);

    task_pool.push_dyn_async(11, 1);
    task_pool.push_dyn_async(12, 2);
    task_pool.push_static_async(13, 1);
    task_pool.push_static_async(14, 2);

    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    //println!("create queue--{:?}", task_pool);
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    assert!(task_pool.pop_unlock().is_some());
    // // println!("create queue--{:?}", task_pool);

    // /***************************************多次插入和弹出， 验证接口的正确性*************************************************** */
    let m = 50;
    for i in 0..m {
        task_pool.push_dyn_back(i, queue1);
    }
    for i in 0..m {
        task_pool.push_dyn_back(i, queue2);
    }
    for i in 0..m {
        task_pool.push_static_back(i, queue3);
    }
    for i in 0..m {
        task_pool.push_static_back(i, queue4);
    }
    for i in 0..m {
        task_pool.push_static_back(i, queue5);
    }

    for i in 1..m + 1 {
        task_pool.push_dyn_async(i, i as usize);
    }

    for i in 0..m + 1 {
        task_pool.push_static_async(i, i as usize);
    }

    for _ in 0..m*7 {
        task_pool.pop_unlock().unwrap();
    }

    /***************************************测试移除接口*************************************************** */
    let index1 = task_pool.push_dyn_back(1, queue1);
    let index2 = task_pool.push_dyn_back(2, queue2);
    let index3 = task_pool.push_dyn_async(3, 2);

    assert_eq!(task_pool.remove_sync(queue1, index1), 1);
    assert_eq!(task_pool.remove_sync(queue2, index2), 2);
    assert_eq!(task_pool.remove_async(index3), 3);

    /******************************************测试带锁的弹出***************************************************************/
    task_pool.push_dyn_back(1, queue1);
    task_pool.push_dyn_back(2, queue1);
    assert!(task_pool.lock_queue(queue1));
    assert!(task_pool.pop().is_none());
    assert!(task_pool.free_queue(queue1));
    //println!("create queue--{:?}", task_pool);
    assert!(task_pool.pop().is_some());
    assert!(task_pool.pop().is_none());
    assert!(task_pool.free_queue(queue1));
    assert!(task_pool.pop().is_some());
    assert!(task_pool.free_queue(queue1));

    task_pool.delete_queue(queue2);
    task_pool.delete_queue(queue4);
    task_pool.delete_queue(queue5);

    task_pool.push_dyn_back(1, queue1);
    task_pool.push_static_back(2, queue3);
    task_pool.push_static_async(3, 3);
    task_pool.push_dyn_async(4, 4);
    task_pool.clear();
    assert_eq!(task_pool.len(), 0);
    use std::fs::File;
    flame::dump_text_to_writer(&mut File::create("flame.text").unwrap()).unwrap();
    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
}

#[test]
fn test_effect(){
    let task_pool: TaskPool<usize> = TaskPool::new(Timer::new(10), Arc::new(|_ty, _n| {}));

    let time = run_millis();
    for i in 1..100001 {
        task_pool.push_dyn_async(i, i);
    }
    println!("push_dyn_async-------{} ", run_millis() - time );

    let time = run_millis();
    for i in 1..100001 {
        task_pool.push_static_async(i, i);
    }
    println!("push_static_async-------{} ", run_millis() - time );

    let time = run_millis();
    for i in 1..1001 {
        task_pool.create_dyn_queue(i);
    }
    for i in 1..1001 {
        task_pool.create_static_queue(i);
    }
    println!("create_queue-------{} ", run_millis() - time );

    //task_pool.push_dyn_back(1, to_queue_id(1));
    let time = run_millis();
    for queue_id in 1..1001 {
        for i in 1..101 {
            task_pool.push_dyn_back(i, to_queue_id(queue_id));
        }
    }
    println!("push_dyn_back-------{} ", run_millis() - time );

    let time = run_millis();
    for queue_id in 1..1001 {
        for i in 1..101 {
            task_pool.push_static_back(i, to_static_queue_id(queue_id));
        }
    }
    println!("push_static_back-------{} ", run_millis() - time );

    let time = run_millis();
    for i in 1..100001 {
        task_pool.remove_async(to_async_id(i));
    }
    println!("remove_async-------{} ", run_millis() - time );

    let time = run_millis();
    for queue_id in 1..1001 {
        let q = (queue_id - 1) * 100;
        for i in 0..100 {
            //println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx{}", q + i + 1);
            task_pool.remove_sync(to_queue_id(queue_id), to_sync_id(q + i + 1));
        }
    }
    println!("remove_sync-------{} ", run_millis() - time );
}
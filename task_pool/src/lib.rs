#![feature(box_into_raw_non_null)]
#![feature(proc_macro_hygiene)]

extern crate rand;

extern crate flame;

extern crate flamer;

extern crate wtree;
extern crate timer;
extern crate index_class;
extern crate ver_index;
extern crate deque;
extern crate slab;
extern crate share;

pub mod enums;
mod sync_pool;

use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::{Mutex};
use std::marker::Send;
use std::fmt;

use rand::prelude::*;
use rand::rngs::SmallRng;

use timer::{Timer, Runer};
use wtree::wtree::WeightTree;
use wtree::simple_wtree::SimpleWeightTree;
use index_class::{IndexClassFactory};
use ver_index::ver::U32Index;
use deque::deque::{Direction};
use share::Share;

use enums:: {TaskType, DequeStat, FreeSign};


pub struct TaskPool<T: Debug + 'static>{
    static_sync_pool: Share<(AtomicUsize, Mutex<sync_pool::SyncPool<T, U32Index>>)>,
    sync_pool: Share<(AtomicUsize, Mutex<sync_pool::SyncPool<(T, u64), U32Index>>)>,
    static_async_pool: Share<(AtomicUsize, Mutex<SimpleWeightTree<T>>)>,
    async_pool: Share<(AtomicUsize, Mutex<(WeightTree<T, u64>, IndexClassFactory<u64, (), U32Index>)>)>,

    timer: Timer<DelayTask<T>, U32Index>,

    handler: Share<dyn Fn(TaskType, usize)>,
    count: AtomicUsize,
    rng: Share<Mutex<SmallRng>>,
}

impl<T: Debug + 'static> TaskPool<T> {
    pub fn new(timer: Timer<DelayTask<T>, U32Index>, handler: Share<dyn Fn(TaskType, usize)>) -> Self {
        TaskPool {
            static_sync_pool: Share::new((AtomicUsize::new(0), Mutex::new(sync_pool::SyncPool::default()))),
            sync_pool: Share::new((AtomicUsize::new(0), Mutex::new(sync_pool::SyncPool::default()))),
            static_async_pool: Share::new((AtomicUsize::new(0), Mutex::new(SimpleWeightTree::default()))),
            async_pool: Share::new((AtomicUsize::new(0), Mutex::new((WeightTree::default(), IndexClassFactory::default())))),

            timer: timer,
            count: AtomicUsize::new(0),
            handler,
            rng: Share::new(Mutex::new(SmallRng::from_entropy())),
        }
    }

    pub fn set_count(&self, count: usize) {
        self.count.store(count, AOrd::Relaxed);
    }

    // create sync queues, return true, or false if id is exist
    pub fn create_dyn_queue(&self, weight: usize) -> i64 {
        to_queue_id(self.sync_pool.1.lock().unwrap().create_queue(weight))
    }

    pub fn create_static_queue(&self, weight: usize) -> i64 {
        to_static_queue_id(self.static_sync_pool.1.lock().unwrap().create_queue(weight))
    }

    // delete queue, return true, or false if id is not exist
    pub fn delete_queue(&self, queue_id: i64) -> bool {
        if is_queue(queue_id) {
            self.sync_pool.1.lock().unwrap().remove_queue(from_queue_id(queue_id))
        } else if is_static_queue(queue_id) {
            self.static_sync_pool.1.lock().unwrap().remove_queue(from_static_queue_id(queue_id))
        } else {
            false
        }
    }
    // // push a sync task, return task id, or 0 if queue id is not exist
    pub fn push_dyn_front(&self, task: T, queue_id: i64) -> i64 {
        self.push_dyn(queue_id, task, Direction::Front)
    }
    // push a sync task, return task id, or 0 if queue id is not exist
    pub fn push_dyn_back(&self, task: T, queue_id: i64) -> i64 {
        self.push_dyn(queue_id, task, Direction::Back)
    }

    // // push a sync task, return task id, or 0 if queue id is not exist
    pub fn push_dyn(&self, queue_id: i64, task: T, direct: Direction) -> i64 {
        let (id, task_len) = {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let (id, locked) = lock.push(from_queue_id(queue_id), (task, 0), direct);
            if id > 0 {
                self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
            }
            if locked {
                (id, 0)
            } else {
                (id, lock.len())
            }
        };
        if task_len > 0 {
            self.notify(TaskType::DynSync, task_len);
        }
        to_sync_id(id)
    }
    // // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_static_front(&self, task: T, queue_id: i64) -> bool {
        self.push_static(queue_id, task, Direction::Front)
    }
    // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_static_back(&self, task: T, queue_id: i64) -> bool {
        self.push_static(queue_id, task, Direction::Back)
    }

    // // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_static(&self, queue_id: i64, task: T, direct: Direction) -> bool {
        let (id, task_len) = {
            let mut lock = self.static_sync_pool.1.lock().unwrap();
            let (id, locked) = lock.push(from_static_queue_id(queue_id), task, direct);
            if id > 0 {
                self.static_sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
            }
            if locked {
                (id, 0)
            } else {
                (id, lock.len())
            }
        };
        if task_len > 0 {
            self.notify(TaskType::StaticSync, task_len);
        }
        id > 0
    }

    // // push a async task
    pub fn push_dyn_async(&self, task: T, weight: usize) -> i64 {
        let (id, task_len) = {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, factory) = &mut *lock;
            let id = factory.create(0, 0, ());
            pool.push(task, weight, id, factory);
            self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
            (id, pool.len())
        };
        self.notify(TaskType::DynAsync, task_len);
        to_async_id(id)
    }

    pub fn push_static_async(&self, task: T, weight: usize) {
        let len = {
            let mut lock = self.static_async_pool.1.lock().unwrap();
            lock.push(task, weight);
            self.static_async_pool.0.store(lock.amount(), AOrd::Relaxed);
            lock.len()
        };
        self.notify(TaskType::StaticAsync, len);
    }

    //push a delay task, return Share<AtomicUsize> as index
    pub fn push_sync_delay(&self, task: T, queue_id: i64, direct: Direction, ms: u32) -> i64{
        let mut lock = self.sync_pool.1.lock().unwrap();
        let id = lock.create_node((task, 0));
        let t = DelayTask::Sync {
            queue_id: from_queue_id(queue_id),
            direct: direct,
            id: id,
            sync_pool: self.sync_pool.clone(),
            handler: self.handler.clone(),
        };
        let timer_id = self.timer.set_timeout(t, ms);
        lock.get_node_mut(id).unwrap().elem.1 = timer_id;
        to_sync_id(id)
    }

    pub fn push_async_delay(&self, task: T, weight: usize, ms: u32) -> i64{
        let mut lock = self.async_pool.1.lock().unwrap();
        let id = lock.1.create(0, 0, ());
        let t = DelayTask::Async {
            weight: weight,
            id: id,
            async_pool: self.async_pool.clone(),
            task: task,
            handler: self.handler.clone(),
        };
        let timer_id = self.timer.set_timeout(t, ms);
        unsafe{lock.1.get_unchecked_mut(id)}.class = timer_id;
        to_async_id(id)
    }

    //pop a task
    pub fn pop(&self, locked: bool, limit_type: usize) -> Option<(T, i64)>{
        let (sync_w, static_sync_w, async_w, _static_async_w, rand, mut w) = self.weight_rng(limit_type);
        if rand == 0 {
            return None
        }
        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let w = lock.get_weight();
            if w != 0 {
                let r = if locked {
                    lock.pop_lock(rand%w)
                }else{
                    lock.pop(rand%w)
                };
                match r.0 {
                    Some((task, _)) => {
                        self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                        return Some((task, to_queue_id(r.1)))
                    },
                    _ => return None
                }
            }
        } else {
            w = w - sync_w;
        }
        if w < static_sync_w {
            let mut lock = self.static_sync_pool.1.lock().unwrap();
            let w = lock.get_weight();
            if w != 0 {
                let r = if locked {
                    lock.pop_lock(rand%w)
                }else{
                    lock.pop(rand%w)
                };
                match r.0 {
                    Some(task) => {
                        self.static_sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                        return Some((task, to_static_queue_id(r.1)))
                    },
                    _ => return None
                }
            }
        } else {
            w = w - static_sync_w;
        }

        if w < async_w {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (tree, factory) = &mut *lock;
            let w = tree.amount();
            if w != 0 {
                let r = tree.pop(rand%w, factory).unwrap();
                self.async_pool.0.store(tree.amount(), AOrd::Relaxed);
                factory.remove(r.2);
                return Some((r.0, 0));
            }
        }
        let mut lock = self.static_async_pool.1.lock().unwrap();
        let w = lock.amount();
        if w == 0 {
            return None
        }
        let r = lock.pop(rand%w).unwrap();
        self.static_async_pool.0.store(lock.amount(), AOrd::Relaxed);
        return Some((r.0, 0));
    }

    pub fn remove_sync(&self, queue_id: i64, id: i64) {
        if is_queue(queue_id) && is_sync(id) {
            let timer_id = {
                let mut lock = self.sync_pool.1.lock().unwrap();
                match lock.remove(from_queue_id(queue_id) , from_sync_id(id)) {
                    Some((_, timer_id)) => if timer_id > 0 {
                        timer_id
                    }else{
                        self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                        return
                    },
                    _ => return
                }
            };
            self.timer.cancel(timer_id);
        }
    }

    pub fn remove_async(&self, id: i64) {
        if !is_async(id) {
            return;
        }
        let timer_id = {
            let mut lock = self.async_pool.1.lock().unwrap();
            let (pool, factory) = &mut *lock;
            match factory.remove(from_async_id(id)) {
                Some(r) => if r.class > 0 {
                    r.class
                }else{
                    unsafe{pool.delete(r.index, factory)};
                    self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
                    return
                },
                None => return
            }
        };
        self.timer.cancel(timer_id);
    }

    //check queue locked
    pub fn get_queue_stat(&self, id: i64) -> Option<DequeStat> {
        if is_queue(id) {
            self.sync_pool.1.lock().unwrap().get_queue_stat(from_queue_id(id))
        } else if is_static_queue(id) {
            self.static_sync_pool.1.lock().unwrap().get_queue_stat(from_static_queue_id(id))
        } else {
            None
        }
    }

    //lock sync_queue
    pub fn lock_queue(&self, id: i64) -> bool {
        if is_queue(id){
            let mut lock = self.sync_pool.1.lock().unwrap();
            let r = lock.lock_queue(from_queue_id(id));
            if r {
                self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
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

    //free lock sync_queue
    pub fn unlock_queue(&self, id: i64) -> bool{
        if is_queue(id){
            let (r, len) = {
                let mut lock = self.sync_pool.1.lock().unwrap();
                let r = lock.unlock_queue(from_queue_id(id));
                match r {
                    FreeSign::Success => {
                        self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                        (true, lock.len())
                    },
                    FreeSign::Ignore => (true, 0),
                    _ => (false, 0)
                }
            };
            self.notify(TaskType::DynSync, len);
            r
        }else if is_static_queue(id) {
            let (r, len) = {
                let mut lock = self.static_sync_pool.1.lock().unwrap();
                let r = lock.unlock_queue(from_static_queue_id(id));
                match r {
                    FreeSign::Success => {
                        self.static_sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                        (true, lock.len())
                    },
                    FreeSign::Ignore => (true, 0),
                    _ => (false, 0)
                }
            };
            self.notify(TaskType::StaticSync, len);
            r
        }else {
            false
        }
    }

    pub fn clear(&self) {
        let mut sync_pool = self.sync_pool.1.lock().unwrap();
        sync_pool.clear();
        sync_pool.clear();
        self.static_sync_pool.1.lock().unwrap().clear();

        let mut async_pool = self.async_pool.1.lock().unwrap();
        async_pool.0.clear();
        async_pool.1.clear();
        self.static_async_pool.1.lock().unwrap().clear();

        self.sync_pool.0.store(0, AOrd::Relaxed);
        self.async_pool.0.store(0, AOrd::Relaxed);
        self.static_sync_pool.0.store(0, AOrd::Relaxed);
        self.static_async_pool.0.store(0, AOrd::Relaxed);
        self.timer.clear();
    }

    pub fn len(&self) -> usize {
        let len1 = self.sync_pool.0.load(AOrd::Relaxed);
        let len2 = self.static_async_pool.0.load(AOrd::Relaxed);
        let len3 = self.async_pool.0.load(AOrd::Relaxed);
        let len4 = self.static_sync_pool.0.load(AOrd::Relaxed);
        len1 + len2 + len3 + len4
    }

    fn notify(&self, task_type: TaskType, task_size: usize) {
        if task_size <= self.count.load(AOrd::Relaxed) {
            (self.handler)(task_type, task_size)
        }
    }

    fn weight_rng(&self, limit_type: usize) -> (usize, usize, usize, usize, usize, usize){
        let sync_w = if limit_type & TaskType::DynAsync as usize != 0 {
            self.sync_pool.0.load(AOrd::Relaxed)  //同步池总权重
        }else{
            0
        };
        let static_sync_w = if limit_type & TaskType::DynAsync as usize != 0 {
            self.static_sync_pool.0.load(AOrd::Relaxed)  //静态同步池总权重
        }else{
            0
        };
        let async_w = if limit_type & TaskType::DynAsync as usize != 0 {
            self.async_pool.0.load(AOrd::Relaxed)  //异步池总权重
        }else{
            0
        };
        let static_async_w = if limit_type & TaskType::DynAsync as usize != 0 {
            self.static_async_pool.0.load(AOrd::Relaxed)  //静态异步池总权重
        }else{
            0
        };
        let amount = async_w + sync_w + static_async_w + static_sync_w;
        if amount == 0 {
            (sync_w, static_sync_w, async_w, static_async_w, 0, 0)
        }else {
            let r: usize = self.rng.lock().unwrap().gen();
            (sync_w, static_sync_w, async_w, static_async_w, r, r%amount)
        }
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
        )"##, self.sync_pool.0, sync_pool, self.static_sync_pool.0, static_sync_pool, self.async_pool.0, async_pool.0, self.static_async_pool.0, static_async_pool)
    }
}

// pub struct QueueLock<T: 'static>{
//     sync_pool: Share<(AtomicUsize, Mutex<SyncPool<T>>)>,
//     index: Share<AtomicUsize>,
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









pub enum DelayTask<T: 'static> {
    Async{
        weight: usize,
        id: u64,
        async_pool: Share<(AtomicUsize, Mutex<(WeightTree<T, u64>, IndexClassFactory<u64, (), U32Index>)>)>,
        task: T,
        handler: Share<dyn Fn(TaskType, usize)>,
    },//异步任务
    Sync{
        queue_id: u64,
        id: u64,
        direct: Direction,
        sync_pool: Share<(AtomicUsize, Mutex<sync_pool::SyncPool<(T, u64), U32Index>>)>,
        handler: Share<dyn Fn(TaskType, usize)>,
    }//同步任务Sync(队列id, push方向)
}

impl<T: 'static> Runer<u64> for DelayTask<T> {
    fn run(self, _key: u64){
        match self {
            DelayTask::Async { weight, id, async_pool, task, handler} => {
                let mut lock = async_pool.1.lock().unwrap();
                let (pool, factory) = &mut *lock;
                // 尝试将timer_id清空， 如果找不到该节点，则已被移除
                match factory.get_mut(id) {
                    Some(node) => node.class = 0,
                    _ => return
                }
                pool.push(task, weight, id, factory);
                async_pool.0.store(pool.amount(), AOrd::Relaxed);
                handler(TaskType::DynAsync, pool.len());
            },
            DelayTask::Sync { queue_id, id, direct, sync_pool, handler} => {
                let mut lock = sync_pool.1.lock().unwrap();
                // 尝试将timer_id清空， 如果找不到该节点，则已被移除
                match lock.get_node_mut(id) {
                    Some(node) => node.elem.1 = 0,
                    _ => return
                }
                lock.push_id(queue_id, id, direct);
                sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                handler(TaskType::DynSync, lock.len());
            }
        }
    }
}



unsafe impl<T> Send for DelayTask<T> {}


#[inline]
fn to_static_queue_id(id: u64) -> i64{
    -(id as i64)
}

#[inline]
fn from_static_queue_id(id: i64) -> u64{
    (-id) as u64
}

#[inline]
fn is_static_queue(id: i64) -> bool{
    id < 0
}

#[inline]
fn to_queue_id(id: u64) -> i64{
    id as i64
}

#[inline]
fn from_queue_id(id: i64) -> u64{
    id as u64
}

#[inline]
fn is_queue(id: i64) -> bool{
    id > 0
}

#[inline]
fn to_sync_id(id: u64) -> i64{
    -(id as i64)
}

#[inline]
fn from_sync_id(id: i64) -> u64{
    -id as u64
}

#[inline]
fn is_sync(id: i64) -> bool{
    id < 0
}

#[inline]
fn to_async_id(id: u64) -> i64{
    id as i64
}

#[inline]
fn from_async_id(id: i64) -> u64{
    id as u64
}

#[inline]
fn is_async(id: i64) -> bool{
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
    let task_pool: TaskPool<u32> = TaskPool::new(Timer::new(10), Share::new(|_, _| {}));

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

    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    //println!("create queue--{:?}", task_pool);
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
    assert!(task_pool.pop(false, 15).is_some());
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
        task_pool.pop(false, 15).unwrap();
    }

    /***************************************测试移除接口*************************************************** */
    let index1 = task_pool.push_dyn_back(1, queue1);
    let index2 = task_pool.push_dyn_back(2, queue2);
    let index3 = task_pool.push_dyn_async(3, 2);

    task_pool.remove_sync(queue1, index1);
    task_pool.remove_sync(queue2, index2);
    task_pool.remove_async(index3);

    /******************************************测试带锁的弹出***************************************************************/
    task_pool.push_dyn_back(1, queue1);
    task_pool.push_dyn_back(2, queue1);
    assert!(task_pool.lock_queue(queue1));
    assert!(task_pool.pop(true, 15).is_none());
    assert!(task_pool.unlock_queue(queue1));
    //println!("create queue--{:?}", task_pool);
    assert!(task_pool.pop(true, 15).is_some());
    assert!(task_pool.pop(true, 15).is_none());
    assert!(task_pool.unlock_queue(queue1));
    assert!(task_pool.pop(true, 15).is_some());
    assert!(task_pool.unlock_queue(queue1));

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
    let task_pool: TaskPool<usize> = TaskPool::new(Timer::new(10), Share::new(|_, _| {}));

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
#![feature(box_into_raw_non_null)]
#![feature(proc_macro_hygiene)]
extern crate rand;

extern crate flame;
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

use rand::Rng;

use timer::{Timer, Runer};
use dyn_uint::{SlabFactory, UintFactory, ClassFactory};

use enums:: {IndexType, Direction, Task, FreeSign};

pub struct TaskPool<T: Debug + 'static>{
    static_sync_pool: Arc<(AtomicUsize, Mutex<static_pool::SyncPool<T>>)>,
    // static_lock_queues: Arc<Mutex<Slab<WeightQueue<T>>>>,

    sync_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>)>)>,
    //lock_queues: Arc<Mutex<Slab<WeightQueueD<T>>>>,

    static_async_pool: Arc<(AtomicUsize, Mutex<static_pool::AsyncPool<T>>)>,
    async_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>)>)>,

    delay_queue: Timer<DelayTask<T>>,

    handler: Box<Fn()>,
    count: AtomicUsize,
}

impl<T: Debug + 'static> TaskPool<T> {
    pub fn new(timer: Timer<DelayTask<T>>, handler: Box<Fn()>) -> Self {
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
        }
    }

    pub fn set_count(&self, count: usize) {
        self.count.store(count, AOrd::SeqCst);
    }

    // create sync queues, return true, or false if id is exist
    pub fn create_dyn_queue(&self, weight: usize) -> isize {
        to_queue_id(self.sync_pool.1.lock().unwrap().0.create_queue(weight))
    }

    pub fn create_static_queue(&self, weight: usize) -> isize {
        to_static_queue_id(self.static_sync_pool.1.lock().unwrap().create_queue(weight))
    }

    // // delete sync queues, return true, or false if id is not exist
    // pub fn delete_dny_queue(&self, id: isize) -> bool{
    //     self.sync_pool.1.lock().unwrap().0.remove_queue(from_queue_id(id));
    // }

    // // // delete sync queues, return true, or false if id is not exist
    // pub fn delete_static_queue(&self, id: isize) {
    //     self.static_sync_pool.1.lock().unwrap().remove_queue(from_static_queue_id(id));
    // }
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

    // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_dyn_back(&self, task: T, queue_id: isize) -> isize {
        let (id, queue_len) = {
            let mut sync_pool = self.sync_pool.1.lock().unwrap();
            let id = sync_pool.1.create(0, IndexType::Sync, ());
            let index = sync_pool.0.push_back(task, from_queue_id(queue_id), id);
            self.sync_pool.0.store(sync_pool.0.get_weight(), AOrd::Relaxed);
            sync_pool.1.store(id, index);
//            println!("!!!!!!push dyn sync push back, weight:{}, len: {}", sync_pool.0.get_weight(), sync_pool.0.queue_len());
            (id, sync_pool.0.queue_len())
        };
//        println!("!!!!!!push dyn sync queue start");
        self.notify(queue_len);
//        println!("!!!!!!push dyn sync queue finish");
        to_sync_id(id)
    }

    // // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_dyn_front(&self, task: T, queue_id: isize) -> isize {
        let (id, queue_len) = {
            let mut sync_pool = self.sync_pool.1.lock().unwrap();
            let id = sync_pool.1.create(0, IndexType::Sync, ());
            let index = sync_pool.0.push_front(task, from_queue_id(queue_id), id);
            self.sync_pool.0.store(sync_pool.0.get_weight(), AOrd::Relaxed);
            sync_pool.1.store(id, index);
            (id, sync_pool.0.queue_len())
        };
        self.notify(queue_len);
        to_sync_id(id)
    }

    // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_static_back(&self, task: T, queue_id: isize) {
        let len = {
            let mut sync_pool = self.static_sync_pool.1.lock().unwrap();
            sync_pool.push_back(task, from_static_queue_id(queue_id));
            self.static_sync_pool.0.store(sync_pool.get_weight(), AOrd::Relaxed);
            sync_pool.queue_len()
        };
//        println!("!!!!!!push static sync queue start");
        self.notify(len);
//        println!("!!!!!!push static sync queue start");
    }

    // // push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_static_front(&self, task: T, queue_id: isize) {
        let len = {
            let mut sync_pool = self.static_sync_pool.1.lock().unwrap();
            sync_pool.push_front(task, from_static_queue_id(queue_id));
            self.static_sync_pool.0.store(sync_pool.get_weight(), AOrd::Relaxed);
            sync_pool.queue_len()
        };
        self.notify(len);
    }

    // // push a async task
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
        self.notify(len);
//        println!("!!!!!!push dyn async queue finish");
        to_async_id(index)
    }

    pub fn push_static_async(&self, task: T, priority: usize) {
        let len = {
            let mut lock = self.static_async_pool.1.lock().unwrap();
            lock.push(task, priority);
            self.static_async_pool.0.store(lock.amount(), AOrd::Relaxed);
            lock.len()
        };
//        println!("!!!!!!push static async queue start");
        self.notify(len);
//        println!("!!!!!!push dyn async queue finish");
    }

    //push a delay task, return Arc<AtomicUsize> as index
    pub fn push_sync_delay(&self, task: T, queue_id: isize, direc: Direction, ms: u32) -> isize{
        let index = self.sync_pool.1.lock().unwrap().1.create(0, IndexType::Delay, ());
        let task = DelayTask::Sync {
            queue_id: from_sync_id(queue_id),
            direc: direc,
            index: index,
            sync_pool: self.sync_pool.clone(),
            task: Box::into_raw_non_null(Box::new(task)),
        };
        let index1 = self.delay_queue.set_timeout(task, ms);
        self.sync_pool.1.lock().unwrap().1.store(index, index1);
        to_sync_id(index)
    }

    pub fn push_async_delay(&self, task: T, priority: usize, ms: u32) -> isize{
        let index = self.sync_pool.1.lock().unwrap().1.create(0, IndexType::Delay, ());
        let task = DelayTask::Async {
            priority: priority,
            index: index,
            async_pool: self.async_pool.clone(),
            task: Box::into_raw_non_null(Box::new(task)),
        };
        let index1 = self.delay_queue.set_timeout(task, ms);
        self.sync_pool.1.lock().unwrap().1.store(index, index1);
        to_async_id(index)
    }

    //pop a task
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

    pub fn pop(&self) -> Option<Task<T>>{
        let (async_w, sync_w, static_async_w, static_sync_w, r, mut w) = self.weight_rng();
//        println!("w--------------{:?}", (async_w, sync_w, static_async_w, static_sync_w, r, w));
        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
            let w = pool.get_weight();
            if w != 0 {
                let r = pool.pop_front_with_lock(r%w);
                let elem = r.0.unwrap();
                self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                indexs.destroy(elem.1);
//                println!("w---dyn_sync_pop");
                return Some(Task::Sync(elem.0, to_sync_id(r.1) ));
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
//                println!("w---dyn_async_pop");
                return Some(Task::Async(r.0));
            }
        } else {
            w = w - async_w;
        }

        
        if w < static_async_w {
            let mut pool = self.static_async_pool.1.lock().unwrap();
            let w = pool.amount();
            if w != 0 {
                let r = Some(Task::Async(pool.pop(r%w).0));
                self.static_async_pool.0.store(pool.amount(), AOrd::Relaxed);
//                println!("w---static_async_pop");
                return r;
            }
        } else {
            w = w - static_async_w;
        }

        if w < static_sync_w {
            let mut pool = self.static_sync_pool.1.lock().unwrap();
            let w = pool.get_weight();
            if w != 0 {
                let r = pool.pop_front_with_lock(r%w);
                let elem = r.0.unwrap();
                self.static_sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
//                println!("w---static_sync_pop");
                return Some(Task::Sync(elem, to_static_queue_id(r.1)));
            }
        }
//        println!("w---empty_pop");
        None
    }

    pub fn remove_sync(&self, queue_id: isize, id: isize) -> T {
        let mut lock = self.sync_pool.1.lock().unwrap();
        let (pool, indexs): &mut(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
        let (elem, index) = pool.remove_elem(from_queue_id(queue_id) , indexs.load(from_sync_id(id)));
        indexs.destroy(index);
        self.sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
        elem
    }

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

    pub fn remove_async(&self, id: isize) -> T {
        let mut lock = self.async_pool.1.lock().unwrap();
        let (pool, indexs): &mut(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
        let (elem, _, i) = unsafe{pool.delete(indexs.load(from_async_id(id)), indexs)};
        indexs.destroy(i);
        self.async_pool.0.store(pool.amount(), AOrd::Relaxed);
        elem
    }

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

    //lock sync_queue
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

    //free lock sync_queue
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
            self.notify(len);
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
            self.notify(len);
            r
        }else {
            false
        }
    }

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

    pub fn len(&self) -> usize {
        let len1 = self.sync_pool.1.lock().unwrap().0.len();
        let len2 = self.static_async_pool.1.lock().unwrap().len();
        let len3 = self.async_pool.1.lock().unwrap().0.len();
        let len4 = self.static_sync_pool.1.lock().unwrap().len();
        len1 + len2 + len3 + len4
    }

    fn notify(&self, len: usize) {
        if len <= self.count.load(AOrd::SeqCst) {
            (self.handler)()
        }
    }

    fn weight_rng(&self) -> (usize, usize, usize, usize, usize, usize){
        let async_w = self.async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let sync_w = self.sync_pool.0.load(AOrd::Relaxed);  //同步池总权重
        let static_async_w = self.static_async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let static_sync_w = self.static_sync_pool.0.load(AOrd::Relaxed);  //同步池总权重
        let r: usize = rand::thread_rng().gen(); // 由外部实现随机生成器， TODO
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









pub enum DelayTask<T: 'static> {
    Async{
        priority: usize,
        index: usize,
        async_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>)>)>,
        task:  NonNull<T>,
    },//异步任务
    Sync{
        queue_id: usize,
        index: usize,
        direc: Direction,
        sync_pool: Arc<(AtomicUsize, Mutex<(dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>)>)>,
        task:  NonNull<T>,
    }//同步任务Sync(队列id, push方向)
}

impl<T: 'static> Runer for DelayTask<T> {
    fn run(self, _key: usize){
        match self {
            DelayTask::Async { priority,index, async_pool,task } => {
                let mut lock = async_pool.1.lock().unwrap();
                let (pool, indexs): &mut (dyn_pool  ::AsyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
                pool.push(unsafe {task.as_ptr().read()} , priority, index, indexs);
                async_pool.0.store(pool.amount(), AOrd::Relaxed);
            },
            DelayTask::Sync { queue_id, index, direc, sync_pool, task } => {
                let mut lock = sync_pool.1.lock().unwrap();
                let (pool, indexs): &mut (dyn_pool  ::SyncPool<T>, SlabFactory<IndexType, ()>) = &mut *lock;
                let id = match direc {
                    Direction::Front => pool.push_front(unsafe {task.as_ptr().read()}, queue_id, index),
                    Direction::Back => pool.push_front(unsafe {task.as_ptr().read()}, queue_id, index)
                };
                sync_pool.0.store(pool.get_weight(), AOrd::Relaxed);
                indexs.store(index, id);
                indexs.set_class(index, IndexType::Sync);
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
	let task_pool: TaskPool<u32> = TaskPool::new(Timer::new(10), Box::new(|| {}));
    
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
    let task_pool: TaskPool<usize> = TaskPool::new(Timer::new(10), Box::new(|| {}));

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
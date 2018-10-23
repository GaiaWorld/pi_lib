use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering as AOrd};
use std::sync::{Arc, Mutex};
use std::marker::Send;
use rand::Rng;
use rand;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::fmt;

use fnv::FnvHashMap;

use wtree::WeightTree;
use timer::{Timer, Runer};
use fast_deque::FastDeque;

pub struct TaskPool<T: 'static>{
    sync_pool: (Arc<AtomicUsize>, Arc<Mutex<SyncPool<T>>>),
    async_pool: (Arc<AtomicUsize>, Arc<Mutex<AsyncPool<T>>>),
    delay_queue: Timer<DelayTask<T>>,
}

impl<T: 'static> TaskPool<T>{
    pub fn new() -> Self {
        let timer = Timer::new(10);
        timer.run();
        TaskPool {
            sync_pool: (Arc::new(AtomicUsize::new(0)), Arc::new(Mutex::new(SyncPool::new()))),
            async_pool: (Arc::new(AtomicUsize::new(0)), Arc::new(Mutex::new(AsyncPool::new()))),
            delay_queue: timer,
        }
    }
    
    /// create sync queues, return true, or false if id is exist
    pub fn create_sync_queue(&self, id: u64, weight: usize, can_del: bool) -> bool{
        self.sync_pool.1.lock().unwrap().create_queue(id, weight, can_del)
    }

    /// delete sync queues, return true, or false if id is not exist
    pub fn del_sync_queue(&self, id: &u64) {
        self.sync_pool.1.lock().unwrap().remove(id)
    }

    /// push a task, return Ok(index), or Err if tasktype is Sync and queue id is exist
    // pub fn push(&self, task: T, task_type: TaskType) -> Result<Arc<AtomicUsize>, Atom>{
    //     match task_type {
    //         TaskType::Async(priority) => {
    //             Ok(self.push_async(task, priority))
    //         },
    //         TaskType::Sync(id, direc) => {
    //             self.push_sync(task, id, direc)
    //         }
    //     }
    // }

    /// push a sync task, return Ok(index), or Err if queue id is exist
    pub fn push_sync(&self, task: T, id: u64, direc: Direction) -> Result<Arc<AtomicUsize>, String>{
        let mut sync_pool = self.sync_pool.1.lock().unwrap();
        match direc {
            Direction::Front => {
                let r = sync_pool.push_front(id, task);
                self.sync_pool.0.store(sync_pool.get_weight(), AOrd::Relaxed);
                r
            },
            Direction::Back => {
                let r = sync_pool.push_back(id, task);
                self.sync_pool.0.store(sync_pool.get_weight(), AOrd::Relaxed);
                r
            }
        }
    }

    /// push a async task, return Arc<AtomicUsize> as index
    #[inline]
    pub fn push_async(&self, task: T, priority: usize) -> Arc<AtomicUsize> {
        let mut lock = self.async_pool.1.lock().unwrap();
        let r = lock.push(task, priority);
        self.async_pool.0.store(lock.amount(), AOrd::Relaxed);
        r
    }

    /// push a delay task, return Arc<AtomicUsize> as index
    pub fn push_delay(&self, mut task: T, task_type: TaskType, ms: u32) -> Arc<AtomicUsize> {
        let r = DelayTask{
            task_type: task_type,
            task: Box::into_raw_non_null(Box::new(task)),
            sync_pool: self.sync_pool.clone(),
            async_pool: self.async_pool.clone(),
        };
        self.delay_queue.set_timeout(r, ms)
    }

    /// pop a task by weight
    pub fn pop(&self) -> Option<T>{
        let async_w = self.async_pool.0.load(AOrd::Relaxed);  //异步池总权重
        let sync_w = self.sync_pool.0.load(AOrd::Relaxed);  //同步池总权重
        let r: usize = rand::thread_rng().gen();
        let amount = async_w + sync_w;
        let w =  if amount == 0 {
            0
        }else {
            r%amount
        };
        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let w = lock.get_weight();
            if w != 0 {
                let r = Some(lock.pop_front(r%w));
                self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                return r;
            }
        }
        let mut lock = self.async_pool.1.lock().unwrap();
        let w = lock.amount();
        if w != 0 {
            let r = Some(lock.remove_by_weight(r%w).0);
            self.async_pool.0.store(lock.amount(), AOrd::Relaxed);
            return r;
        }else {
            return None;
        }
    }

    /// pop a task , lock the queue of tasks if task is sync
    pub fn pop_with_sync_lock(&self) -> Option<Task<T>>{
        let async_w = self.async_pool.0.load(AOrd::Relaxed); //异步池总权重
        let sync_w = self.sync_pool.0.load(AOrd::Relaxed); //同步池总权重
        let r: usize = rand::thread_rng().gen();
        let amount = async_w + sync_w;
        let w =  if amount == 0 {
            0
        }else {
            r%amount
        };
        if w < sync_w {
            let mut lock = self.sync_pool.1.lock().unwrap();
            let w = lock.get_weight();
            if w != 0 {
                let (mut elem, index, weight) = lock.pop_front_with_lock(r%w);
                self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                return Some(Task::Sync(TaskLock{
                    task: NonNull::new(&mut elem as *mut T).unwrap(),
                    sync_pool: self.sync_pool.1.clone(),
                    index: index,
                    weight: weight,
                }));
            }
        }
        let mut lock = self.async_pool.1.lock().unwrap();
        let w = lock.amount();
        if w != 0 {
            let r = Some(Task::Async(lock.remove_by_weight(r%w).0));
            self.async_pool.0.store(lock.amount(), AOrd::Relaxed);
            return r;
        }else {
            return None;
        }
    }

    pub fn clear(&self) {
        self.sync_pool.1.lock().unwrap().clear();
        self.async_pool.1.lock().unwrap().clear();
        self.sync_pool.0.store(0, AOrd::Relaxed);
        self.async_pool.0.store(0, AOrd::Relaxed);
        self.delay_queue.clear();
    }
}

unsafe impl<T: Send> Send for TaskPool<T> {}
unsafe impl<T: Send> Sync for TaskPool<T> {}

impl<T: fmt::Debug> fmt::Debug for TaskPool<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sync_pool = self.sync_pool.1.lock().unwrap();
        let async_pool = self.async_pool.1.lock().unwrap();
        write!(f, "TaskPool {{ sync_pool: ({}), async_pool: ({}) }}", sync_pool.len(), async_pool.len())
    }
}

pub struct TaskLock<T: 'static>{
    task: NonNull<T>,
    sync_pool: Arc<Mutex<SyncPool<T>>>,
    index: Arc<AtomicUsize>,
    weight: usize,
}

impl<T: 'static> TaskLock<T> {
    pub fn unwrap(self) -> T {
        unsafe {self.task.as_ptr().read()}
    }
}

impl<T: 'static> Drop for TaskLock<T> {
    fn drop(&mut self){
        self.sync_pool.lock().unwrap().free_lock(&self.index, self.weight);
    }
}


impl<T: 'static> Deref for TaskLock<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe{ self.task.as_ref() }
    }
}

impl<T: 'static> DerefMut for TaskLock<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe{ self.task.as_mut() }
    }
}

unsafe impl<T: Send> Send for TaskLock<T> {}

//任务
pub enum Task<T: 'static> {
    Async(T),
    Sync(TaskLock<T>),
}

//任务类型
#[derive(Clone)]
pub enum TaskType {
    Async(usize),      //异步任务, Async(任务优先级)
    Sync(u64, Direction),       //同步任务Sync(队列id)
}

//同步任务push的方向
#[derive(Clone)]
pub enum Direction {
    Front,
    Back,
}

struct SyncPool<T: 'static>{
    weight_queues: WeightTree<Rc<RefCell<WeightQueue<T>>>>,
    weight_map: FnvHashMap<u64, (Rc<RefCell<WeightQueue<T>>>, Arc<AtomicUsize>)>,
    len: usize,
}

unsafe impl<T: Send> Send for SyncPool<T> {}
unsafe impl<T: Send> Sync for SyncPool<T> {}

impl<T: 'static> SyncPool<T>{

    fn new() -> Self {
        SyncPool {
            weight_queues: WeightTree::new(),
            weight_map: FnvHashMap::default(),
            len: 0
        }
    }

    //create queues, if id is exist, return false
    fn create_queue(&mut self, id: u64, weight: usize, can_del: bool) -> bool {
        if self.weight_map.contains_key(&id) {
            return false;
        }else {
            let r = Rc::new(RefCell::new(WeightQueue::new(weight, can_del)));
            let index = self.weight_queues.push(r.clone(), 0);
            self.weight_map.insert(id, (r.clone(), index));
            return true;
        }
    }

    //Find a queue with weight, Removes the first element from the queue and returns it, Painc if weight >= get_weight().
    fn pop_front(&mut self, weight: usize) -> T {
        let (r, weight, index) = {
            let queue = self.weight_queues.get_mut_by_weight(weight);
            let mut q = queue.0.borrow_mut();
            (q.pop_front().unwrap(), q.get_weight(), queue.1.clone())  //如果能够根据权重取到队列， 必然能从队列中弹出元素
        };
        self.weight_queues.update_weight(weight, &index);
        self.len -= 1;
        r
    }

    //pop elements from specified queue, and not update weight, Painc if weight >= get_weight()
    fn pop_front_with_lock(&mut self, weight: usize) -> (T, Arc<AtomicUsize>, usize) {
        let r = {
            let queue = self.weight_queues.get_mut_by_weight(weight);
            let mut q = queue.0.borrow_mut();
            (q.pop_front().unwrap(), queue.1.clone(), q.get_weight()) //如果能够根据权重取到队列， 必然能从队列中弹出元素
        };
        self.weight_queues.update_weight(0, &r.1);
        self.len += 1;
        r
    }

    fn free_lock(&mut self, index: &Arc<AtomicUsize>, weight: usize) {
        self.weight_queues.update_weight(weight, &index);
    }

    //Find a queue with weight, Removes the last element from the queue and returns it, or None if the queue is empty or the queue is not exist.
    fn _pop_back(&mut self, weight: usize) -> Option<T> {
        let (r, weight, index) = {
            let queue = match self.weight_queues.try_get_mut_by_weight(weight){
                Some(v) => {v},
                None => return None
            };
            let mut q = queue.0.borrow_mut();
            (q._pop_back(), q.get_weight(), queue.1.clone())
        };
        self.weight_queues.update_weight(weight, &index);
        if r.is_some() {
            self.len -= 1;
        }
        r
    }

    //Append an element to the queue of the specified ID. return true, or false if the queue is not exist
    fn push_back(&mut self, id: u64, task: T) -> Result<Arc<AtomicUsize>, String> {
        let index = Arc::new(AtomicUsize::new(0));
        match self.push_back_with_index(id, task, &index){
            true => {
                Ok(index)
            },
            false => Err(String::from("id is not exist")),
        }
    }

    //Prepends an element to the queue of the specified ID. return true, or false if the queue is not exist
    fn push_front(&mut self, id: u64, task: T) -> Result<Arc<AtomicUsize>, String>{
        let index = Arc::new(AtomicUsize::new(0));
        match self.push_front_with_index(id, task, &index){
            true => {
                Ok(index)
            },
            false => Err(String::from("id is not exist")),
        }
    }

    fn push_front_with_index(&mut self, id: u64, task: T, index: &Arc<AtomicUsize>) -> bool{
        let (weight, index) = match self.weight_map.get_mut(&id){
            Some(q) => {
                self.len += 1;
                let mut queue = q.0.borrow_mut();
                queue.push_front_with_index(task, index);
                (queue.get_weight(), q.1.clone())
            },
            None => return false,
        };
        self.weight_queues.update_weight(weight, &index);
        true
    }

    fn push_back_with_index(&mut self, id: u64, task: T, index: &Arc<AtomicUsize>) -> bool{
        let (weight, index) = match self.weight_map.get_mut(&id){
            Some(q) => {
                self.len += 1;
                let mut queue = q.0.borrow_mut();
                queue.push_back_with_index(task, index);
                (queue.get_weight(), q.1.clone())
            },
            None => return false,
        };
        self.weight_queues.update_weight(weight, &index);
        true
    }

    //取队列的权重（所有任务的权重总值）
    fn get_weight(&self) -> usize{
        self.weight_queues.amount()
    }

    //移除指定id的队列
    fn remove(&mut self, id: &u64) {
       match self.weight_map.remove(id){
           Some((_, index)) => {
               self.weight_queues.remove(&index);
           },
           None => ()
       }
    }

    //清空同步任务池
    fn clear(&mut self) {
        self.weight_map.clear();
        self.weight_queues.clear();
    }

    //清空同步任务池
    fn len(&self) -> usize {
        self.len
    }
}

type AsyncPool<T> = WeightTree<T>;

struct DelayTask<T: 'static> {
    task_type: TaskType,
    task: NonNull<T>,
    sync_pool: (Arc<AtomicUsize>, Arc<Mutex<SyncPool<T>>>),
    async_pool: (Arc<AtomicUsize>, Arc<Mutex<AsyncPool<T>>>),
}

impl<T: 'static> Runer for DelayTask<T> {
    fn run(self, index: &Arc<AtomicUsize>){
        match self.task_type {
            TaskType::Async(priority) => {
                let mut lock = self.async_pool.1.lock().unwrap();
                self.async_pool.1.lock().unwrap().push(unsafe {self.task.as_ptr().read()} , priority);
                self.async_pool.0.store(lock.amount(), AOrd::Relaxed);
            },
            TaskType::Sync(id, direc) => {
                match direc {
                    Direction::Front => {
                        let mut lock = self.sync_pool.1.lock().unwrap();
                        lock.push_front_with_index(id, unsafe {self.task.as_ptr().read()}, &index);
                        self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                    },
                    Direction::Back => {
                        let mut lock = self.sync_pool.1.lock().unwrap();
                        lock.push_back_with_index(id, unsafe {self.task.as_ptr().read()}, &index);
                        self.sync_pool.0.store(lock.get_weight(), AOrd::Relaxed);
                    }
                }
            }
        }
    }
}

unsafe impl<T> Send for DelayTask<T> {}

enum Deque<T>{
    VecDeque(VecDeque<T>),
    FastDeque(FastDeque<T>)
}

//权重队列， WeightQueue(权重, 队列)
struct WeightQueue<T>{
    weight_unit: usize, //单个任务权重
    weight: usize, //队列总权重
    queue: Deque<T>, //队列
}

impl<T> WeightQueue<T>{
    fn new(weight_unit: usize, can_del: bool) -> Self{
        let q = match can_del {
            true => Deque::FastDeque(FastDeque::new()),
            false => Deque::VecDeque(VecDeque::new())
        };
        WeightQueue{
            weight_unit: weight_unit,
            weight: 0,
            queue: q
        }
    }

    fn _pop_back(&mut self) -> Option<T>{
        let r = match self.queue {
            Deque::FastDeque(ref mut queue) => queue.pop_back(),
            Deque::VecDeque(ref mut queue) => queue.pop_back(),
        };
        if r.is_some() {
            self.weight -= self.weight_unit;
        }
        r
    }

    fn pop_front(&mut self) -> Option<T>{
        let r = match self.queue {
            Deque::FastDeque(ref mut queue) => queue.pop_front(),
            Deque::VecDeque(ref mut queue) => queue.pop_front(),
        };
        if r.is_some() {
            self.weight -= self.weight_unit;
        }
        r
    }

    fn _push_back(&mut self, task: T) -> Arc<AtomicUsize>{
        let r = Arc::new(AtomicUsize::new(0));
        self.push_back_with_index(task, &r);
        r
    }

    fn _push_front(&mut self, task: T) -> Arc<AtomicUsize>{
        let r = Arc::new(AtomicUsize::new(0));
        self.push_front_with_index(task, &r);
        r
    }

    fn push_back_with_index(&mut self, task: T, index: &Arc<AtomicUsize>){
        self.weight += self.weight_unit;
        match self.queue {
            Deque::FastDeque(ref mut queue) => index.store(queue.push_back(task), AOrd::Relaxed),
            Deque::VecDeque(ref mut queue) => {queue.push_back(task); index.store(0, AOrd::Relaxed);},
        }
    }

    fn push_front_with_index(&mut self, task: T, index: &Arc<AtomicUsize>){
        self.weight += self.weight_unit;
        match self.queue {
            Deque::FastDeque(ref mut queue) => index.store(queue.push_front(task), AOrd::Relaxed),
            Deque::VecDeque(ref mut queue) => {queue.push_front(task); index.store(0, AOrd::Relaxed);},
        }
    }

    //取队列的权重（所有任务的权重总值）
    fn get_weight(&self) -> usize{
        self.weight
    }
}

#[cfg(test)]
use time::now_millis;
#[cfg(test)]
use std::thread;
#[cfg(test)]
use std::time::{Duration};

#[test]
fn test_sync(){
	let task_pool: Arc<TaskPool<u32>> = Arc::new(TaskPool::new());
    let syncs:[u32; 5] = [100000, 100000, 100000, 100000, 100000];
    let async = 100000;

    let now = now_millis();
    for i in 0..syncs.len() {
        task_pool.create_sync_queue(i as u64, i + 1, false);
    }

    for i in 0..syncs.len() {
        for v in 0..syncs[i].clone() {
           task_pool.push_sync(v, i as u64, Direction::Back);
        }
    }
    println!("push sync back time{}",  now_millis() - now);

    let now = now_millis();
    for i in 0..async{
        task_pool.push_async(i, i as usize);
    }
    println!("push async back time{}",  now_millis() - now);

    let mut max = async;
    //let mut max = 0;
    for i in 0..syncs.len() {
        max += syncs[i];
    }

    let now = now_millis();
    for _ in 0..max{
        task_pool.pop();
    }
    println!("pop back time{}",  now_millis() - now);
}



#[test]
fn test_async(){
	let task_pool: Arc<TaskPool<u32>> = Arc::new(TaskPool::new());

    for i in 0..5{
        task_pool.create_sync_queue(i as u64, i + 1, false);
    }

    let now = now_millis();
    let count = Arc::new(AtomicUsize::new(0));
    for i in 0..5{
        let task_pool = task_pool.clone();
        let count = count.clone();
        thread::spawn(move || {
            for v in 0..1000 {
                task_pool.push_sync(v, i as u64, Direction::Back);
            }
            count.fetch_add(1, AOrd::Relaxed);
            if count.load(AOrd::Relaxed) == 10 {
                println!("push time{}",  now_millis() - now);
                pop(task_pool.clone());
            }
        });
    }
    for i in 0..5{
        let task_pool = task_pool.clone();
        let count = count.clone();
        thread::spawn(move || {
            for v in 0..1000 {
                let r = v * i;
                task_pool.push_async(r as u32, r + 1);
            }
            count.fetch_add(1, AOrd::Relaxed);
            if count.load(AOrd::Relaxed) == 10 {
                println!("push time{}",  now_millis() - now);
                pop(task_pool.clone());
            }
        });
    }

    thread::sleep(Duration::from_millis(1000));
}

#[cfg(test)]
fn pop (task_pool: Arc<TaskPool<u32>>){
    let now = now_millis();
    let count = Arc::new(AtomicUsize::new(0));
    println!("task_pool len------{:?}", task_pool);
    for _ in 0..10{
        let task_pool = task_pool.clone();
        let count = count.clone();
        thread::spawn(move || {
            for _ in 0..1000 {
                task_pool.pop().unwrap();
            }
            count.fetch_add(1, AOrd::Relaxed);
            if count.load(AOrd::Relaxed) == 10 {
                println!("pop time{}",  now_millis() - now);
                println!("task_pool len------{:?}", task_pool);
            }
        });
    }
}
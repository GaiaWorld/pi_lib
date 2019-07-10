use rand;
use rand::Rng;
use fnv::FnvHashMap;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Result};

use atom::Atom;
use task::{TaskType, Task, TaskCache};

/*
* 同步任务池
*/
struct SyncPool {
    weight:         u64,                                //同步任务池权重
    map:            FnvHashMap<u64, VecDeque<Task>>,    //同步任务队列表
    delay_queue:    VecDeque<Task>,                     //延迟同步任务队列
}

impl Display for SyncPool {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "SyncPool[weight = {}, priority_size = {}, size = {}, delay_size = {}]", 
        self.weight, self.map.len(), self.size(), self.delay_size())
	}
}

impl SyncPool {
    //构建一个同步任务池
    fn new() -> Self {
        SyncPool {
            weight:         0,
            map:            FnvHashMap::default(),
            delay_queue:    VecDeque::new(),
        }
    }

    //获取同步任务数量
    fn size(&self) -> u64 {
        let mut size: u64 = 0;
        for val in self.map.values() {
            size += val.len() as u64;
        }
        size
    }

    //获取延迟同步任务数量
    fn delay_size(&self) -> u64 {
        self.delay_queue.len() as u64
    }

    //从同步任务队列中弹出任务
    fn pop(&mut self, weight: u64, task: &mut Task) -> Option<Task> {
        let mut reply = Option::None;
        let mut w: i64 = weight as i64;
        for (priority, queue) in self.map.iter_mut() {
            w -= (priority * (queue.len() as u64)) as i64;
            if w < 0 {
                self.weight -= priority; //减少同步任务池权重
                match queue.pop_front() {
                    Some(t) => {
                        //填充任务
                        t.copy_to(task);
                        reply = Some(t);
                    },
                    None => (),
                }
                break;
            }
        }
        reply
    }

    //从同步延迟任务队列中弹出任务
    fn delay_pop(&mut self, task: &mut Task) -> Option<Task> {
        match self.delay_queue.pop_front() {
            Some(t) => {
                //填充任务
                t.copy_to(task);
                Some(t)
            },
            None => Option::None,
        }
    }

    //向同步任务队列尾加入任务
    fn push_back(&mut self, task: Task) {
        let priority = task.get_priority() as u64;
        self.weight += priority;
        self.map.entry(priority).or_insert(VecDeque::new()).push_back(task); //获取指定优先级的同步任务队列并加入任务，如果队列为空，则创建一个队列后再加入任务
    }

    //向同步任务队列头加入任务
    fn push_front(&mut self, task: Task) {
        let priority = task.get_priority() as u64;
        self.weight += priority;
        self.map.entry(priority).or_insert(VecDeque::new()).push_front(task); //获取指定优先级的同步任务队列并加入任务，如果队列为空，则创建一个队列后再加入任务
    }

    //向同步延迟任务队列尾加入任务
    fn delay_push_back(&mut self, task: Task) {
        self.delay_queue.push_back(task);
    }

    //向同步延迟任务队列头加入任务
    fn delay_push_front(&mut self, task: Task) {
        self.delay_queue.push_front(task);
    }

    //移除指定优先级的同步任务队列
    fn remove(&mut self, priority: u64) {
       self.map.remove(&(priority as u64));
    }

    //移除同步延迟队列任务
    fn delay_remove(&mut self) {
       self.delay_queue.clear();
    }

    //清空同步任务池
    fn clear(&mut self) {
        self.map.clear();
        self.delay_remove();
    }
}

/*
* 异步任务池
*/
struct AsyncPool {
    weight:         u64,            //异步任务队列权重
    queue:          VecDeque<Task>, //异步任务队列
    delay_queue:    VecDeque<Task>, //延迟异步任务队列
}

impl Display for AsyncPool {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "AsyncPool[weight = {}, size = {}, delay_size = {}]", 
        self.weight, self.size(), self.delay_size())
	}
}

impl AsyncPool {
    //构建一个同步任务池
    fn new() -> Self {
        AsyncPool {
            weight:         0,
            queue:          VecDeque::new(),
            delay_queue:    VecDeque::new(),
        }
    }

    //获取异步任务数量
    fn size(&self) -> u64 {
        self.queue.len() as u64
    }

    //获取延迟异步任务数量
    fn delay_size(&self) -> u64 {
        self.delay_queue.len() as u64
    }

    //从异步任务队列中弹出任务
    fn pop(&mut self, mut weight: u64, task: &mut Task) -> Option<Task> {
        let mut index = -1;
        let mut priority = 0;
        let mut reply = Option::None;
        for i in 0..self.queue.len() {
            match self.queue.get_mut(i) {
                Some(t) => {
                    priority = t.get_priority();
                    if weight < (priority as u64) {
                        //已选中异步任务
                        index = i as isize;
                        break;
                    }
                    //没有选中，则减少权重继续查找下一个任务
                    weight -= priority as u64;
                },
                None => continue,
            }
        }
        if index > -1 {
            self.weight -= priority as u64; //减少异步任务池权重
            match self.queue.remove(index as usize) {
                Some(t) => {
                    //填充任务
                    t.copy_to(task);
                    reply = Some(t);
                },
                None => (),
            }
        }
        reply
    }

    //从异步延迟任务队列中弹出任务
    fn delay_pop(&mut self, task: &mut Task) -> Option<Task> {
        match self.delay_queue.pop_front() {
            Some(t) => {
                //填充任务
                t.copy_to(task);
                Some(t)
            },
            None => Option::None,
        }
    }

    //向异步任务队列尾加入任务
    fn push_back(&mut self, task: Task) {
        let priority = task.get_priority() as u64;
        self.weight += priority;
        self.queue.push_back(task);
    }

    //向异步延迟任务队列尾加入任务
    fn delay_push_back(&mut self, task: Task) {
        self.delay_queue.push_back(task);
    }

    //移除异步任务队列
    pub fn remove(&mut self) {
       self.queue.clear();
    }

    //移除异步延迟队列任务
    pub fn delay_remove(&mut self) {
       self.delay_queue.clear();
    }

    //清空异步任务池
    fn clear(&mut self) {
        self.remove();
        self.delay_remove();
    }
}

/*
* 任务池
*/
pub struct TaskPool {
    task_cache:     TaskCache,  //任务缓存
    sync_pool:      SyncPool,   //同步任务池
    async_pool:     AsyncPool,  //异步任务池
}

unsafe impl Sync for TaskPool {}

impl Display for TaskPool {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "TaskPool[cache_size = {}, sync_pool = {}, async_pool = {}]", 
            self.task_cache.size(), self.sync_pool, self.async_pool)
	}
}

impl TaskPool {
    //构建一个任务池
    pub fn new(len: u32) -> Self {
        TaskPool {
            task_cache: TaskCache::new(len),
            sync_pool:  SyncPool::new(),
            async_pool: AsyncPool::new(),
        }
    }

    //获取任务数量
    pub fn size(&self) -> u64 {
        self.sync_pool.size() + self.sync_pool.delay_size() + self.async_pool.size() + self.async_pool.delay_size()
    }

    //从任务池中弹出一个任务
    pub fn pop(&mut self, task: &mut Task) {
        let mut wait_free: Option<Task>;
        let mut r: u64;
        let mut sw = self.sync_pool.weight;
        let mut aw = self.async_pool.weight;
        let w = sw + aw;
        if w > 0 {
            //判断从同步还是异步任务队列中弹出
            r = rand::thread_rng().gen_range(0, w);
            if r < sw {
                //从同步任务队列中弹出
                wait_free=self.sync_pool.pop(r, task);
                self.free(wait_free);
            } else {
                //从异步任务队列中弹出
                wait_free=self.async_pool.pop(r - sw, task);
                self.free(wait_free);
            }
        }
        sw = self.sync_pool.delay_size();
        aw = self.async_pool.delay_size();
        if sw > 0 {
            if aw > 0 {
                //判断从同步还是异步延迟任务队列中弹出
                r = rand::thread_rng().gen_range(0, w);
                if r < sw {
                    //从同步延迟任务队列中弹出
                    wait_free=self.sync_pool.delay_pop(task);
                    self.free(wait_free);
                } else {
                    //从异步延迟任务队列中弹出
                    wait_free=self.async_pool.delay_pop(task);
                    self.free(wait_free);
                }
            } else {
                //只有从同步延迟任务队列中弹出
                wait_free=self.sync_pool.delay_pop(task);
                self.free(wait_free);
            }
        } else if aw > 0 {
            //只有从异步延迟任务队列中弹出
            wait_free=self.async_pool.delay_pop(task);
            self.free(wait_free);
        }
    }

    //向任务池加入一个任务
    pub fn push(&mut self, task_type: TaskType, priority: u64, func: Box<FnOnce()>, info: Atom) {
        let mut task: Task = self.task_cache.pop();
        task.set_priority(priority);
        task.set_func(Some(func));
        task.set_info(info);
        if priority > 0 {
            match task_type {
                TaskType::Async => {
                    //加入异步任务队列
                    self.async_pool.push_back(task);
                },
                TaskType::Sync => {
                    //加入同步任务队列尾
                    self.sync_pool.push_back(task);
                },
                TaskType::SyncImme => {
                    //加入同步任务队列头
                    self.sync_pool.push_front(task);
                },
                _ => (),
            }
        } else {
            //加入延迟任务队列
            match task_type {
                TaskType::Async => {
                    //加入异步延迟任务队列
                    self.async_pool.delay_push_back(task);
                },
                TaskType::Sync => {
                    //加入同步延迟任务队列尾
                    self.sync_pool.delay_push_back(task);
                },
                TaskType::SyncImme => {
                    //加入同步延迟任务队列头
                    self.sync_pool.delay_push_front(task);
                },
                _ => (),
            }
        }
    }

    //移除指定优先级的同步任务
    pub fn remove_sync_task(&mut self, priority: u64) {
        self.sync_pool.remove(priority);
    }

    //清空所有任务
    pub fn clear(&mut self) {
        self.async_pool.clear();
        self.sync_pool.clear();
    }

    //释放指定任务
    fn free(&mut self, task: Option<Task>) {
        match task {
            Some(t) => self.task_cache.push(t),
            None => (),
        }
    }
}
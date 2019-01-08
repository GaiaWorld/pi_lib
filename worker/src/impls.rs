use std::boxed::FnBox;
use std::sync::{Arc, Mutex, Condvar};

use atom::Atom;
use timer::Timer;
use task_pool::{TaskPool, DelayTask};

use task::{TaskType, Task};

/*
* 任务池定时器
*/
lazy_static! {
    pub static ref TASK_POOL_TIMER: Timer<DelayTask<Task>> = Timer::new(10);
}

/*
* 唤醒者
*/
lazy_static! {
	pub static ref JS_WORKER_WALKER: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));
	pub static ref STORE_WORKER_WALKER: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));
	pub static ref NET_WORKER_WALKER: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));
}

/*
* 虚拟机任务池
*/
lazy_static! {
	pub static ref JS_TASK_POOL: Arc<TaskPool<Task>> = Arc::new(TaskPool::new((*TASK_POOL_TIMER).clone(), Box::new(|| {
	    //唤醒虚拟机工作者
        let &(ref lock, ref cvar) = &**JS_WORKER_WALKER;
        let mut wake = lock.lock().unwrap();
        *wake = true;
        cvar.notify_one();
	})));
}

/*
* 存储任务池
*/
lazy_static! {
	pub static ref STORE_TASK_POOL: Arc<TaskPool<Task>> = Arc::new(TaskPool::new((*TASK_POOL_TIMER).clone(), Box::new(|| {
	    //唤醒存储工作者
        let &(ref lock, ref cvar) = &**STORE_WORKER_WALKER;
        let mut wake = lock.lock().unwrap();
        *wake = true;
        cvar.notify_one();
	})));
}

/*
* 网络任务池
*/
lazy_static! {
	pub static ref NET_TASK_POOL: Arc<TaskPool<Task>> = Arc::new(TaskPool::new((*TASK_POOL_TIMER).clone(), Box::new(|| {
	    //唤醒网络工作者
        let &(ref lock, ref cvar) = &**NET_WORKER_WALKER;
        let mut wake = lock.lock().unwrap();
        *wake = true;
        cvar.notify_one();
	})));
}

/*
* 线程安全的为虚拟机任务池创建队列
*/
pub fn create_js_task_queue(priority: usize, can_del: bool) -> isize {
    create_task_queue(&JS_TASK_POOL, priority, can_del)
}

/*
* 线程安全的锁住虚拟机任务池队列
*/
pub fn lock_js_task_queue(queue: isize) -> bool {
    JS_TASK_POOL.lock_queue(queue)
}

/*
* 线程安全的解锁虚拟机任务池队列
*/
pub fn unlock_js_task_queue(queue: isize) -> bool {
    JS_TASK_POOL.free_queue(queue)
}

/*
* 线程安全的向虚拟机任务池投递任务，返回可移除的任务句柄
*/
pub fn cast_js_task(task_type: TaskType, priority: usize, queue: Option<isize>,
                    func: Box<FnBox(Option<isize>)>, info: Atom) -> Option<isize> {
    cast_task(&JS_TASK_POOL, task_type, priority, queue, func, info)
}

/*
* 线程安全的为虚拟机任务池移除队列
*/
pub fn remove_js_task_queue(queue: isize) -> bool {
    JS_TASK_POOL.delete_queue(queue)
}

/*
* 线程安全的为存储任务池创建队列
*/
pub fn create_store_task_queue(priority: usize, can_del: bool) -> isize {
    create_task_queue(&STORE_TASK_POOL, priority, can_del)
}

/*
* 线程安全的锁住存储任务池队列
*/
pub fn lock_store_task_queue(queue: isize) -> bool {
    STORE_TASK_POOL.lock_queue(queue)
}

/*
* 线程安全的解锁存储任务池队列
*/
pub fn unlock_store_task_queue(queue: isize) -> bool {
    STORE_TASK_POOL.free_queue(queue)
}

/*
* 线程安全的向存储任务池投递任务，返回可移除的任务句柄
*/
pub fn cast_store_task(task_type: TaskType, priority: usize, queue: Option<isize>, func: Box<FnBox(Option<isize>)>, info: Atom) -> Option<isize> {
    cast_task(&STORE_TASK_POOL, task_type, priority, queue, func, info)
}

/*
* 线程安全的为存储任务池移除队列
*/
pub fn remove_store_task_queue(queue: isize) -> bool {
    STORE_TASK_POOL.delete_queue(queue)
}

/*
* 线程安全的为网络任务池创建队列
*/
pub fn create_net_task_queue(priority: usize, can_del: bool) -> isize {
    create_task_queue(&NET_TASK_POOL, priority, can_del)
}

/*
* 线程安全的锁住网络任务池队列
*/
pub fn lock_net_task_queue(queue: isize) -> bool {
    NET_TASK_POOL.lock_queue(queue)
}

/*
* 线程安全的解锁网络任务池队列
*/
pub fn unlock_net_task_queue(queue: isize) -> bool {
    NET_TASK_POOL.free_queue(queue)
}

/*
* 线程安全的向网络任务池投递任务，返回可移除的任务句柄
*/
pub fn cast_net_task(task_type: TaskType, priority: usize, queue: Option<isize>, func: Box<FnBox(Option<isize>)>, info: Atom) -> Option<isize> {
    cast_task(&NET_TASK_POOL, task_type, priority, queue, func, info)
}

/*
* 线程安全的为网络任务池移除队列
*/
pub fn remove_net_task_queue(queue: isize) -> bool {
    NET_TASK_POOL.delete_queue(queue)
}

//创建任务队列
fn create_task_queue(task_pool: &Arc<TaskPool<Task>>, priority: usize, can_del: bool) -> isize {
    if can_del {
        task_pool.create_dyn_queue(priority)
    } else {
        task_pool.create_static_queue(priority)
    }
}

//投递任务
fn cast_task(task_pool: &Arc<TaskPool<Task>>, task_type: TaskType, priority: usize,
             queue: Option<isize>, func: Box<FnBox(Option<isize>)>, info: Atom) -> Option<isize> {
    let mut task = Task::new();
    task.set_priority(priority as u64);
    task.set_func(Some(func));
    task.set_info(info);
    match task_type {
        TaskType::Async(false) => {
            //静态异步任务
            (*task_pool).push_static_async(task, priority);
            None
        },
        TaskType::Async(true) => {
            //动态异步任务
            Some((*task_pool).push_dyn_async(task, priority))
        },
        TaskType::Sync(true) => {
            //同步队列尾
            match queue.unwrap() {
                q if q < 0 => {
                    //静态同步任务
                    (*task_pool).push_static_back(task, q);
                    None
                },
                q => {
                    //动态同步任务
                    Some((*task_pool).push_dyn_back(task, q))
                },
            }
        },
        _ => {
            //同步队列头
            match queue.unwrap() {
                q if q < 0 => {
                    //静态同步任务
                    (*task_pool).push_static_front(task, q);
                    None
                },
                q => {
                    //动态同步任务
                    Some((*task_pool).push_dyn_front(task, q))
                },
            }
        },
    }
}
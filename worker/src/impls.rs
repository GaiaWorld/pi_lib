use std::sync::{Arc, Mutex, Condvar};

use atom::Atom;
use apm::allocator::is_alloced_limit;
use apm::counter::{GLOBAL_PREF_COLLECT, PrefCounter};
use timer::Timer;
use task_pool::{enums::{Direction, QueueType}, TaskPool, DelayTask};

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
	pub static ref JS_TASK_POOL: Arc<TaskPool<Task>> = Arc::new(TaskPool::new((*TASK_POOL_TIMER).clone(), Arc::new(|task_type, _task_size| {
	    match task_type {
	        QueueType::StaticSync if is_alloced_limit() => {
	            //如果是静态同步任务唤醒，且当前已达最大可分配内存限制，则忽略唤醒
	            return;
	        },
	        _ => {
	            //唤醒虚拟机工作者
                let &(ref lock, ref cvar) = &**JS_WORKER_WALKER;
                let mut wake = lock.lock().unwrap();
                *wake = true;
                cvar.notify_one();
	        },
	    }
	})));
}

/*
* 存储任务池
*/
lazy_static! {
	pub static ref STORE_TASK_POOL: Arc<TaskPool<Task>> = Arc::new(TaskPool::new((*TASK_POOL_TIMER).clone(), Arc::new(|task_type, _task_size| {
	    match task_type {
	        QueueType::StaticSync if is_alloced_limit() => {
	            //如果是静态同步任务唤醒，且当前已达最大可分配内存限制，则忽略唤醒
	            return;
	        },
	        _ => {
	            //唤醒存储工作者
                let &(ref lock, ref cvar) = &**STORE_WORKER_WALKER;
                let mut wake = lock.lock().unwrap();
                *wake = true;
                cvar.notify_one();
	        },
	    }
	})));
}

/*
* 网络任务池
*/
lazy_static! {
	pub static ref NET_TASK_POOL: Arc<TaskPool<Task>> = Arc::new(TaskPool::new((*TASK_POOL_TIMER).clone(), Arc::new(|task_type, _task_size| {
	    match task_type {
	        QueueType::StaticSync if is_alloced_limit() => {
	            //如果是静态同步任务唤醒，且当前已达最大可分配内存限制，则忽略唤醒
	            return;
	        },
	        _ => {
	            //唤醒网络工作者
                let &(ref lock, ref cvar) = &**NET_WORKER_WALKER;
                let mut wake = lock.lock().unwrap();
                *wake = true;
                cvar.notify_one();
	        },
	    }
	})));
}

lazy_static! {
    //虚拟机动态队列创建数量
    static ref JS_DYNAMIC_QUEUE_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_dynamic_queue_create_count"), 0).unwrap();
    //虚拟机动态队列移除数量
    static ref JS_DYNAMIC_QUEUE_REMOVE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_dynamic_queue_remove_count"), 0).unwrap();
    //虚拟机静态队列创建数量
    static ref JS_STATIC_QUEUE_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_static_queue_create_count"), 0).unwrap();
    //虚拟机静态队列移除数量
    static ref JS_STATIC_QUEUE_REMOVE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_static_queue_remove_count"), 0).unwrap();
    //存储动态队列创建数量
    static ref STORE_DYNAMIC_QUEUE_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_dynamic_queue_create_count"), 0).unwrap();
    //存储动态队列移除数量
    static ref STORE_DYNAMIC_QUEUE_REMOVE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_dynamic_queue_remove_count"), 0).unwrap();
    //存储静态队列创建数量
    static ref STORE_STATIC_QUEUE_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_static_queue_create_count"), 0).unwrap();
    //存储静态队列移除数量
    static ref STORE_STATIC_QUEUE_REMOVE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_static_queue_remove_count"), 0).unwrap();
    //网络动态队列创建数量
    static ref NET_DYNAMIC_QUEUE_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_dynamic_queue_create_count"), 0).unwrap();
    //网络动态队列移除数量
    static ref NET_DYNAMIC_QUEUE_REMOVE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_dynamic_queue_remove_count"), 0).unwrap();
    //网络静态队列创建数量
    static ref NET_STATIC_QUEUE_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_static_queue_create_count"), 0).unwrap();
    //网络静态队列移除数量
    static ref NET_STATIC_QUEUE_REMOVE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_static_queue_remove_count"), 0).unwrap();
    //虚拟机动态异步任务投递数量
    static ref JS_DYNAMIC_ASYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_dynamic_async_task_cast_count"), 0).unwrap();
    //虚拟机动态同步任务投递数量
    static ref JS_DYNAMIC_SYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_dynamic_sync_task_cast_count"), 0).unwrap();
    //虚拟机静态异步任务投递数量
    static ref JS_STATIC_ASYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_static_async_task_cast_count"), 0).unwrap();
    //虚拟机静态同步任务投递数量
    static ref JS_STATIC_SYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_static_sync_task_cast_count"), 0).unwrap();
    //存储动态异步任务投递数量
    static ref STORE_DYNAMIC_ASYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_dynamic_async_task_cast_count"), 0).unwrap();
    //存储动态同步任务投递数量
    static ref STORE_DYNAMIC_SYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_dynamic_sync_task_cast_count"), 0).unwrap();
    //存储静态异步任务投递数量
    static ref STORE_STATIC_ASYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_static_async_task_cast_count"), 0).unwrap();
    //存储静态同步任务投递数量
    static ref STORE_STATIC_SYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_static_sync_task_cast_count"), 0).unwrap();
    //网络动态异步任务投递数量
    static ref NET_DYNAMIC_ASYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_dynamic_async_task_cast_count"), 0).unwrap();
    //网络动态同步任务投递数量
    static ref NET_DYNAMIC_SYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_dynamic_sync_task_cast_count"), 0).unwrap();
    //网络静态异步任务投递数量
    static ref NET_STATIC_ASYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_static_async_task_cast_count"), 0).unwrap();
    //网络静态同步任务投递数量
    static ref NET_STATIC_SYNC_TASK_CAST_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_static_sync_task_cast_count"), 0).unwrap();
}

/*
* 线程安全的为虚拟机任务池创建队列
*/
pub fn create_js_task_queue(priority: usize, can_del: bool) -> isize {
    if can_del {
        JS_DYNAMIC_QUEUE_CREATE_COUNT.sum(1);
    } else {
        JS_STATIC_QUEUE_CREATE_COUNT.sum(1);
    }

    create_task_queue(&JS_TASK_POOL, priority, can_del)
}

/*
* 线程安全的获取虚拟机静态同步任务数
*/
pub fn js_static_sync_task_size() -> usize {
    JS_TASK_POOL.static_sync_len()
}

/*
* 线程安全的获取虚拟机静态同步任务数
*/
pub fn js_dyn_sync_task_size() -> usize {
    JS_TASK_POOL.dyn_sync_len()
}

/*
* 线程安全的获取虚拟机静态同步任务数
*/
pub fn js_static_async_task_size() -> usize {
    JS_TASK_POOL.static_async_len()
}

/*
* 线程安全的获取虚拟机静态同步任务数
*/
pub fn js_dyn_async_task_size() -> usize {
    JS_TASK_POOL.dyn_async_len()
}

/*
* 线程安全的获取虚拟机任务池任务数
*/
pub fn js_task_size() -> usize {
    JS_TASK_POOL.len()
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
                    func: Box<FnOnce(Option<isize>)>, info: Atom) -> Option<isize> {
    match task_type {
        TaskType::Async(false) => {
            JS_STATIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Async(true) => {
            JS_DYNAMIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Sync(_) => {
            match queue.as_ref().unwrap() {
                q if q < &0 => {
                    JS_STATIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
                q => {
                    JS_DYNAMIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
            }
        },
        _ => (),
    }

    cast_task(&JS_TASK_POOL, task_type, priority, queue, func, info)
}

/*
* 线程安全的向虚拟机任务池投递延迟任务，返回可移除的任务句柄
*/
pub fn cast_js_delay_task(task_type: TaskType, priority: usize, queue: Option<isize>,
                    func: Box<FnOnce(Option<isize>)>, timeout: u32, info: Atom) -> Option<isize> {
    match task_type {
        TaskType::Async(false) => {
            JS_STATIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Async(true) => {
            JS_DYNAMIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Sync(_) => {
            match queue.as_ref().unwrap() {
                q if q < &0 => {
                    JS_STATIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
                q => {
                    JS_DYNAMIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
            }
        },
        _ => (),
    }

    cast_delay_task(&JS_TASK_POOL, task_type, priority, queue, func, timeout, info)
}

/*
* 线程安全的为虚拟机任务池移除队列
*/
pub fn remove_js_task_queue(queue: isize) -> bool {
    if queue < 0 {
        JS_STATIC_QUEUE_REMOVE_COUNT.sum(1);
    } else {
        JS_DYNAMIC_QUEUE_REMOVE_COUNT.sum(1);
    }

    JS_TASK_POOL.delete_queue(queue)
}

/*
* 线程安全的为存储任务池创建队列
*/
pub fn create_store_task_queue(priority: usize, can_del: bool) -> isize {
    if can_del {
        STORE_DYNAMIC_QUEUE_CREATE_COUNT.sum(1);
    } else {
        STORE_STATIC_QUEUE_CREATE_COUNT.sum(1);
    }

    create_task_queue(&STORE_TASK_POOL, priority, can_del)
}

/*
* 线程安全的获取存储任务池任务数
*/
pub fn store_task_size() -> usize {
    STORE_TASK_POOL.len()
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
pub fn cast_store_task(task_type: TaskType, priority: usize, queue: Option<isize>, func: Box<FnOnce(Option<isize>)>, info: Atom) -> Option<isize> {
    match task_type {
        TaskType::Async(false) => {
            STORE_STATIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Async(true) => {
            STORE_DYNAMIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Sync(_) => {
            match queue.as_ref().unwrap() {
                q if q < &0 => {
                    STORE_STATIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
                q => {
                    STORE_DYNAMIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
            }
        },
        _ => (),
    }

    cast_task(&STORE_TASK_POOL, task_type, priority, queue, func, info)
}

/*
* 线程安全的向存储任务池投递延迟任务，返回可移除的任务句柄
*/
pub fn cast_store_delay_task(task_type: TaskType, priority: usize, queue: Option<isize>,
                             func: Box<FnOnce(Option<isize>)>, timeout: u32, info: Atom) -> Option<isize> {
    match task_type {
        TaskType::Async(false) => {
            STORE_STATIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Async(true) => {
            STORE_DYNAMIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Sync(_) => {
            match queue.as_ref().unwrap() {
                q if q < &0 => {
                    STORE_STATIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
                q => {
                    STORE_DYNAMIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
            }
        },
        _ => (),
    }

    cast_delay_task(&STORE_TASK_POOL, task_type, priority, queue, func, timeout, info)
}

/*
* 线程安全的为存储任务池移除队列
*/
pub fn remove_store_task_queue(queue: isize) -> bool {
    if queue < 0 {
        STORE_STATIC_QUEUE_REMOVE_COUNT.sum(1);
    } else {
        STORE_DYNAMIC_QUEUE_REMOVE_COUNT.sum(1);
    }

    STORE_TASK_POOL.delete_queue(queue)
}

/*
* 线程安全的为网络任务池创建队列
*/
pub fn create_net_task_queue(priority: usize, can_del: bool) -> isize {
    if can_del {
        NET_DYNAMIC_QUEUE_CREATE_COUNT.sum(1);
    } else {
        NET_STATIC_QUEUE_CREATE_COUNT.sum(1);
    }

    create_task_queue(&NET_TASK_POOL, priority, can_del)
}

/*
* 线程安全的获取网络任务池任务数
*/
pub fn net_task_size() -> usize {
    NET_TASK_POOL.len()
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
pub fn cast_net_task(task_type: TaskType, priority: usize, queue: Option<isize>, func: Box<FnOnce(Option<isize>)>, info: Atom) -> Option<isize> {
    match task_type {
        TaskType::Async(false) => {
            NET_STATIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Async(true) => {
            NET_DYNAMIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Sync(_) => {
            match queue.as_ref().unwrap() {
                q if q < &0 => {
                    NET_STATIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
                q => {
                    NET_DYNAMIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
            }
        },
        _ => (),
    }

    cast_task(&NET_TASK_POOL, task_type, priority, queue, func, info)
}

/*
* 线程安全的向网络任务池投递延迟任务，返回可移除的任务句柄
*/
pub fn cast_net_delay_task(task_type: TaskType, priority: usize, queue: Option<isize>,
                           func: Box<FnOnce(Option<isize>)>, timeout: u32, info: Atom) -> Option<isize> {
    match task_type {
        TaskType::Async(false) => {
            NET_STATIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Async(true) => {
            NET_DYNAMIC_ASYNC_TASK_CAST_COUNT.sum(1);
        },
        TaskType::Sync(_) => {
            match queue.as_ref().unwrap() {
                q if q < &0 => {
                    NET_STATIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
                q => {
                    NET_DYNAMIC_SYNC_TASK_CAST_COUNT.sum(1);
                },
            }
        },
        _ => (),
    }

    cast_delay_task(&NET_TASK_POOL, task_type, priority, queue, func, timeout, info)
}

/*
* 线程安全的为网络任务池移除队列
*/
pub fn remove_net_task_queue(queue: isize) -> bool {
    if queue < 0 {
        NET_STATIC_QUEUE_REMOVE_COUNT.sum(1);
    } else {
        NET_DYNAMIC_QUEUE_REMOVE_COUNT.sum(1);
    }

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
             queue: Option<isize>, func: Box<FnOnce(Option<isize>)>, info: Atom) -> Option<isize> {
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

//投递延迟任务
fn cast_delay_task(task_pool: &Arc<TaskPool<Task>>, task_type: TaskType, priority: usize,
                   queue: Option<isize>, func: Box<FnOnce(Option<isize>)>, timeout: u32, info: Atom) -> Option<isize> {
    let mut task = Task::new();
    task.set_priority(priority as u64);
    task.set_func(Some(func));
    task.set_info(info);
    match task_type {
        TaskType::Async(false) => {
            //静态异步任务
            None
        },
        TaskType::Async(true) => {
            //动态异步任务
            Some((*task_pool).push_async_delay(task, priority, timeout))
        },
        TaskType::Sync(true) => {
            //同步队列尾
            match queue.unwrap() {
                q if q < 0 => {
                    //静态同步任务
                    None
                },
                q => {
                    //动态同步任务
                    Some((*task_pool).push_sync_delay(task, q, Direction::Back, timeout))
                },
            }
        },
        _ => {
            //同步队列头
            match queue.unwrap() {
                q if q < 0 => {
                    //静态同步任务
                    None
                },
                q => {
                    //动态同步任务
                    Some((*task_pool).push_sync_delay(task, q, Direction::Front, timeout))
                },
            }
        },
    }
}
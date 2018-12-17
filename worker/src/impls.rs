use std::boxed::FnBox;
use std::sync::{Arc, Mutex, Condvar};

use atom::Atom;

use task::TaskType;
use task_pool::TaskPool;

/*
* 虚拟机任务池
*/
lazy_static! {
	pub static ref JS_TASK_POOL: Arc<(Mutex<TaskPool>, Condvar)> = Arc::new((Mutex::new(TaskPool::new(10)), Condvar::new()));
}

/*
* 存储任务池
*/
lazy_static! {
	pub static ref STORE_TASK_POOL: Arc<(Mutex<TaskPool>, Condvar)> = Arc::new((Mutex::new(TaskPool::new(10)), Condvar::new()));
}

/*
* 外部任务池
*/
lazy_static! {
	pub static ref EXT_TASK_POOL: Arc<(Mutex<TaskPool>, Condvar)> = Arc::new((Mutex::new(TaskPool::new(10)), Condvar::new()));
}


/*
* 线程安全的向虚拟机任务池投递任务
*/
pub fn cast_js_task(task_type: TaskType, priority: u64, func: Box<FnBox()>, info: Atom) {
    let &(ref lock, ref cvar) = &**JS_TASK_POOL;
    let mut task_pool = lock.lock().unwrap();
    (*task_pool).push(task_type, priority, func, info);
    cvar.notify_one();
}

/*
* 线程安全的向存储任务池投递任务
*/
pub fn cast_store_task(task_type: TaskType, priority: u64, func: Box<FnBox()>, info: Atom) {
    let &(ref lock, ref cvar) = &**STORE_TASK_POOL;
    let mut task_pool = lock.lock().unwrap();
    (*task_pool).push(task_type, priority, func, info);
    cvar.notify_one();
}

/*
* 线程安全的向外部任务池投递任务
*/
pub fn cast_ext_task(task_type: TaskType, priority: u64, func: Box<FnBox()>, info: Atom) {
    let &(ref lock, ref cvar) = &**EXT_TASK_POOL;
    let mut task_pool = lock.lock().unwrap();
    (*task_pool).push(task_type, priority, func, info);
    cvar.notify_one();
}
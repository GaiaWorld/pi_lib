use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::task::Task;
use npnc::bounded::spsc::{channel as npnc_channel, Producer, Consumer};

use atom::Atom;

use worker::task::TaskType;
use future::FutTask;

///
/// 未来异步任务优先级
///
const FUTURE_ASYNC_TASK_PRIORITY: usize = 100;

///
/// 未来任务池
///
#[derive(Debug)]
pub struct FutTaskPool {
    counter:    AtomicUsize,                                                                                //未来任务计数器
    executor:   fn(TaskType, usize, Option<isize>, Box<dyn FnOnce(Option<isize>)>, Atom) -> Option<isize>,  //未来任务执行器
}

impl Clone for FutTaskPool {
    fn clone(&self) -> Self {
        FutTaskPool {
            counter: AtomicUsize::new(0),
            executor: self.executor,
        }
    }
}

impl FutTaskPool {
    /// 构建一个未来任务池
    pub fn new(executor: fn(TaskType, usize, Option<isize>, Box<dyn FnOnce(Option<isize>)>, Atom) -> Option<isize>) -> Self {
        FutTaskPool {
            counter: AtomicUsize::new(0),
            executor: executor,
        }
    }

    /// 获取当前未来任务计数
    pub fn counte(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }

    /// 分派一个未来任务
    pub fn spawn<T, E>(&self,
        callback: Box<dyn FnOnce(fn(TaskType, usize, Option<isize>, Box<dyn FnOnce(Option<isize>)>, Atom) -> Option<isize>, Arc<Producer<Result<T, E>>>, Arc<Consumer<Task>>, usize)>,
        timeout: u32) -> FutTask<T, E> where T: Send + 'static, E: Send + 'static {
            let uid = self.counter.fetch_add(1, Ordering::Relaxed);
            let (p0, c0) = npnc_channel(1);
            let (p1, c1) = npnc_channel(1);
            let copy = self.executor;
            let func = Box::new(move |_lock| {
                callback(copy, Arc::new(p0), Arc::new(c1), uid);
            });
            (self.executor)(TaskType::Async(false), FUTURE_ASYNC_TASK_PRIORITY, None, func, Atom::from(uid.to_string() + " future task"));
            FutTask::new(uid, timeout, Arc::new(c0), Arc::new(p1))
    }
}


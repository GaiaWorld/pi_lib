//! # 多线程运行时
//!

use std::any::Any;
use std::vec::IntoIter;
use std::future::Future;
use std::mem::transmute;
use std::cell::UnsafeCell;
use std::sync::{Arc, Weak};
use std::marker::PhantomData;
use std::thread::{self, Builder};
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicPtr, Ordering};

use parking_lot::{Mutex, RwLock, Condvar};
use crossbeam_channel::{Sender, bounded, unbounded};
use crossbeam_deque::{Injector, Stealer, Steal, Worker};
use crossbeam_queue::{ArrayQueue, SegQueue};
use flume::bounded as async_bounded;
use futures::{future::{FutureExt, BoxFuture},
              stream::{Stream, StreamExt, BoxStream},
              task::{ArcWake, waker_ref}, TryFuture};
use async_stream::stream;
use num_cpus;
use minstant;
use log::{debug, warn};

use super::{PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME,
            AsyncTaskPool,
            AsyncTaskPoolExt,
            AsyncRuntimeExt,
            TaskId,
            AsyncTask,
            AsyncRuntime,
            AsyncTimingTask,
            AsyncTaskTimer,
            AsyncWaitTimeout,
            AsyncWait,
            AsyncWaitAny,
            AsyncWaitAnyCallback,
            AsyncMapReduce,
            AsyncPipelineResult,
            alloc_rt_uid, current_async_runtime};

/*
* 默认的初始工作者数量
*/
const DEFAULT_INIT_WORKER_SIZE: usize = 2;

/*
* 默认的工作者线程名称前缀
*/
const DEFAULT_WORKER_THREAD_PREFIX: &str = "Default-Multi-RT";

/*
* 默认的线程栈大小
*/
const DEFAULT_THREAD_STACK_SIZE: usize = 1024 * 1024;

/*
* 默认的工作者线程空闲休眠时长，单位ms
*/
const DEFAULT_WORKER_THREAD_SLEEP_TIME: u64 = 10;

/*
* 默认的运行时空闲休眠时长，单位ms，运行时空闲是指绑定当前运行时的队列为空，且定时器内未到期的任务为空
*/
const DEFAULT_RUNTIME_SLEEP_TIME: u64 = 1000;

/*
* 线程唯一id
*/
thread_local! {
    static PI_ASYNC_THREAD_LOCAL_ID: UnsafeCell<usize> = UnsafeCell::new(0);
}

///
/// 计算型的工作者任务队列
///
struct ComputationalTaskQueue<O: Default + 'static> {
    stack:          Worker<Arc<AsyncTask<O, ComputationalTaskPool<O>>>>,   //工作者任务栈
    queue:          SegQueue<Arc<AsyncTask<O, ComputationalTaskPool<O>>>>,  //工作者任务队列
    thread_waker:   Arc<(AtomicBool, Mutex<()>, Condvar)>,                  //工作者线程的唤醒器
}

impl<O: Default + 'static> ComputationalTaskQueue<O> {
    //构建计算型的工作者任务队列
    pub fn new(thread_waker: Arc<(AtomicBool, Mutex<()>, Condvar)>) -> Self {
        let stack = Worker::new_lifo();
        let queue = SegQueue::new();

        ComputationalTaskQueue {
            stack,
            queue,
            thread_waker
        }
    }

    //获取计算型的工作者任务队列的任务数量
    pub fn len(&self) -> usize {
        self.stack.len() + self.queue.len()
    }
}

///
/// 计算型的多线程任务池，适合用于Cpu密集型的应用，不支持运行时伸缩
///
pub struct ComputationalTaskPool<O: Default + 'static> {
    workers:        Vec<ComputationalTaskQueue<O>>,                                     //工作者的任务队列列表
    waits:          Option<Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>>,     //待唤醒的工作者唤醒器队列
    consume_count:  Arc<AtomicUsize>,                                                   //任务消费计数
    produce_count:  Arc<AtomicUsize>,                                                   //任务生产计数
}

unsafe impl<O: Default + 'static> Send for ComputationalTaskPool<O> {}
unsafe impl<O: Default + 'static> Sync for ComputationalTaskPool<O> {}

impl<O: Default + 'static> Default for ComputationalTaskPool<O> {
    fn default() -> Self {
        let core_len = num_cpus::get(); //工作者任务池数据等于本机逻辑核数
        ComputationalTaskPool::new(core_len)
    }
}

impl<O: Default + 'static> AsyncTaskPool<O> for ComputationalTaskPool<O> {
    type Pool = ComputationalTaskPool<O>;

    #[inline]
    fn get_thread_id(&self) -> usize {
        match PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() }
        }) {
            Err(e) => {
                //不应该执行到这个分支
                panic!("Get thread id failed, thread: {:?}, reason: {:?}", thread::current(), e);
            },
            Ok(id) => {
                id
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        if let Some(len) = self
            .produce_count
            .load(Ordering::Relaxed)
            .checked_sub(self.consume_count.load(Ordering::Relaxed)) {
            len
        } else {
            0
        }
    }

    #[inline]
    fn push(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        let index = self.produce_count.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        self.workers[index].queue.push(task);
        Ok(())
    }

    #[inline]
    fn push_timed_out(&self, _index: u64, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        let id = self.get_thread_id();
        let worker = &self.workers[id];
        worker.stack.push(task);
        self.produce_count.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    #[inline]
    fn push_keep(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        self.push(task)
    }

    #[inline]
    fn try_pop(&self) -> Option<Arc<AsyncTask<O, Self::Pool>>> {
        let id = self.get_thread_id();
        let worker = &self.workers[id];
        let task = worker.stack.pop();
        if task.is_some() {
            //指定工作者的任务栈有任务，则立即返回任务
            self.consume_count.fetch_add(1, Ordering::Relaxed);
            return task;
        }

        let task = worker.queue.pop();
        if task.is_some() {
            self.consume_count.fetch_add(1, Ordering::Relaxed);
        }

        task
    }

    #[inline]
    fn try_pop_all(&self) -> IntoIter<Arc<AsyncTask<O, Self::Pool>>> {
        let mut tasks = Vec::with_capacity(self.len());
        while let Some(task) = self.try_pop() {
            tasks.push(task);
        }

        tasks.into_iter()
    }

    #[inline]
    fn get_thread_waker(&self) -> Option<&Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        //多线程任务运行时不支持此方法
        None
    }
}

impl<O: Default + 'static> AsyncTaskPoolExt<O> for ComputationalTaskPool<O> {
    #[inline]
    fn set_waits(&mut self,
                 waits: Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>) {
        self.waits = Some(waits);
    }

    #[inline]
    fn get_waits(&self) -> Option<&Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>> {
        self.waits.as_ref()
    }

    #[inline]
    fn worker_len(&self) -> usize {
        self.workers.len()
    }

    #[inline]
    fn clone_thread_waker(&self) -> Option<Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        let thread_id = self.get_thread_id();
        let worker = &self.workers[thread_id];
        Some(worker.thread_waker.clone())
    }
}

impl<O: Default + 'static> ComputationalTaskPool<O> {
    //构建指定数量的工作者的计算型的多线程任务池
    pub fn new(mut size: usize) -> Self {
        if size < DEFAULT_INIT_WORKER_SIZE {
            //工作者数量过少，则设置为默认的工作者数量
            size = DEFAULT_INIT_WORKER_SIZE;
        }

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let thread_waker = Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()));
            let worker = ComputationalTaskQueue::new(thread_waker);
            workers.push(worker);
        }
        let consume_count = Arc::new(AtomicUsize::new(0));
        let produce_count = Arc::new(AtomicUsize::new(0));

        ComputationalTaskPool {
            workers,
            waits: None,
            consume_count,
            produce_count,
        }
    }
}

///
/// 可窃取的工作者任务队列
///
struct StealableTaskQueue<O: Default + 'static> {
    stack:          Worker<Arc<AsyncTask<O, StealableTaskPool<O>>>>,    //工作者任务栈
    queue:          Worker<Arc<AsyncTask<O, StealableTaskPool<O>>>>,    //工作者任务队列，可窃取
    thread_waker:   Arc<(AtomicBool, Mutex<()>, Condvar)>,              //工作者线程的唤醒器
}

impl<O: Default + 'static> StealableTaskQueue<O> {
    //构建可窃取的工作者任务队列
    pub fn new(thread_waker: Arc<(AtomicBool, Mutex<()>, Condvar)>)
               -> (Self, Stealer<Arc<AsyncTask<O, StealableTaskPool<O>>>>) {
        let stack = Worker::new_lifo();
        let queue = Worker::new_fifo();
        let stealer = queue.stealer();

        (StealableTaskQueue {
            stack,
            queue,
            thread_waker
        }, stealer)
    }

    //获取可窃取的工作者任务队列的任务数量
    pub fn len(&self) -> usize {
        self.stack.len() + self.queue.len()
    }
}

///
/// 可窃取的多线程任务池，适合用于block较多的应用，支持运行时伸缩
///
pub struct StealableTaskPool<O: Default + 'static> {
    public:             Injector<Arc<AsyncTask<O, StealableTaskPool<O>>>>,                          //公共的任务池
    workers:            Vec<Arc<RwLock<Option<StealableTaskQueue<O>>>>>,                            //工作者的任务队列列表
    worker_stealers:    Vec<Arc<RwLock<Option<Stealer<Arc<AsyncTask<O, StealableTaskPool<O>>>>>>>>, //工作者任务队列的窃取者
    frees:              ArrayQueue<usize>,                                                          //可分派的工作者列表偏移
    waits:              Option<Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>>,             //待唤醒的工作者唤醒器队列
    consume_count:      Arc<AtomicUsize>,                                                           //任务消费计数
    produce_count:      Arc<AtomicUsize>,                                                           //任务生产计数
}

unsafe impl<O: Default + 'static> Send for StealableTaskPool<O> {}
unsafe impl<O: Default + 'static> Sync for StealableTaskPool<O> {}

impl<O: Default + 'static> Default for StealableTaskPool<O> {
    fn default() -> Self {
        StealableTaskPool::new()
    }
}

impl<O: Default + 'static> AsyncTaskPool<O> for StealableTaskPool<O> {
    type Pool = StealableTaskPool<O>;

    #[inline]
    fn get_thread_id(&self) -> usize {
        match PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() }
        }) {
            Err(e) => {
                //不应该执行到这个分支
                panic!("Get thread id failed, thread: {:?}, reason: {:?}", thread::current(), e);
            },
            Ok(id) => {
                id
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        if let Some(len) = self
            .produce_count
            .load(Ordering::Relaxed)
            .checked_sub(self.consume_count.load(Ordering::Relaxed)) {
            len
        } else {
            0
        }
    }

    #[inline]
    fn push(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        self.public.push(task);
        self.produce_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    #[inline]
    fn push_timed_out(&self, _index: u64, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        let id = self.get_thread_id();
        if let Some(worker) = &*(&self.workers[id]).read() {
            worker.stack.push(task);
            self.produce_count.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        Err(Error::new(ErrorKind::Other,
                       format!("Push timed out failed, thread id: {}, reason: worker not exists", id)))
    }

    #[inline]
    fn push_keep(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        self.push(task)
    }

    #[inline]
    fn try_pop(&self) -> Option<Arc<AsyncTask<O, Self::Pool>>> {
        let id = self.get_thread_id();
        if let Some(worker) = &*(&self.workers[id]).read() {
            let task = worker.stack.pop();
            if task.is_some() {
                //指定工作者的任务栈有任务，则立即返回任务
                self.consume_count.fetch_add(1, Ordering::Relaxed);
                return task;
            }

            //从指定工作者的任务队列中弹出任务
            let task = worker.queue.pop();
            if task.is_some() {
                //如果工作者有任务，则立即返回
                self.consume_count.fetch_add(1, Ordering::Relaxed);
                return task;
            } else {
                //工作者的任务队列为空，则从公共任务池中窃取所有任务
                loop {
                    match self.public.steal_batch_and_pop(&worker.queue) {
                        Steal::Retry => {
                            //需要重试窃取公共任务池的任务
                            continue;
                        },
                        Steal::Success(task) => {
                            //从已窃取的所有公共任务中获取到首个任务，并立即返回
                            self.consume_count.fetch_add(1, Ordering::Relaxed);
                            return Some(task);
                        },
                        Steal::Empty => {
                            //公共任务池中没有可窃取的任务
                            break;
                        },
                    }
                }

                let mut steal_task = None; //从其它工作者中窃取到的首个任务
                for index in 0..self.worker_stealers.len() {
                    if id == index {
                        //忽略当前工作者的窃取者
                        continue;
                    }

                    if let Some(stealer) = &*self.worker_stealers[index].read() {
                        if stealer.is_empty() {
                            //待窃取的工作者没有可窃取的任务，则继续窃取下一个工作者的任务
                            continue;
                        }

                        loop {
                            match stealer.steal() {
                                Steal::Retry => {
                                    //需要重试窃取指定工作者中的任务
                                    continue;
                                },
                                Steal::Success(task) => {
                                    //从指定工作者中窃取到任务
                                    if steal_task.is_none() {
                                        //当前没有窃取到任何的任务，则设置
                                        self.consume_count.fetch_add(1, Ordering::Relaxed);
                                        steal_task = Some(task);
                                    } else {
                                        //当前已窃取过任务，则加入当前工作者的任务队列中
                                        worker.queue.push(task);
                                    }
                                },
                                Steal::Empty => {
                                    //指定工作者中没有可窃取的任务，则继续窃取下一个工作者的任务
                                    break;
                                }
                            }
                        }
                    }
                }

                return steal_task;
            }
        }

        None
    }

    #[inline]
    fn try_pop_all(&self) -> IntoIter<Arc<AsyncTask<O, Self::Pool>>> {
        let mut tasks = Vec::with_capacity(self.len());
        while let Some(task) = self.try_pop() {
            tasks.push(task);
        }

        tasks.into_iter()
    }

    #[inline]
    fn get_thread_waker(&self) -> Option<&Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        //多线程任务运行时不支持此方法
        None
    }
}

impl<O: Default + 'static> AsyncTaskPoolExt<O> for StealableTaskPool<O> {
    #[inline]
    fn set_waits(&mut self,
                 waits: Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>) {
        self.waits = Some(waits);
    }

    #[inline]
    fn get_waits(&self) -> Option<&Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>> {
        self.waits.as_ref()
    }

    #[inline]
    fn idler_len(&self) -> usize {
        self.frees.len()
    }

    #[inline]
    fn spawn_worker(&self) -> Option<usize> {
        if self.idler_len() == 0 {
            //当前没有可分派的空闲工作者偏移，则立即返回分派失败
            return None;
        }

        if let Some(id) = self.frees.pop() {
            //有可分派的空闲工作者偏移，则在指定偏移处初始化工作者
            if self.workers[id].read().is_some() || self.worker_stealers[id].read().is_some() {
                //当前空闲工作者偏移已分派，则立即返回分派失败
                return None;
            }

            //分派新的工作者任务池和窃取者
            let thread_waker = Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()));
            let (worker, worker_stealer) = StealableTaskQueue::new(thread_waker);
            *self.workers[id].write() = Some(worker);
            *self.worker_stealers[id].write() = Some(worker_stealer);

            return Some(id);
        }

        None
    }

    #[inline]
    fn worker_len(&self) -> usize {
        let mut workers_len = 0usize;
        for worker in &self.workers {
            if worker.read().is_some() {
                workers_len += 1;
            }
        }

        workers_len
            .checked_sub(self.frees.len())
            .or(Some(0))
            .unwrap()
    }

    #[inline]
    fn buffer_len(&self) -> usize {
        self.public.len()
    }

    #[inline]
    fn clone_thread_waker(&self) -> Option<Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        let thread_id = self.get_thread_id();
        if let Some(worker) = &*self.workers[thread_id].read() {
            return Some(worker.thread_waker.clone());
        }

        None
    }

    #[inline]
    fn close_worker(&self) {
        let thread_id = self.get_thread_id();
        let current = thread::current();

        //移除当前线程的工作者任务池
        let _ = self.workers[thread_id].write().take();
        warn!("Remove worker task pool ok, worker: {}, thread: {:?}", thread_id, current);

        //移除当前线程的工作者的任务窃取者
        let _ = self.worker_stealers[thread_id].write().take();
        warn!("Remove worker stealer ok, worker: {}, thread: {:?}", thread_id, current);

        //记录当前被关闭的工作者偏移
        self.frees.push(thread_id);
    }
}

impl<O: Default + 'static> StealableTaskPool<O> {
    /// 构建可窃取的多线程任务池
    pub fn new() -> Self {
        let size = num_cpus::get() * 2; //默认最大工作者任务池数量是当前cpu逻辑核的2倍
        StealableTaskPool::with(DEFAULT_INIT_WORKER_SIZE, size)
    }

    /// 构建指定初始工作者任务池数量和最大工作者任务池数量的可窃取的多线程任务池
    pub fn with(init: usize, max: usize) -> Self {
        if init == 0 || max == 0 || init > max {
            //初始工作者任务池数量或最大工作者任务池数量无效，则立即抛出异常
            panic!("Create MultiTaskPool failed, init: {}, max: {}, reason: invalid init or max", init, max);
        }

        let public = Injector::new();
        let mut workers = Vec::with_capacity(max);
        let mut worker_stealers = Vec::with_capacity(max);
        let frees = ArrayQueue::new(max);
        for _ in 0..init {
            //初始化指定初始作者任务池数量的工作者任务池和窃取者
            let thread_waker = Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()));
            let (worker, worker_stealer) = StealableTaskQueue::new(thread_waker);
            workers.push(Arc::new(RwLock::new(Some(worker))));
            worker_stealers.push(Arc::new(RwLock::new(Some(worker_stealer))));
        }
        for index in init..max {
            //将剩余工作者任务池和窃取者设置为None
            workers.push(Arc::new(RwLock::new(None)));
            worker_stealers.push(Arc::new(RwLock::new(None)));
            frees.push(index); // 将剩余的工作者偏移，记入可分派的工作者列表中
        }
        let consume_count = Arc::new(AtomicUsize::new(0));
        let produce_count = Arc::new(AtomicUsize::new(0));

        StealableTaskPool {
            public,
            workers,
            worker_stealers,
            frees,
            waits: None,
            consume_count,
            produce_count,
        }
    }
}

///
/// 异步多线程任务运行时，支持运行时线程伸缩
///
pub struct MultiTaskRuntime<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O> = StealableTaskPool<O>
>(Arc<(
    usize,                                                                                      //运行时唯一id
    Arc<P>,                                                                                     //异步任务池
    Option<Vec<(Sender<(usize, AsyncTimingTask<O, P>)>, Arc<Mutex<AsyncTaskTimer<O, P>>>)>>,    //休眠的异步任务生产者和本地定时器
    AtomicUsize,                                                                                //定时任务计数器
    Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>,                                     //待唤醒的工作者唤醒器队列
    (String, usize, usize, u64, Option<usize>),                                                 //当前运行时配置参数
)>);

unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Send for MultiTaskRuntime<O, P> {}
unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Sync for MultiTaskRuntime<O, P> {}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Clone for MultiTaskRuntime<O, P> {
    fn clone(&self) -> Self {
        MultiTaskRuntime(self.0.clone())
    }
}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> AsyncRuntimeExt<O> for MultiTaskRuntime<O, P> {
    fn spawn_with_context<F, C>(&self,
                                task_id: TaskId,
                                future: F,
                                context: C) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static {
        let boxed = Box::new(future).boxed();
        let task = Arc::new(AsyncTask::with_context(
            task_id,
            (self.0).1.clone(),
            Some(boxed),
            context));
        let result = (self.0).1.push(task);

        if let Some(worker_waker) = (self.0).4.pop() {
            //有待唤醒的工作者
            let (is_sleep, lock, condvar) = &*worker_waker;
            let locked = lock.lock();
            if is_sleep.load(Ordering::Relaxed) {
                //待唤醒的工作者，正在休眠，则立即唤醒此工作者
                is_sleep.store(false, Ordering::SeqCst); //设置为未休眠
                condvar.notify_one();
            }
        } else {
            //没有待唤醒的工作者，则检查缓冲区内任务数量是否过多
            let busy_size = (self.0).1.buffer_len();
            if busy_size > 100 {
                //当前运行时繁忙，且缓冲区任务过多，则分派新的工作者
                if let Some(index) = (self.0).1.spawn_worker() {
                    //分派新的工作者任务池成功，则分派新的工作者线程
                    let builder = Builder::new()
                        .name(((self.0).5).0.clone() + "-" + index.to_string().as_str())
                        .stack_size(((self.0).5).2);
                    if let Some(timers) = &(self.0).2 {
                        //分派一个指定定时器的工作者线程
                        let (_, timer) = &timers[index];
                        spawn_worker_thread(builder,
                                            index,
                                            self.clone(),
                                            ((self.0).5).1,
                                            ((self.0).5).3,
                                            ((self.0).5).4,
                                            Some(timer.clone()));
                    } else {
                        //分派一个没有定时器的工作者线程
                        spawn_worker_thread(builder,
                                            index,
                                            self.clone(),
                                            ((self.0).5).1,
                                            ((self.0).5).3,
                                            ((self.0).5).4,
                                            None);
                    }
                }
            }
        }

        result
    }

    fn spawn_timing_with_context<F, C>(&self,
                                       task_id: TaskId,
                                       future: F,
                                       context: C,
                                       time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static {
        if let Some(timers) = &(self.0).2 {
            let mut index: usize = (self.0).3.fetch_add(1, Ordering::Relaxed) % timers.len(); //随机选择一个线程的队列和定时器
            let (_, timer) = &timers[index];
            let boxed = Box::new(future).boxed();
            timer
                .lock()
                .set_timer(AsyncTimingTask::WaitRun(Arc::new(AsyncTask::with_context(task_id,
                                                                                     (self.0).1.clone(),
                                                                                     Some(boxed),
                                                                                     context))),
                           time); //为定时器设置定时异步任务

            return Ok(());
        }

        Err(Error::new(ErrorKind::Other, format!("Spawn timing task failed, task_id: {:?}, reason: timer not exist", task_id)))
    }

    fn block_on<RP, F>(&self, future: F) -> Result<F::Output>
        where RP: AsyncTaskPoolExt<F::Output> + AsyncTaskPool<F::Output, Pool = RP>,
              F: Future + Send + 'static,
              <F as Future>::Output: Default + Send + 'static {
        //从本地线程获取当前异步运行时
        if let Some(current) = current_async_runtime::<F::Output, RP>() {
            //本地线程绑定了异步运行时
            if current.get_id() == self.get_id() {
                //如果是相同运行时，则立即返回错误
                return Err(Error::new(ErrorKind::WouldBlock, format!("Block on failed, reason: would block")));
            }
        }

        let (sender, receiver) = bounded(1);
        if let Err(e) = self.spawn(self.alloc(), async move {
            //在指定运行时中执行，并返回结果
            let r = future.await;
            sender.send(r);

            Default::default()
        }) {
            return Err(Error::new(ErrorKind::Other, format!("Block on failed, reason: {:?}", e)));
        }

        //同步阻塞等待异步任务返回
        match receiver.recv() {
            Err(e) => {
                Err(Error::new(ErrorKind::Other, format!("Block on failed, reason: {:?}", e)))
            },
            Ok(result) => {
                Ok(result)
            },
        }
    }
}

/*
* 异步多线程任务运行时同步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> MultiTaskRuntime<O, P> {
    /// 共享运行时内部任务池
    pub(crate) fn shared_pool(&self) -> Arc<P> {
        (self.0).1.clone()
    }

    /// 获取当前运行时的唯一id
    pub fn get_id(&self) -> usize {
        (self.0).0
    }

    /// 获取当前运行时可新增的工作者数量
    fn idler_len(&self) -> usize {
        (self.0).1.idler_len()
    }

    /// 获取当前运行时的工作者数量
    pub fn worker_len(&self) -> usize {
        (self.0).1.worker_len()
    }

    /// 获取当前运行时任务数量
    pub fn len(&self) -> usize {
        (self.0).1.len()
    }

    /// 获取当前运行时中剩余未到期的定时任务数量
    pub fn timing_len(&self) -> usize {
        let mut len = 0;
        if let Some(vec) = &(self.0).2 {
            for (_, timer) in vec.iter() {
                len += timer.lock().len();
            }
        }

        len
    }

    /// 获取当前运行时指定工作者的剩余未到期的定时任务数量
    pub fn worker_timing_len(&self, index: usize) -> usize {
        if let Some(vec) = &(self.0).2 {
            let (_, timer) = &vec[index];
            return timer.lock().len();
        }

        0
    }

    /// 获取当前运行时缓冲区的任务数量，缓冲区的任务暂时没有分配给工作者
    pub fn buffer_len(&self) -> usize {
        (self.0).1.buffer_len()
    }

    /// 分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        TaskId(Arc::new(AtomicUsize::new(0)))
    }

    /// 派发一个指定的异步任务到异步多线程运行时
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let task = Arc::new(AsyncTask::new(task_id,
                                           (self.0).1.clone(),
                                           Some(Box::new(future).boxed())));
        let result = (self.0).1.push(task);

        if let Some(worker_waker) = (self.0).4.pop() {
            //有待唤醒的工作者
            let (is_sleep, lock, condvar) = &*worker_waker;
            let locked = lock.lock();
            if is_sleep.load(Ordering::Relaxed) {
                //待唤醒的工作者，正在休眠，则立即唤醒此工作者
                is_sleep.store(false, Ordering::SeqCst); //设置为未休眠
                condvar.notify_one();
            }
        } else {
            //没有待唤醒的工作者，则检查缓冲区内任务数量是否过多
            let busy_size = (self.0).1.buffer_len();
            if busy_size > 100 {
                //当前运行时繁忙，且缓冲区任务过多，则分派新的工作者
                if let Some(index) = (self.0).1.spawn_worker() {
                    //分派新的工作者任务池成功，则分派新的工作者线程
                    let builder = Builder::new()
                        .name(((self.0).5).0.clone() + "-" + index.to_string().as_str())
                        .stack_size(((self.0).5).2);
                    if let Some(timers) = &(self.0).2 {
                        //分派一个指定定时器的工作者线程
                        let (_, timer) = &timers[index];
                        spawn_worker_thread(builder,
                                            index,
                                            self.clone(),
                                            ((self.0).5).1,
                                            ((self.0).5).3,
                                            ((self.0).5).4,
                                            Some(timer.clone()));
                    } else {
                        //分派一个没有定时器的工作者线程
                        spawn_worker_thread(builder,
                                            index,
                                            self.clone(),
                                            ((self.0).5).1,
                                            ((self.0).5).3,
                                            ((self.0).5).4,
                                            None);
                    }
                }
            }
        }

        result
    }

    /// 派发一个在指定时间后执行的异步任务到异步多线程运行时，时间单位ms
    pub fn spawn_timing<F>(&self, task_id: TaskId, future: F, time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        if let Some(timers) = &(self.0).2 {
            let mut index: usize = (self.0).3.fetch_add(1, Ordering::Relaxed) % timers.len(); //随机选择一个线程的队列和定时器
            let (_, timer) = &timers[index];
            timer
                .lock()
                .set_timer(AsyncTimingTask::WaitRun(Arc::new(AsyncTask::new(task_id,
                                                                            (self.0).1.clone(),
                                                                            Some(Box::new(future).boxed())))),
                           time); //为定时器设置定时异步任务

            return Ok(());
        }

        Err(Error::new(ErrorKind::Other, format!("Spawn timing task failed, task_id: {:?}, reason: timer not exist", task_id)))
    }

    /// 挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: &TaskId, waker: Waker) -> Poll<Output> {
        task_id.0.store(Box::into_raw(Box::new(waker)) as usize, Ordering::Relaxed);
        Poll::Pending
    }

    /// 唤醒执行指定唯一id的异步任务
    pub fn wakeup(&self, task_id: &TaskId) {
        match task_id.0.load(Ordering::Relaxed) {
            0 => panic!("Multi runtime wakeup task failed, reason: task id not exist"),
            ptr => {
                unsafe {
                    let waker = Box::from_raw(ptr as *mut Waker);
                    waker.wake();
                }
            },
        }
    }

    /// 构建用于派发多个异步任务到指定运行时的映射归并，需要指定映射归并的容量
    pub fn map_reduce<V: Send + 'static>(&self, capacity: usize) -> AsyncMapReduce<V> {
        let (producor, consumer) = async_bounded(capacity);

        AsyncMapReduce {
            count: 0,
            capacity,
            producor,
            consumer,
        }
    }

    /// 生成一个异步管道，输入指定流，输入流的每个值通过过滤器生成输出流的值
    pub fn pipeline<S, SO, F, FO>(&self, input: S, mut filter: F) -> BoxStream<'static, FO>
        where S: Stream<Item = SO> + Send + 'static,
              SO: Send + 'static,
              F: FnMut(SO) -> AsyncPipelineResult<FO> + Send + 'static,
              FO: Send + 'static {
        let output = stream! {
            for await value in input {
                match filter(value) {
                    AsyncPipelineResult::Disconnect => {
                        //立即中止管道
                        break;
                    },
                    AsyncPipelineResult::Filtered(result) => {
                        yield result;
                    },
                }
            }
        };

        output.boxed()
    }
}

/*
* 异步多线程任务运行时异步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> MultiTaskRuntime<O, P> {
    /// 挂起当前多线程运行时的当前任务，等待指定的时间后唤醒当前任务
    pub async fn wait_timeout(&self, timeout: usize) {
        if let Some(timers) = &(self.0).2 {
            //有本地定时器，则异步等待指定时间
            match PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
                //将休眠的异步任务投递到当前派发线程的定时器内
                let thread_id = unsafe { *thread_id.get() };
                timers[thread_id].clone()
            }) {
                Err(_) => (),
                Ok((producor, _)) => {
                    AsyncWaitTimeout::new(AsyncRuntime::Multi(self.clone()), producor, timeout).await;
                },
            }
        } else {
            //没有本地定时器，则同步休眠指定时间
            thread::sleep(Duration::from_millis(timeout as u64));
        }
    }

    /// 挂起当前多线程运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, RP, V, F>(&self, rt: AsyncRuntime<R, RP>, future: F) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        AsyncWait::new(AsyncRuntime::Multi(self.clone()), rt, Some(Box::new(future).boxed())).await
    }

    /// 挂起当前多线程运行时的当前任务，并在多个其它运行时上执行多个其它任务，其中任意一个任务完成，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成的任务的值将被忽略
    pub async fn wait_any<R, RP, V>(&self,
                                    futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static {
        AsyncWaitAny::new(AsyncRuntime::Multi(self.clone()), futures).await
    }

    /// 挂起当前多线程运行时的当前任务，并在多个其它运行时上执行多个其它任务，任务返回后需要通过用户指定的检查回调进行检查，其中任意一个任务检查通过，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成或未检查通过的任务的值将被忽略，如果所有任务都未检查通过，则强制唤醒当前运行时的当前任务
    pub async fn wait_any_callback<R, RP, V, F>(&self,
                                                futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>,
                                                callback: F) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static,
              F: Fn(&Result<V>) -> bool + Send + Sync + 'static {
        AsyncWaitAnyCallback::new(AsyncRuntime::Multi(self.clone()), futures, Some(callback)).await
    }
}

///
/// 异步多线程任务运行时构建器
///
pub struct MultiTaskRuntimeBuilder<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O> = StealableTaskPool<O>
> {
    pool:       P,              //异步多线程任务运行时
    prefix:     String,         //工作者线程名称前缀
    init:       usize,          //初始工作者数量
    min:        usize,          //最少工作者数量
    max:        usize,          //最大工作者数量
    stack_size: usize,          //工作者线程栈大小
    timeout:    u64,            //工作者空闲时最长休眠时间
    interval:   Option<usize>,  //工作者定时器间隔
    marker:     PhantomData<O>,
}

unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Send for MultiTaskRuntimeBuilder<O, P> {}
unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Sync for MultiTaskRuntimeBuilder<O, P> {}

impl<O: Default + 'static> Default for MultiTaskRuntimeBuilder<O> {
    //默认构建可窃取可伸缩的多线程运行时
    fn default() -> Self {
        let core_len = num_cpus::get(); //默认的工作者的数量为本机逻辑核数
        let pool = StealableTaskPool::with(core_len, core_len);
        MultiTaskRuntimeBuilder::new(pool)
            .thread_stack_size(2 * 1024 * 1024)
            .set_timer_interval(1)
    }
}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> MultiTaskRuntimeBuilder<O, P> {
    /// 构建指定任务池、线程名前缀、初始线程数量、最少线程数量、最大线程数量、线程栈大小、线程空闲时最长休眠时间和是否使用本地定时器的多线程任务池
    pub fn new(mut pool: P) -> Self {
        let core_len = num_cpus::get(); //获取本机cpu逻辑核数

        MultiTaskRuntimeBuilder {
            pool,
            prefix: DEFAULT_WORKER_THREAD_PREFIX.to_string(),
            init: core_len,
            min: core_len,
            max: core_len,
            stack_size: DEFAULT_THREAD_STACK_SIZE,
            timeout: DEFAULT_WORKER_THREAD_SLEEP_TIME,
            interval: None,
            marker: PhantomData,
        }
    }

    /// 设置工作者线程名称前缀
    pub fn thread_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// 设置工作者线程栈大小
    pub fn thread_stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// 设置初始工作者数量
    pub fn init_worker_size(mut self, mut init: usize) -> Self {
        if init == 0 {
            //初始线程数量过小，则设置默认的初始线程数量
            init = DEFAULT_INIT_WORKER_SIZE;
        }

        self.init = init;
        self
    }

    /// 设置最小工作者数量和最大工作者数量
    pub fn set_worker_limit(mut self, mut min: usize, mut max: usize) -> Self {
        if self.init > max {
            //初始线程数量大于最大线程数量，则设置最大线程数量为初始线程数量
            max = self.init;
        }

        if min == 0 || min > max {
            //最少线程数量无效，则设置最少线程数量为最大线程数量
            min = max;
        }

        self.min = min;
        self.max = max;
        self
    }

    /// 设置工作者空闲时最大休眠时长
    pub fn set_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置工作者定时器间隔
    pub fn set_timer_interval(mut self, interval: usize) -> Self {
        self.interval = Some(interval);
        self
    }

    /// 构建并启动多线程异步运行时
    pub fn build(mut self) -> MultiTaskRuntime<O, P> {
        //构建多线程任务运行时的本地定时器和定时异步任务生产者
        let interval = self.interval;
        let mut timers = if let Some(_) = interval {
            Some(Vec::with_capacity(self.max))
        } else {
            None
        };
        for _ in 0..self.max {
            //初始化指定的最大线程数量的本地定时器和定时异步任务生产者，定时器不会在关闭工作者时被移除
            if let Some(vec) = &mut timers {
                let timer = AsyncTaskTimer::with_interval(interval.unwrap());
                let producor = timer.producor.clone();
                let timer = Arc::new(Mutex::new(timer));
                vec.push((producor, timer));
            };
        }

        //构建多线程任务运行时
        let rt_uid = alloc_rt_uid();
        let waits = Arc::new(ArrayQueue::new(self.max));
        let mut pool = self.pool;
        pool.set_waits(waits.clone()); //设置待唤醒的工作者唤醒器队列
        let pool = Arc::new(pool);
        let runtime = MultiTaskRuntime(Arc::new((
            rt_uid,
            pool,
            timers,
            AtomicUsize::new(0),
            waits,
            (self.prefix.clone(), self.min, self.stack_size, self.timeout, self.interval),
        )));

        //构建初始化线程数量的线程构建器
        let mut builders = Vec::with_capacity(self.init);
        for index in 0..self.init {
            let builder = Builder::new()
                .name(self.prefix.clone() + "-" + index.to_string().as_str())
                .stack_size(self.stack_size);
            builders.push(builder);
        }

        //启动工作者线程
        let min = self.min;
        for index in 0..builders.len() {
            let builder = builders.remove(0);
            let runtime = runtime.clone();
            let timeout = self.timeout;
            let timer = if let Some(timers) = &(runtime.0).2 {
                let (_, timer) = &timers[index];
                Some(timer.clone())
            } else {
                None
            };

            spawn_worker_thread(builder, index, runtime, min, timeout, interval, timer);
        }

        runtime
    }
}

//分派工作者线程，并开始工作
fn spawn_worker_thread<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
>(builder: Builder,
  index: usize,
  runtime: MultiTaskRuntime<O, P>,
  min: usize,
  timeout: u64,
  interval: Option<usize>,
  timer: Option<Arc<Mutex<AsyncTaskTimer<O, P>>>>) {
    if let Some(timer) = timer {
        //设置了定时器
        let _ = builder.spawn(move || {
            //设置线程本地唯一id
            if let Err(e) = PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
                unsafe { *thread_id.get() = index; }
            }) {
                panic!("Multi thread runtime startup failed, thread id: {:?}, reason: {:?}", index, e);
            }

            //绑定运行时到线程
            let runtime_copy = runtime.clone();
            match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME.try_with(move |rt| {
                unsafe { *rt.get() = transmute(Arc::new(AsyncRuntime::Multi(runtime_copy)) as Arc<dyn Any>); }
            }) {
                Err(e) => {
                    panic!("Bind multi runtime to local thread failed, reason: {:?}", e);
                },
                Ok(_) => (),
            }

            //执行有定时器的工作循环
            timer_work_loop(runtime,
                            index,
                            min,
                            timeout,
                            interval.unwrap() as u64,
                            timer);
        });
    } else {
        //未设置定时器
        let _ = builder.spawn(move || {
            //设置线程本地唯一id
            if let Err(e) = PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
                unsafe { *thread_id.get() = index; }
            }) {
                panic!("Multi thread runtime startup failed, thread id: {:?}, reason: {:?}", index, e);
            }

            //绑定运行时到线程
            let runtime_copy = runtime.clone();
            match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME.try_with(move |rt| {
                unsafe { *rt.get() = transmute(Arc::new(AsyncRuntime::Multi(runtime_copy)) as Arc<dyn Any>); }
            }) {
                Err(e) => {
                    panic!("Bind multi runtime to local thread failed, reason: {:?}", e);
                },
                Ok(_) => (),
            }

            //执行无定时器的工作循环
            work_loop(runtime, index, min, timeout);
        });
    }
}

//线程工作循环
fn timer_work_loop<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
>(runtime: MultiTaskRuntime<O, P>,
  index: usize,
  min: usize,
  sleep_timeout: u64,
  timer_interval: u64,
  timer: Arc<Mutex<AsyncTaskTimer<O, P>>>) {
    //初始化当前线程的线程id和线程活动状态
    let pool = (runtime.0).1.clone();
    let worker_waker = pool.clone_thread_waker().unwrap();

    let mut sleep_count = 0; //连续休眠计数器
    loop {
        //设置新的定时异步任务，并唤醒已到期的定时异步任务
        let mut timer_run_millis = minstant::Instant::now(); //重置定时器运行时长
        timer.lock().consume(); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
        loop {
            let current_time = timer.lock().is_require_pop(); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
            if let Some(current_time) = current_time {
                //当前有到期的定时异步任务，则开始处理到期的所有定时异步任务
                loop {
                    let timed_out = timer.lock().pop(current_time); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
                    if let Some((handle, timing_task)) = timed_out {
                        match timing_task {
                            AsyncTimingTask::Pended(expired) => {
                                //唤醒休眠的异步任务，不需要立即在本工作者中执行，因为休眠的异步任务无法取消
                                runtime.wakeup(&expired);
                            },
                            AsyncTimingTask::WaitRun(expired) => {
                                //执行到期的定时异步任务，需要立即在本工作者中执行，因为定时异步任务可以取消
                                (runtime.0).1.push_timed_out(handle as u64, expired);
                                if let Some(task) = pool.try_pop() {
                                    sleep_count = 0; //重置连续休眠次数
                                    run_task(&runtime, task);
                                }
                            },
                        }

                        if let Some(task) = pool.try_pop() {
                            //执行当前工作者任务池中的异步任务，避免定时异步任务占用当前工作者的所有工作时间
                            sleep_count = 0; //重置连续休眠次数
                            run_task(&runtime, task);
                        }
                    } else {
                        //当前所有的到期任务已处理完，则退出本次定时异步任务处理
                        break;
                    }
                }
            } else {
                //当前没有到期的定时异步任务，则退出本次定时异步任务处理
                break;
            }
        }

        //继续执行当前工作者任务池中的异步任务
        match pool.try_pop() {
            None => {
                if runtime.len() > 0 {
                    //确认当前还有任务需要处理，可能还没分配到当前工作者，则当前工作者继续工作
                    continue;
                }

                //无任务，则准备休眠
                if sleep_count > 2 {
                    //连续休眠次数达到或超过3次，则检查是否可以关闭当前工作者
                    if is_closeable(&runtime, min) {
                        //当前工作者空闲，且当前运行时空闲，则立即关闭当前工作者
                        break;
                    }

                    //不允许关闭当前工作者，则重置连续休眠计数器
                    sleep_count = 0;
                }

                {
                    let (is_sleep, lock, condvar) = &*worker_waker;
                    let mut locked = lock.lock();

                    //设置当前为休眠状态
                    is_sleep.store(true, Ordering::SeqCst);

                    //获取休眠的实际时长
                    let diff_time =  minstant::Instant::now()
                        .duration_since(timer_run_millis)
                        .as_millis() as u64; //获取定时器运行时长
                    let real_timeout = if timer.lock().len() == 0 {
                        //当前定时器没有未到期的任务，则休眠指定时长
                        sleep_timeout
                    } else {
                        //当前定时器还有未到期的任务，则计算需要休眠的时长
                        if diff_time >= timer_interval {
                            //定时器内部时间与当前时间差距过大，则忽略休眠，并继续工作
                            continue;
                        } else {
                            //定时器内部时间与当前时间差距不大，则休眠差值时间
                            timer_interval - diff_time
                        }
                    };

                    //记录待唤醒的工作者唤醒器，用于有新任务时唤醒对应的工作者
                    (runtime.0).4.push(worker_waker.clone());

                    //让当前工作者休眠，等待有任务时被唤醒或超时后自动唤醒
                    if condvar
                        .wait_for(
                            &mut locked,
                            Duration::from_millis(real_timeout),
                        )
                        .timed_out()
                    {
                        //条件超时唤醒，则设置状态为未休眠
                        is_sleep.store(false, Ordering::SeqCst);
                        //记录连续休眠次数，因为任务导致的唤醒不会计数
                        sleep_count += 1;
                    }
                }
            },
            Some(task) => {
                //有任务，则执行
                sleep_count = 0; //重置连续休眠次数
                run_task(&runtime, task);
            },
        }
    }

    //关闭当前工作者的任务池
    (runtime.0).1.close_worker();
    warn!("Worker of runtime closed, runtime: {}, worker: {}, thread: {:?}",
          runtime.get_id(),
          index,
          thread::current());
}

//线程工作循环
fn work_loop<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
>(runtime: MultiTaskRuntime<O, P>, index: usize, min: usize, sleep_timeout: u64) {
    //初始化当前线程的线程id和线程活动状态
    let pool = (runtime.0).1.clone();
    let worker_waker = pool.clone_thread_waker().unwrap();

    let mut sleep_count = 0; //连续休眠计数器
    loop {
        match pool.try_pop() {
            None => {
                //无任务，则准备休眠
                if runtime.len() > 0 {
                    //确认当前还有任务需要处理，可能还没分配到当前工作者，则当前工作者继续工作
                    continue;
                }

                if sleep_count > 2 {
                    //连续休眠次数达到或超过3次，则检查是否可以关闭当前工作者
                    if is_closeable(&runtime, min) {
                        //当前工作者空闲，且当前运行时空闲，则立即关闭当前工作者
                        break;
                    }

                    //不允许关闭当前工作者，则重置连续休眠计数器
                    sleep_count = 0;
                }

                {
                    let (is_sleep, lock, condvar) = &*worker_waker;
                    let mut locked = lock.lock();

                    //设置当前为休眠状态
                    is_sleep.store(true, Ordering::SeqCst);

                    //记录待唤醒的工作者唤醒器，用于有新任务时唤醒对应的工作者
                    (runtime.0).4.push(worker_waker.clone());

                    //让当前工作者休眠，等待有任务时被唤醒或超时后自动唤醒
                    if condvar
                        .wait_for(
                            &mut locked,
                            Duration::from_millis(sleep_timeout),
                        )
                        .timed_out()
                    {
                        //条件超时唤醒，则设置状态为未休眠
                        is_sleep.store(false, Ordering::SeqCst);
                        //记录连续休眠次数，因为任务导致的唤醒不会计数
                        sleep_count += 1;
                    }
                }
            },
            Some(task) => {
                //有任务，则执行
                sleep_count = 0; //重置连续休眠次数
                run_task(&runtime, task);
            },
        }
    }

    //关闭当前工作者的任务池
    (runtime.0).1.close_worker();
    warn!("Worker of runtime closed, runtime: {}, worker: {}, thread: {:?}",
          runtime.get_id(),
          index,
          thread::current());
}

//检查是否可以关闭当前工作者
#[inline]
fn is_closeable<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
>(runtime: &MultiTaskRuntime<O, P>, min: usize) -> bool {
    match PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
        unsafe { *thread_id.get() }
    }) {
        Err(_) => {
            //如果本地线程唯一id不存在，则立即关闭当前工作者
            true
        },
        Ok(thread_id) => {
            if runtime.worker_timing_len(thread_id) > 0 {
                //当前工作者还有未处理的定时任务，则不允许关闭
                false
            } else {
                //当前工作者没有未处理的定时任务
                if runtime.worker_len() <= min {
                    //当前工作者过少，则不允许关闭
                    false
                } else {
                    if runtime.buffer_len() > 0 {
                        //当前缓冲区还有任务未处理，则不允许关闭
                        false
                    } else {
                        //当前运行时空闲，则允许关闭
                        true
                    }
                }
            }
        },
    }
}

//执行异步任务
#[inline]
fn run_task<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
>(runtime: &MultiTaskRuntime<O, P>, task: Arc<AsyncTask<O, P>>) {
    let waker = waker_ref(&task);
    let mut context = Context::from_waker(&*waker);
    if let Some(mut future) = task.get_inner() {
        if let Poll::Pending = future.as_mut().poll(&mut context) {
            //当前未准备好，则恢复异步任务，以保证异步服务后续访问异步任务和异步任务不被提前释放
            task.set_inner(Some(future));
        }
    } else {
        //当前异步任务在唤醒时还未被重置内部任务，则继续加入当前异步运行时队列，并等待下次被执行
        (runtime.0).1.push(task);
    }
}

#[test]
fn test_mutli_task_pool() {
    use std::time::Instant;

    let pool = Arc::new(StealableTaskPool::with(8, 8));
    println!("!!!!!!pool len: {}", pool.len());

    let pool0 = pool.clone();
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let pool4 = pool.clone();
    let pool5 = pool.clone();
    let pool6 = pool.clone();
    let pool7 = pool.clone();

    let pool00 = pool.clone();
    let pool01 = pool.clone();
    let pool02 = pool.clone();
    let pool03 = pool.clone();
    let pool04 = pool.clone();
    let pool05 = pool.clone();
    let pool06 = pool.clone();
    let pool07 = pool.clone();

    let start = Instant::now();

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool0.clone(),
                Some(async move {}.boxed()));
            pool0.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool1.clone(),
                Some(async move {}.boxed()));
            pool1.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool2.clone(),
                Some(async move {}.boxed()));
            pool2.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool3.clone(),
                Some(async move {}.boxed()));
            pool3.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool4.clone(),
                Some(async move {}.boxed()));
            pool4.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool5.clone(),
                Some(async move {}.boxed()));
            pool5.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool6.clone(),
                Some(async move {}.boxed()));
            pool6.push(Arc::new(task));
        }
    });

    thread::spawn(move || {
        for _ in 0..2000000 {
            let task = AsyncTask::new(
                TaskId(Arc::new(AtomicUsize::new(0))),
                pool7.clone(),
                Some(async move {}.boxed()));
            pool7.push(Arc::new(task));
        }
    });

    let join0 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 0; }
        });

        let mut count = 0;
        loop {
            if let None = pool00.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool00.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool00 count: {}", count);
    });

    let join1 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 1; }
        });

        let mut count = 0;
        loop {
            if let None = pool01.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool01.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool01 count: {}", count);
    });

    let join2 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 2; }
        });

        let mut count = 0;
        loop {
            if let None = pool02.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool02.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool02 count: {}", count);
    });

    let join3 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 3; }
        });

        let mut count = 0;
        loop {
            if let None = pool03.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool03.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool03 count: {}", count);
    });

    let join4 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 4; }
        });

        let mut count = 0;
        loop {
            if let None = pool04.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool04.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool04 count: {}", count);
    });

    let join5 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 5; }
        });

        let mut count = 0;
        loop {
            if let None = pool05.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool05.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool05 count: {}", count);
    });

    let join6 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 6; }
        });

        let mut count = 0;
        loop {
            if let None = pool06.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool06.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool06 count: {}", count);
    });

    let join7 = thread::spawn(move || {
        PI_ASYNC_THREAD_LOCAL_ID.try_with(move |thread_id| {
            unsafe { *thread_id.get() = 7; }
        });

        let mut count = 0;
        loop {
            if let None = pool07.try_pop() {
                thread::sleep(Duration::from_millis(10));
                if pool07.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
            count += 1;
        }
        println!("!!!!!!pool07 count: {}", count);
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("pool len: {}, time: {:?}", pool.len(), Instant::now() - start);
}

#[test]
fn test_computational_runtime() {
    use std::mem;
    use std::time::Instant;
    use env_logger;
    use crate::rt::{spawn_local,
                    get_local_dict,
                    get_local_dict_mut,
                    set_local_dict,
                    remove_local_dict,
                    clear_local_dict};

    env_logger::init();

    struct AtomicCounter(AtomicUsize, Instant);
    impl Drop for AtomicCounter {
        fn drop(&mut self) {
            unsafe {
                println!("!!!!!!drop counter, count: {:?}, time: {:?}", self.0.load(Ordering::Relaxed), Instant::now() - self.1);
            }
        }
    }

    let pool = ComputationalTaskPool::new(8);
    let builer = MultiTaskRuntimeBuilder::new(pool)
        .set_timer_interval(1);
    let rt = builer.build();
    let rt0 = rt.clone();
    let rt1 = rt.clone();
    let rt2 = rt.clone();
    let rt3 = rt.clone();
    let rt4 = rt.clone();
    let rt5 = rt.clone();
    let rt6 = rt.clone();
    let rt7 = rt.clone();

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let count0 = counter.clone();
    let count1 = counter.clone();
    let count2 = counter.clone();
    let count3 = counter.clone();
    let count4 = counter.clone();
    let count5 = counter.clone();
    let count6 = counter.clone();
    let count7 = counter.clone();
    mem::drop(counter);

    rt.spawn(rt.alloc(), async move {
        use crate::rt::spawn_local;

        if let Err(e) = spawn_local::<ComputationalTaskPool<()>, _>(async move {
            println!("Test spawn local ok");
        }) {
            println!("Test spawn local failed, reason: {:?}", e);
        }
    });

    let rt_copy = rt.clone();
    let thread_handle = thread::spawn(move || {
        match rt_copy.block_on::<ComputationalTaskPool<String>, _>(async move {
            set_local_dict::<usize>(0);
            println!("get local dict, init value: {}", *get_local_dict::<usize>().unwrap());
            *get_local_dict_mut::<usize>().unwrap() = 0xffffffff;
            println!("get local dict, value after modify: {}", *get_local_dict::<usize>().unwrap());
            if let Some(value) = remove_local_dict::<usize>() {
                println!("get local dict, value after remove: {:?}, last value: {}", get_local_dict::<usize>(), value);
            }
            set_local_dict::<usize>(0);
            clear_local_dict();
            println!("get local dict, value after clear: {:?}", get_local_dict::<usize>());

            "Test block on ok".to_string()
        }) {
            Err(e) => {
                println!("Test block on failed, reason: {:?}", e);
            },
            Ok(r) => {
                println!("{}", r);
            },
        }
    });
    thread_handle.join();

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count0.clone();
            if let Err(e) = rt0.spawn(rt0.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count1.clone();
            if let Err(e) = rt1.spawn(rt1.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count2.clone();
            if let Err(e) = rt2.spawn(rt2.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count3.clone();
            if let Err(e) = rt3.spawn(rt3.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count4.clone();
            if let Err(e) = rt4.spawn(rt4.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count5.clone();
            if let Err(e) = rt5.spawn(rt5.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count6.clone();
            if let Err(e) = rt6.spawn(rt6.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count7.clone();
            if let Err(e) = rt7.spawn(rt7.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::sleep(Duration::from_millis(1000000000));
}

#[test]
fn test_stealable_runtime() {
    use std::mem;
    use std::time::Instant;
    use env_logger;
    use crate::rt::{spawn_local,
                    get_local_dict,
                    get_local_dict_mut,
                    set_local_dict,
                    remove_local_dict,
                    clear_local_dict};

    env_logger::init();

    struct AtomicCounter(AtomicUsize, Instant);
    impl Drop for AtomicCounter {
        fn drop(&mut self) {
            unsafe {
                println!("!!!!!!drop counter, count: {:?}, time: {:?}", self.0.load(Ordering::Relaxed), Instant::now() - self.1);
            }
        }
    }

    let pool = StealableTaskPool::with(8, 8);
    let builer = MultiTaskRuntimeBuilder::new(pool)
        .set_timer_interval(1);
    let rt = builer.build();
    let rt0 = rt.clone();
    let rt1 = rt.clone();
    let rt2 = rt.clone();
    let rt3 = rt.clone();
    let rt4 = rt.clone();
    let rt5 = rt.clone();
    let rt6 = rt.clone();
    let rt7 = rt.clone();

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let count0 = counter.clone();
    let count1 = counter.clone();
    let count2 = counter.clone();
    let count3 = counter.clone();
    let count4 = counter.clone();
    let count5 = counter.clone();
    let count6 = counter.clone();
    let count7 = counter.clone();
    mem::drop(counter);

    rt.spawn(rt.alloc(), async move {
        use crate::rt::spawn_local;

        if let Err(e) = spawn_local::<StealableTaskPool<()>, _>(async move {
            println!("Test spawn local ok");
        }) {
            println!("Test spawn local failed, reason: {:?}", e);
        }
    });

    let rt_copy = rt.clone();
    let thread_handle = thread::spawn(move || {
        match rt_copy.block_on::<StealableTaskPool<String>, _>(async move {
            set_local_dict::<usize>(0);
            println!("get local dict, init value: {}", *get_local_dict::<usize>().unwrap());
            *get_local_dict_mut::<usize>().unwrap() = 0xffffffff;
            println!("get local dict, value after modify: {}", *get_local_dict::<usize>().unwrap());
            if let Some(value) = remove_local_dict::<usize>() {
                println!("get local dict, value after remove: {:?}, last value: {}", get_local_dict::<usize>(), value);
            }
            set_local_dict::<usize>(0);
            clear_local_dict();
            println!("get local dict, value after clear: {:?}", get_local_dict::<usize>());

            "Test block on ok".to_string()
        }) {
            Err(e) => {
                println!("Test block on failed, reason: {:?}", e);
            },
            Ok(r) => {
                println!("{}", r);
            },
        }
    });
    thread_handle.join();

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count0.clone();
            if let Err(e) = rt0.spawn(rt0.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count1.clone();
            if let Err(e) = rt1.spawn(rt1.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count2.clone();
            if let Err(e) = rt2.spawn(rt2.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count3.clone();
            if let Err(e) = rt3.spawn(rt3.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count4.clone();
            if let Err(e) = rt4.spawn(rt4.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count5.clone();
            if let Err(e) = rt5.spawn(rt5.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count6.clone();
            if let Err(e) = rt6.spawn(rt6.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2000000 {
            let counter_copy = count7.clone();
            if let Err(e) = rt7.spawn(rt7.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn multi task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn multi task ok, time: {:?}", Instant::now() - start);
    });

    thread::sleep(Duration::from_millis(1000000000));
}














use std::thread;
use std::pin::Pin;
use std::sync::Arc;
use std::cell::RefCell;
use std::future::Future;
use std::time::Duration;
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

pub mod single_thread;
pub mod multi_thread;

use futures::{future::BoxFuture, FutureExt};
use parking_lot::{Mutex, Condvar};
use crossbeam_channel::{Sender, Receiver, unbounded};

use local_timer::LocalTimer;

use single_thread::{SingleTask, SingleTaskRuntime};
use multi_thread::{MultiTask, MultiTaskRuntime};

use crate::lock::spin;

/*
* 异步运行时唯一id生成器
*/
static RUNTIME_UID_GEN: AtomicUsize = AtomicUsize::new(1);

/*
* 分配异步运行时唯一id
*/
pub fn alloc_rt_uid() -> usize {
    RUNTIME_UID_GEN.fetch_add(1, Ordering::Relaxed)
}

/*
* 异步任务唯一id
*/
#[derive(Clone)]
pub struct TaskId(Arc<AtomicUsize>);

impl Debug for TaskId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "TaskId[inner = {}]", self.0.load(Ordering::Relaxed))
    }
}

/*
* 异步运行时
*/
pub enum AsyncRuntime<O: Default + 'static> {
    Local(SingleTaskRuntime<O>),                                                            //本地运行时
    Multi(MultiTaskRuntime<O>),                                                             //多线程运行时
    Worker(Arc<AtomicBool>, Arc<(AtomicBool, Mutex<()>, Condvar)>, SingleTaskRuntime<O>),   //工作者运行时
}

unsafe impl<O: Default + 'static> Send for AsyncRuntime<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncRuntime<O> {}

impl<O: Default + 'static> Clone for AsyncRuntime<O> {
    fn clone(&self) -> Self {
        match self {
            AsyncRuntime::Local(rt) => AsyncRuntime::Local(rt.clone()),
            AsyncRuntime::Multi(rt) => AsyncRuntime::Multi(rt.clone()),
            AsyncRuntime::Worker(wroker_status, worker_waker, rt) => AsyncRuntime::Worker(wroker_status.clone(), worker_waker.clone(), rt.clone()),
        }
    }
}

/*
* 异步运行时同步方法
*/
impl<O: Default + 'static> AsyncRuntime<O> {
    //获取当前异步运行时的唯一id
    pub fn get_id(&self) -> usize {
        match self {
            AsyncRuntime::Local(rt) => rt.get_id(),
            AsyncRuntime::Multi(rt) => rt.get_id(),
            AsyncRuntime::Worker(_, _, rt) => rt.get_id(),
        }
    }

    //获取当前异步运行时待处理任务数量
    pub fn wait_len(&self) -> usize {
        match self {
            AsyncRuntime::Local(rt) => rt.wait_len(),
            AsyncRuntime::Multi(rt) => rt.wait_len(),
            AsyncRuntime::Worker(_, _, rt) => rt.wait_len(),
        }
    }

    //获取当前异步运行时任务数量
    pub fn len(&self) -> usize {
        match self {
            AsyncRuntime::Local(rt) => rt.len(),
            AsyncRuntime::Multi(rt) => rt.len(),
            AsyncRuntime::Worker(_, _, rt) => rt.len(),
        }
    }

    //分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        match self {
            AsyncRuntime::Local(rt) => rt.alloc(),
            AsyncRuntime::Multi(rt) => rt.alloc(),
            AsyncRuntime::Worker(_, _, rt) => rt.alloc(),
        }
    }

    //派发一个指定的异步任务到异步运行时
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.spawn(task_id, future),
            AsyncRuntime::Multi(rt) => rt.spawn(task_id, future),
            AsyncRuntime::Worker(worker_status, worker_waker, rt) => {
                if !worker_status.load(Ordering::SeqCst) {
                    return Err(Error::new(ErrorKind::Other, "Spawn async task failed, reason: worker already closed"));
                }

                let result = rt.spawn(task_id, future);
                wakeup_worker_thread(worker_waker, rt);
                result
            },
        }
    }

    //派发一个在指定时间后执行的异步任务到异步运行时，返回定时异步任务的句柄，可以在到期之前使用句柄取消异步任务的执行，时间单位ms
    pub fn spawn_timing<F>(&self, task_id: TaskId, future: F, time: usize) -> Result<usize>
        where F: Future<Output = O> + Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.spawn_timing(task_id, future, time),
            AsyncRuntime::Multi(rt) => {
                match rt.spawn_timing(task_id, future, time) {
                    Err(e) => Err(e),
                    Ok(handle) => Ok(handle as usize),
                }
            },
            AsyncRuntime::Worker(worker_status, worker_waker, rt) => {
                if !worker_status.load(Ordering::SeqCst) {
                    return Err(Error::new(ErrorKind::Other, "Spawn timing async task failed, reason: worker already closed"));
                }

                let result = rt.spawn_timing(task_id, future, time);
                wakeup_worker_thread(worker_waker, rt);
                result
            },
        }
    }

    //取消指定句柄的定时异步任务
    pub fn cancel_timing(&self, handle: usize) {
        match self {
            AsyncRuntime::Local(rt) => rt.cancel_timing(handle),
            AsyncRuntime::Multi(rt) => rt.cancel_timing(handle as u64),
            AsyncRuntime::Worker(_, _, rt) => rt.cancel_timing(handle),
        }
    }

    //挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: &TaskId, waker: Waker) -> Poll<Output> {
        match self {
            AsyncRuntime::Local(rt) => rt.pending(task_id, waker),
            AsyncRuntime::Multi(rt) => rt.pending(task_id, waker),
            AsyncRuntime::Worker(_, _, rt) => rt.pending(task_id, waker),
        }
    }

    //唤醒指定唯一id的异步任务
    pub fn wakeup(&self, task_id: &TaskId) {
        match self {
            AsyncRuntime::Local(rt) => rt.wakeup(task_id),
            AsyncRuntime::Multi(rt) => rt.wakeup(task_id),
            AsyncRuntime::Worker(_, worker_waker, rt) => {
                rt.wakeup(task_id);
                wakeup_worker_thread(worker_waker, rt);
            },
        }
    }

    //构建用于派发多个异步任务到指定运行时的映射
    pub fn map<V: Send + 'static>(&self) -> AsyncMap<O, V> {
        match self {
            AsyncRuntime::Local(rt) => rt.map(),
            AsyncRuntime::Multi(rt) => rt.map(),
            AsyncRuntime::Worker(_, _, rt) => rt.map(),
        }
    }

    //关闭异步运行时，返回请求关闭是否成功
    pub fn close(&self) -> bool {
        match self {
            AsyncRuntime::Worker(worker_status, worker_waker, rt) => {
                if worker_status.compare_and_swap(true, false, Ordering::SeqCst) {
                    //设置工作者状态成功，检查运行时所在线程是否需要唤醒
                    wakeup_worker_thread(worker_waker, rt);
                }

                true
            },
            _ => false,
        }
    }
}

/*
* 异步运行时同步方法
*/
impl<O: Default + 'static> AsyncRuntime<O> {
    //挂起当前异步运行时的当前任务，等待指定的时间后唤醒当前任务
    pub async fn wait_timeout(&self, timeout: usize) {
        match self {
            AsyncRuntime::Local(rt) => rt.wait_timeout(timeout).await,
            AsyncRuntime::Multi(rt) => rt.wait_timeout(timeout).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait_timeout(timeout).await,
        }
    }

    //挂起当前异步运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, V, F>(&self, art: AsyncRuntime<R>, future: F) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.wait(art, future).await,
            AsyncRuntime::Multi(rt) => rt.wait(art, future).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait(art, future).await,
        }
    }

    //挂起当前异步运行时的当前任务，并在多个其它运行时上执行多个其它任务，其中任意一个任务完成，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成的任务的值将被忽略
    pub async fn wait_any<R, V>(&self, futures: Vec<(AsyncRuntime<R>, BoxFuture<'static, Result<V>>)>) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static  {
        match self {
            AsyncRuntime::Local(rt) => rt.wait_any(futures).await,
            AsyncRuntime::Multi(rt) => rt.wait_any(futures).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait_any(futures).await,
        }
    }
}

/*
* 异步值
*/
pub struct AsyncValue<O: Default + 'static, V: Send + 'static> {
    rt:         AsyncRuntime<O>,            //异步值的运行时
    task_id:    TaskId,                     //异步值的任务唯一id
    value:      Arc<RefCell<Option<V>>>,    //值
    status:     Arc<AtomicU8>,              //值状态
}

unsafe impl<O: Default + 'static, V: Send + 'static> Send for AsyncValue<O, V> {}
unsafe impl<O: Default + 'static, V: Send + 'static> Sync for AsyncValue<O, V> {}

impl<O: Default + 'static, V: Send + 'static> Clone for AsyncValue<O, V> {
    fn clone(&self) -> Self {
        AsyncValue {
            rt: self.rt.clone(),
            task_id: self.task_id.clone(),
            value: self.value.clone(),
            status: self.status.clone(),
        }
    }
}

impl<O: Default + 'static, V: Send + 'static> Future for AsyncValue<O, V> {
    type Output = V;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = (&self).value.borrow_mut().take() {
            //异步值已就绪
            return Poll::Ready(value);
        }

        let r = self.rt.pending(&self.task_id, cx.waker().clone());
        (&self).status.store(1, Ordering::Relaxed); //将异步值状态设置为就绪
        r
    }
}

impl<O: Default + 'static, V: Send + 'static> AsyncValue<O, V> {
    //构建异步值，默认值为未就绪
    pub fn new(rt: AsyncRuntime<O>) -> Self {
        let task_id = rt.alloc();

        AsyncValue {
            rt,
            task_id,
            value: Arc::new(RefCell::new(None)),
            status: Arc::new(AtomicU8::new(0)),
        }
    }

    //设置异步值
    pub fn set(self, value: V) {
        let mut spin_len = 1;
        loop {
            match self.status.compare_exchange(1,
                                               2,
                                               Ordering::Acquire,
                                               Ordering::Relaxed) {
                Err(0) => {
                    //还未就绪，则自旋等待
                    spin_len = spin(spin_len);
                },
                Err(_) => {
                    //已求值，则忽略
                    return;
                },
                Ok(_) => {
                    //已就绪，则开始求值
                    break;
                },
            }
        }

        //设置后立即释放可写引用，防止唤醒时出现冲突
        {
            *self.value.borrow_mut() = Some(value);
        }

        //唤醒异步值
        self.rt.wakeup(&self.task_id);
    }
}

/*
* 等待异步任务运行的结果
*/
pub struct AsyncWaitResult<V: Send + 'static>(Arc<RefCell<Option<Result<V>>>>);

unsafe impl<V: Send + 'static> Send for AsyncWaitResult<V> {}
unsafe impl<V: Send + 'static> Sync for AsyncWaitResult<V> {}

impl<V: Send + 'static> Clone for AsyncWaitResult<V> {
    fn clone(&self) -> Self {
        AsyncWaitResult(self.0.clone())
    }
}

/*
* 等待异步任务运行的结果集
*/
pub struct AsyncWaitResults<V: Send + 'static>(Arc<RefCell<Option<Vec<Result<V>>>>>);

unsafe impl<V: Send + 'static> Send for AsyncWaitResults<V> {}
unsafe impl<V: Send + 'static> Sync for AsyncWaitResults<V> {}

impl<V: Send + 'static> Clone for AsyncWaitResults<V> {
    fn clone(&self) -> Self {
        AsyncWaitResults(self.0.clone())
    }
}

/*
* 等待执行的定时任务
*/
pub enum WaitRunTask<O: Default + 'static> {
    SingleTask(Arc<SingleTask<O>>), //单线程定时任务
    MultiTask(Arc<MultiTask<O>>),   //多线程定时任务
}

/*
* 异步定时器任务
*/
pub enum AsyncTimingTask<O: Default + 'static> {
    Pended(TaskId),             //已挂起的定时任务
    WaitRun(WaitRunTask<O>),    //等待执行的定时任务
}

/*
* 异步任务本地定时器
*/
pub struct AsyncTaskTimer<O: Default + 'static> {
    producor:   Sender<(usize, AsyncTimingTask<O>)>,            //定时任务生产者
    consumer:   Receiver<(usize, AsyncTimingTask<O>)>,          //定时任务消费者
    timer:      Arc<RefCell<LocalTimer<AsyncTimingTask<O>>>>,   //定时器
}

unsafe impl<O: Default + 'static> Send for AsyncTaskTimer<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncTaskTimer<O> {}

impl<O: Default + 'static> AsyncTaskTimer<O> {
    //构建异步任务本地定时器
    pub fn new() -> Self {
        let (producor, consumer) = unbounded();
        AsyncTaskTimer {
            producor,
            consumer,
            timer: Arc::new(RefCell::new(LocalTimer::new())),
        }
    }

    //构建指定间隔的异步任务本地定时器
    pub fn with_interval(time: usize) -> Self {
        let (producor, consumer) = unbounded();
        AsyncTaskTimer {
            producor,
            consumer,
            timer: Arc::new(RefCell::new(LocalTimer::with_tick(time))),
        }
    }

    //获取定时任务生产者
    pub fn get_producor(&self) -> Sender<(usize, AsyncTimingTask<O>)> {
        self.producor.clone()
    }

    //获取剩余未到期的定时器任务数量
    pub fn len(&self) -> usize {
        self.timer.as_ref().borrow().len()
    }

    //设置定时器
    pub fn set_timer(&self, task: AsyncTimingTask<O>, timeout: usize) -> usize {
        self.timer.borrow_mut().set_timeout(task, timeout)
    }

    //取消定时器
    pub fn cancel_timer(&self, timer_ref: usize) -> Option<AsyncTimingTask<O>> {
        self.timer.borrow_mut().cancel(timer_ref)
    }

    //消费所有定时任务，返回定时任务数量
    pub fn consume(&self) -> usize {
        let mut len = 0;
        let timer_tasks = self.consumer.try_iter().collect::<Vec<(usize, AsyncTimingTask<O>)>>();
        for (timeout, task) in timer_tasks {
            self.set_timer(task, timeout);
            len += 1;
        }

        len
    }

    //轮询定时器
    pub fn poll(&self) -> u64 {
        self.timer.borrow_mut().try_poll()
    }

    //从定时器中弹出到期的一个任务
    pub fn pop(&self) -> Option<AsyncTimingTask<O>> {
        self.timer.borrow_mut().try_pop()
    }

    //清空定时器
    pub fn clear(&self) {
        self.timer.borrow_mut().clear();
    }
}

/*
* 等待指定超时
*/
pub struct AsyncWaitTimeout<O: Default + 'static> {
    rt:         AsyncRuntime<O>,                        //当前运行时
    producor:   Sender<(usize, AsyncTimingTask<O>)>,    //超时请求生产者
    timeout:    usize,                                  //超时时长，单位ms
    expired:    bool,                                   //是否已过期
}

unsafe impl<O: Default + 'static> Send for AsyncWaitTimeout<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncWaitTimeout<O> {}

impl<O: Default + 'static> Future for AsyncWaitTimeout<O> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if (&self).expired {
            //已到期，则返回
            return Poll::Ready(());
        } else {
            //未到期，则设置为已到期
            (&mut self).expired = true;
        }

        let task_id = self.rt.alloc();
        let reply = self.rt.pending(&task_id, cx.waker().clone());

        //发送超时请求，并返回
        (&self).producor.send(((&self).timeout, AsyncTimingTask::Pended(task_id)));
        reply
    }
}

impl<O: Default + 'static> AsyncWaitTimeout<O> {
    //构建等待指定超时任务的方法
    pub fn new(rt: AsyncRuntime<O>,
               producor: Sender<(usize, AsyncTimingTask<O>)>,
               timeout: usize) -> Self {
        AsyncWaitTimeout {
            rt,
            producor,
            timeout,
            expired: false, //设置初始值
        }
    }
}

/*
* 等待异步任务执行完成
*/
pub struct AsyncWait<O: Default + 'static, R: Default + 'static, V: Send + 'static> {
    wait:   AsyncRuntime<O>,                        //需要等待的异步运行时
    runner: AsyncRuntime<R>,                        //需要运行的异步运行时
    future: Option<BoxFuture<'static, Result<V>>>,  //需要等待执行的异步任务
    result: AsyncWaitResult<V>,                     //需要等待执行的异步任务的结果
}

unsafe impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Send for AsyncWait<O, R, V> {}
unsafe impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Sync for AsyncWait<O, R, V> {}

impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Future for AsyncWait<O, R, V> {
    type Output = Result<V>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = (&self).result.0.borrow_mut().take() {
            //任务已完成，则返回
            return Poll::Ready(result);
        }

        //在指定运行时运行指定的任务
        let task_id = self.wait.alloc();
        let task_id_ = task_id.clone();
        let wait = (&self).wait.clone();
        let runner = (&self).runner.clone();
        let future = (&mut self).future.take();
        let result = (&self).result.clone();
        let task = async move {
            let task_id_copy = task_id_.clone();
            let wait_copy = wait.clone();
            let result_copy = result.clone();

            //将指定任务派发到运行时
            if let Err(e) = runner.spawn(runner.alloc(), async move {
                if let Some(f) = future {
                    //指定了任务
                    *result_copy.0.borrow_mut() = Some(f.await);
                } else {
                    //未指定任务
                    *result_copy.0.borrow_mut() = Some(Err(Error::new(ErrorKind::NotFound, "invalid future")));
                }

                wait_copy.wakeup(&task_id_copy);

                //返回异步任务的默认值
                Default::default()
            }) {
                //派发指定的任务失败，则立即唤醒等待的任务
                *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Runner Error by Wait, reason: {:?}", e))));
                wait.wakeup(&task_id_);
            }

            //返回异步任务的默认值
            Default::default()
        };

        //挂起当前异步等待任务，并返回值未就绪，以保证异步等待任务在执行前不会被唤醒
        let reply = self.wait.pending(&task_id, cx.waker().clone());
        if let Err(e) = self.wait.spawn(self.wait.alloc(), task) {
            //派发异步等待的任务失败，则移除已挂起的异步等待任务，并立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Error, reason: {:?}", e))));
        }

        reply
    }
}

impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> AsyncWait<O, R, V> {
    //构建等待异步任务执行完成的方法
    fn new(wait: AsyncRuntime<O>,
           runner: AsyncRuntime<R>,
           future: Option<BoxFuture<'static, Result<V>>>) -> Self {
        AsyncWait {
            wait,
            runner,
            future,
            result: AsyncWaitResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 等待任意异步任务执行完成
*/
pub struct AsyncWaitAny<O: Default + 'static, R: Default + 'static, V: Send + 'static> {
    wait:       AsyncRuntime<O>,                                                //需要等待的异步运行时
    futures:    Option<Vec<(AsyncRuntime<R>, BoxFuture<'static, Result<V>>)>>,  //需要运行的异步运行时和等待执行的异步任务
    is_finish:  Arc<AtomicBool>,                                                //是否有任意的异步任务已执行完成
    result:     AsyncWaitResult<V>,                                             //需要等待执行的异步任务的结果
}

unsafe impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Send for AsyncWaitAny<O, R, V> {}
unsafe impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Sync for AsyncWaitAny<O, R, V> {}

impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Future for AsyncWaitAny<O, R, V> {
    type Output = Result<V>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = (&self).result.0.borrow_mut().take() {
            //任务已完成，则返回
            return Poll::Ready(result);
        }

        //在指定运行时运行指定的任务
        let task_id = self.wait.alloc();
        let task_id_ = task_id.clone();
        let wait = (&self).wait.clone();
        let mut futures = (&mut self).futures.take().unwrap();
        let is_finish = (&self).is_finish.clone();
        let result = (&self).result.clone();
        let task = async move {
            while let Some((runner, future)) = futures.pop() {
                let task_id_copy = task_id_.clone();
                let wait_copy = wait.clone();
                let is_finish_copy = is_finish.clone();
                let result_copy = result.clone();

                //将指定任务派发到本地运行时
                if let Err(e) = runner.spawn(runner.alloc(), async move {
                    if is_finish_copy.load(Ordering::Relaxed) {
                        //快速检查，当前已有任务执行完成，则忽略，并立即返回
                        return Default::default();
                    }

                    //执行任务，并检查是否由当前任务唤醒等待的任务
                    let r = future.await;
                    if !is_finish_copy.compare_and_swap(false, true, Ordering::SeqCst) {
                        //当前没有任务执行完成，则立即唤醒等待的任务
                        *result_copy.0.borrow_mut() = Some(r);
                        wait_copy.wakeup(&task_id_copy);
                    }

                    //返回异步任务的默认值
                    Default::default()
                }) {
                    //派发指定的任务失败，则退出派发循环
                    if !is_finish.compare_and_swap(false, true, Ordering::SeqCst) {
                        //当前没有任务执行完成，则立即唤醒等待的任务
                        *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Task Runner Error, reason: {:?}", e))));
                        wait.wakeup(&task_id_);
                    }
                    break;
                }
            }

            //返回异步任务的默认值
            Default::default()
        };

        //挂起当前异步等待任务，并返回值未就绪，以保证异步等待任务在执行前不会被唤醒
        let reply = self.wait.pending(&task_id, cx.waker().clone());
        if let Err(e) = self.wait.spawn(self.wait.alloc(), task) {
            //派发异步等待的任务失败，则移除已挂起的异步等待任务，并立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Any Error, reason: {:?}", e))));
        }

        reply
    }
}

impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> AsyncWaitAny<O, R, V> {
    //构建等待异步任务执行完成的方法
    fn new(wait: AsyncRuntime<O>,
           futures: Vec<(AsyncRuntime<R>, BoxFuture<'static, Result<V>>)>) -> Self {
        AsyncWaitAny {
            wait,
            futures: Some(futures),
            is_finish: Arc::new(AtomicBool::new(false)),
            result: AsyncWaitResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

/*
* 异步映射，用于将多个任务派发到多个异步运行时
*/
pub struct AsyncMap<O: Default + 'static, V: Send + 'static> {
    count:      usize,                                                  //派发的任务数量
    futures:    Vec<(AsyncRuntime<O>, BoxFuture<'static, Result<V>>)>,  //待派发任务
    producor:   Sender<(usize, Result<V>)>,                             //异步返回值生成器
    consumer:   Receiver<(usize, Result<V>)>,                           //异步返回值接收器
}

unsafe impl<O: Default + 'static, V: Send + 'static> Send for AsyncMap<O, V> {}

impl<O: Default + 'static, V: Send + 'static> AsyncMap<O, V> {
    //加入需要映射的任务，并返回任务序号
    pub fn join<F>(&mut self, rt: AsyncRuntime<O>, future: F) -> usize
        where F: Future<Output = Result<V>> + Send + 'static {
        let index = self.count;
        self.futures.push((rt, Box::new(future).boxed()));
        self.count += 1;
        index
    }

    //映射所有任务，并返回指定异步运行时的异步归并
    pub fn map(self, wait: AsyncRuntime<O>) -> AsyncReduce<O, V> {
        let count = Arc::new(AtomicUsize::new(self.count));
        let producor = self.producor.clone();
        let consumer = self.consumer.clone();
        let mut result = Vec::with_capacity(self.count);
        for _ in 0..self.count {
            result.push(Err(Error::new(ErrorKind::Other, "Unint map")))
        }

        AsyncReduce {
            futures: Some(self.futures),
            producor: Box::new(producor), //通过Box实现Pin
            consumer: Box::new(consumer), //通过Box实现Pin
            wait,
            result: AsyncWaitResults(Arc::new(RefCell::new(Some(result)))), //设置结果集初值
            count,
        }
    }
}

/*
* 异步归并，用于归并多个任务的返回值
*/
pub struct AsyncReduce<O: Default + 'static, V: Send + 'static> {
    futures:    Option<Vec<(AsyncRuntime<O>, BoxFuture<'static, Result<V>>)>>,  //待派发任务
    producor:   Box<Sender<(usize, Result<V>)>>,                                //异步返回值生成器
    consumer:   Box<Receiver<(usize, Result<V>)>>,                              //异步返回值接收器
    wait:       AsyncRuntime<O>,                                                //等待的异步运行时
    result:     AsyncWaitResults<V>,                                            //需要等待执行的异步任务的结果
    count:      Arc<AtomicUsize>,                                               //需要归并的异步任务数量
}

unsafe impl<O: Default + 'static, V: Send + 'static> Send for AsyncReduce<O, V> {}
unsafe impl<O: Default + 'static, V: Send + 'static> Sync for AsyncReduce<O, V> {}

impl<O: Default + 'static, V: Send + 'static> Future for AsyncReduce<O, V> {
    type Output = Result<Vec<Result<V>>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if (&self).count.load(Ordering::Relaxed) == 0 {
            //任务已完成，则返回
            if let Some(mut result) = (&self).result.0.borrow_mut().take() {
                let mut buf = (&self).consumer.try_iter().collect::<Vec<(usize, Result<V>)>>();
                for (idx, r) in buf {
                    //将归并结果，根据序号填充到结果集中
                    result[idx] = r;
                }

                return Poll::Ready(Ok(result));
            }
        }

        //在归并任务所在运行时中派发所有异步任务
        let task_id = self.wait.alloc();
        let task_id_ = task_id.clone();
        let mut futures = (&mut self).futures.take().unwrap();
        let producor = (&self).producor.clone();
        let wait = (&self).wait.clone();
        let count = (&self).count.clone();
        let task = async move {
            while let Some((runtime, future)) = futures.pop() {
                let task_id_copy = task_id_.clone();
                let wait_copy = wait.clone();
                let count_copy = count.clone();
                let producor_copy = producor.clone();
                let index = futures.len();

                if let Err(e) = runtime.spawn(runtime.alloc(), async move {
                    let value = future.await;
                    let _ = producor_copy.send((index, value));
                    if count_copy.fetch_sub(1, Ordering::SeqCst) == 1 {
                        //最后一个任务已执行完成，则立即唤醒等待的归并任务
                        wait_copy.wakeup(&task_id_copy);
                    }

                    //返回异步任务的默认值
                    Default::default()
                }) {
                    //派发异步任务失败，则退出派发循环
                    let _ = producor.send((index, Err(e)));
                    if count.clone().fetch_sub(1, Ordering::SeqCst) == 1 {
                        //最后一个任务已执行完成，则立即唤醒等待的归并任务
                        wait.wakeup(&task_id_);
                    }
                }
            }

            //返回异步任务的默认值
            Default::default()
        };

        //挂起当前归并任务，并返回值未就绪，以保证归并任务在执行前不会被唤醒
        let reply = self.wait.pending(&task_id, cx.waker().clone());
        if let Err(e) = self.wait.spawn(self.wait.alloc(), task) {
            //派发异步映射任务失败，则移除已挂起的异步等待任务，并立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Map Error, reason: {:?}", e))));
        }

        reply
    }
}

/*
* 派发一个工作线程
* 返回线程的句柄，可以通过句柄关闭线程
* 线程在没有任务可以执行时会休眠，当派发任务或唤醒任务时会自动唤醒线程
*/
pub fn spawn_worker_thread<F0, F1>(thread_name: &str,
                                   thread_stack_size: usize,
                                   condvar_waker: Arc<(AtomicBool, Mutex<()>, Condvar)>, //用于唤醒运行时所在线程的条件变量
                                   sleep_timeout: u64,                                   //休眠超时时长，单位毫秒
                                   loop_interval: Option<u64>,                           //工作者线程循环的间隔时长，None为无间隔，单位毫秒
                                   loop_func: F0,
                                   get_queue_len: F1) -> Arc<AtomicBool>
    where F0: Fn() -> (bool, Duration) + Send + 'static,
          F1: Fn() -> usize + Send + 'static {
    let worker_thread_status = Arc::new(AtomicBool::new(true));
    let worker_thread_status_copy = worker_thread_status.clone();

    thread::Builder::new()
        .name(thread_name.to_string())
        .stack_size(thread_stack_size).spawn(move || {
        let mut sleep_count = 0;

        while worker_thread_status.load(Ordering::Relaxed) {
            let (is_no_task, run_time) = loop_func();

            if is_no_task {
                //当前没有任务
                if sleep_count > 1 {
                    //当前没有任务连续达到2次，则休眠线程
                    sleep_count = 0; //重置休眠计数
                    let (is_sleep, lock, condvar) = &*condvar_waker;
                    let mut locked = lock.lock();
                    if get_queue_len() > 0 {
                        //当前有任务，则继续工作
                        continue;
                    }

                    if !is_sleep.load(Ordering::Relaxed) {
                        //如果当前未休眠，则休眠
                        is_sleep.store(true, Ordering::SeqCst);
                        if condvar
                            .wait_for(
                                &mut locked,
                                Duration::from_millis(sleep_timeout),
                            )
                            .timed_out()
                        {
                            //条件超时唤醒，则设置状态为未休眠
                            is_sleep.store(false, Ordering::SeqCst);
                        }
                    }

                    continue; //唤醒后立即尝试执行任务
                }

                sleep_count += 1; //休眠计数
                if let Some(interval) = &loop_interval {
                    //设置了循环间隔时长
                    if let Some(remaining_interval) = Duration::from_millis(*interval).checked_sub(run_time){
                        //本次运行少于循环间隔，则休眠剩余的循环间隔，并继续执行任务
                        thread::sleep(remaining_interval);
                    }
                }
            } else {
                //当前有任务
                sleep_count = 0; //重置休眠计数
                if let Some(interval) = &loop_interval {
                    //设置了循环间隔时长
                    if let Some(remaining_interval) = Duration::from_millis(*interval).checked_sub(run_time){
                        //本次运行少于循环间隔，则休眠剩余的循环间隔，并继续执行任务
                        thread::sleep(remaining_interval);
                    }
                }
            }
        }
    });

    worker_thread_status_copy
}

//唤醒工作者所在线程，如果线程当前正在运行，则忽略
pub fn wakeup_worker_thread<O: Default + 'static>(worker_waker: &Arc<(AtomicBool, Mutex<()>, Condvar)>, rt: &SingleTaskRuntime<O>) {
    //检查工作者所在线程是否需要唤醒
    if worker_waker.0.load(Ordering::Relaxed) && rt.len() > 0 {
        let (is_sleep, lock, condvar) = &**worker_waker;
        let locked = lock.lock();
        is_sleep.store(false, Ordering::SeqCst); //设置为未休眠
        let _ = condvar.notify_one();
    }
}

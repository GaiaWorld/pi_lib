//! # 提供了通用的异步运行时
//!

use std::thread;
use std::pin::Pin;
use std::vec::IntoIter;
use std::future::Future;
use std::mem::transmute;
use std::sync::{Arc, Weak};
use std::any::{Any, TypeId};
use std::time::{Instant, Duration};
use std::cell::{RefCell, UnsafeCell};
use std::panic::{PanicInfo, set_hook};
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, AtomicIsize, Ordering};

use futures::stream::{Stream, BoxStream};

pub mod single_thread;
pub mod multi_thread;

use libc;
use futures::{future::{FutureExt, BoxFuture}, task::ArcWake};
use parking_lot::{Mutex, Condvar};
use crossbeam_channel::{Sender, Receiver, unbounded};
use crossbeam_queue::ArrayQueue;
use flume::{Sender as AsyncSender, Receiver as AsyncReceiver, bounded as async_bounded};
use num_cpus;

use hash::XHashMap;
use local_timer::local_timer::LocalTimer;

use single_thread::SingleTaskRuntime;
use multi_thread::MultiTaskRuntime;

use crate::lock::spin;

/*
* 线程绑定的异步运行时
*/
thread_local! {
    static PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME: UnsafeCell<(usize, usize)> = UnsafeCell::new((0, 0));
    static PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME_DICT: UnsafeCell<XHashMap<TypeId, Box<dyn Any + 'static>>> = UnsafeCell::new(XHashMap::default());
}

/*
* 异步运行时唯一id生成器
*/
static RUNTIME_UID_GEN: AtomicUsize = AtomicUsize::new(1);

///
/// 分配异步运行时唯一id
///
pub fn alloc_rt_uid() -> usize {
    RUNTIME_UID_GEN.fetch_add(1, Ordering::Relaxed)
}

///
/// 异步任务唯一id
///
#[derive(Clone)]
pub struct TaskId(Arc<AtomicUsize>);

impl Debug for TaskId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "TaskId[inner = {}]", self.0.load(Ordering::Relaxed))
    }
}

///
/// 异步任务
///
pub struct AsyncTask<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> {
    uid:        TaskId,                                 //任务唯一id
    future:     Mutex<Option<BoxFuture<'static, O>>>,   //异步任务
    pool:       Arc<P>,                                 //异步任务池
    context:    Option<UnsafeCell<Box<dyn Any>>>,       //异步任务上下文
}

unsafe impl<
    O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>
> Send for AsyncTask<O, P> {}
unsafe impl<
    O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>
> Sync for AsyncTask<O, P> {}

impl<
    O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>
> ArcWake for AsyncTask<O, P> {
    #[cfg(not(target_arch = "aarch64"))]
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let pool = arc_self.get_pool();
        let _ = pool.push_keep(arc_self.clone());

        if let Some(waits) = pool.get_waits() {
            //当前任务属于多线程异步运行时
            if let Some(worker_waker) = waits.pop() {
                //有待唤醒的工作者
                let (is_sleep, lock, condvar) = &*worker_waker;
                let locked = lock.lock();
                if is_sleep.load(Ordering::Relaxed) {
                    //待唤醒的工作者，正在休眠，则立即唤醒此工作者
                    if let Ok(true) = is_sleep
                        .compare_exchange_weak(true,
                                               false,
                                               Ordering::SeqCst,
                                               Ordering::SeqCst) {
                        //确认需要唤醒，则唤醒
                        condvar.notify_one();
                    }
                }
            }
        } else {
            //当前线程属于单线程异步运行时
            if let Some(thread_waker) = pool.get_thread_waker() {
                //当前任务池绑定了所在线程的唤醒器，则快速检查是否需要唤醒所在线程
                if thread_waker.0.load(Ordering::Relaxed) {
                    let (is_sleep, lock, condvar) = &**thread_waker;
                    let locked = lock.lock();
                    //待唤醒的线程，正在休眠，则立即唤醒此线程
                    if let Ok(true) = is_sleep
                        .compare_exchange_weak(true,
                                               false,
                                               Ordering::SeqCst,
                                               Ordering::SeqCst) {
                        //确认需要唤醒，则唤醒
                        condvar.notify_one();
                    }
                }
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let pool = arc_self.get_pool();
        let _ = pool.push_keep(arc_self.clone());

        if let Some(waits) = pool.get_waits() {
            //当前任务属于多线程异步运行时
            if let Some(worker_waker) = waits.pop() {
                //有待唤醒的工作者
                let (is_sleep, lock, condvar) = &*worker_waker;
                let locked = lock.lock();
                if is_sleep.load(Ordering::Relaxed) {
                    //待唤醒的工作者，正在休眠，则立即唤醒此工作者
                    if let Ok(true) = is_sleep
                        .compare_exchange(true,
                                          false,
                                          Ordering::SeqCst,
                                          Ordering::SeqCst) {
                        //确认需要唤醒，则唤醒
                        condvar.notify_one();
                    }
                }
            }
        } else {
            //当前线程属于单线程异步运行时
            if let Some(thread_waker) = pool.get_thread_waker() {
                //当前任务池绑定了所在线程的唤醒器，则快速检查是否需要唤醒所在线程
                if thread_waker.0.load(Ordering::Relaxed) {
                    let (is_sleep, lock, condvar) = &**thread_waker;
                    let locked = lock.lock();
                    //待唤醒的线程，正在休眠，则立即唤醒此线程
                    if let Ok(true) = is_sleep
                        .compare_exchange(true,
                                          false,
                                          Ordering::SeqCst,
                                          Ordering::SeqCst) {
                        //确认需要唤醒，则唤醒
                        condvar.notify_one();
                    }
                }
            }
        }
    }
}

impl<
    O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>
> AsyncTask<O, P> {
    /// 构建单线程任务
    pub fn new(uid: TaskId,
               pool: Arc<P>,
               future: Option<BoxFuture<'static, O>>) -> AsyncTask<O, P> {
        AsyncTask {
            uid,
            future: Mutex::new(future),
            pool,
            context: None,
        }
    }

    /// 使用指定上下文构建单线程任务
    pub fn with_context<C: 'static>(uid: TaskId,
                                    pool: Arc<P>,
                                    future: Option<BoxFuture<'static, O>>,
                                    context: C) -> AsyncTask<O, P> {
        let any = Box::new(context);

        AsyncTask {
            uid,
            future: Mutex::new(future),
            pool,
            context: Some(UnsafeCell::new(any)),
        }
    }

    /// 使用指定异步运行时和上下文构建单线程任务
    pub fn with_runtime_and_context<C: 'static>(runtime: &AsyncRuntime<O, P>,
                                                future: Option<BoxFuture<'static, O>>,
                                                context: C) -> AsyncTask<O, P> {
        let any = Box::new(context);

        AsyncTask {
            uid: runtime.alloc(),
            future: Mutex::new(future),
            pool: runtime.shared_pool(),
            context: Some(UnsafeCell::new(any)),
        }
    }

    /// 检查是否允许唤醒
    pub fn is_enable_wakeup(&self) -> bool {
        self.uid.0.load(Ordering::Relaxed) > 0
    }

    /// 获取内部任务
    pub fn get_inner(&self) -> Option<BoxFuture<'static, O>> {
        self.future.lock().take()
    }

    /// 设置内部任务
    pub fn set_inner(&self, inner: Option<BoxFuture<'static, O>>) {
        *self.future.lock() = inner;
    }

    //判断异步任务是否有上下文
    pub fn exist_context(&self) -> bool {
        self.context.is_some()
    }

    //获取异步任务上下文的只读引用
    pub fn get_context<C: 'static>(&self) -> Option<&C> {
        if let Some(context) = &self.context {
            //存在上下文
            let any = unsafe { &*context.get() };
            return <dyn Any>::downcast_ref::<C>(&**any);
        }

        None
    }

    //获取异步任务上下文的可写引用
    pub fn get_context_mut<C: 'static>(&self) -> Option<&mut C> {
        if let Some(context) = &self.context {
            //存在上下文
            let any = unsafe { &mut *context.get() };
            return <dyn Any>::downcast_mut::<C>(&mut **any);
        }

        None
    }

    //设置异步任务上下文，返回上一个异步任务上下文
    pub fn set_context<C: 'static>(&self, new: C) {
        if let Some(context) = &self.context {
            //存在上一个上下文，则释放上一个上下文
            let _ = unsafe { &*context.get() };

            //设置新的上下文
            let any: Box<dyn Any + 'static> = Box::new(new);
            unsafe { *context.get() = any; }
        }
    }

    //获取异步任务的任务池
    pub fn get_pool(&self) -> &P {
        self.pool.as_ref()
    }
}

///
/// 异步任务池
///
pub trait AsyncTaskPool<O: Default + 'static>: Default + Send + Sync + 'static {
    type Pool: AsyncTaskPoolExt<O> + AsyncTaskPool<O>;

    /// 获取绑定的线程唯一id
    fn get_thread_id(&self) -> usize;

    /// 获取当前异步任务池内任务数量
    fn len(&self) -> usize;

    /// 将异步任务加入异步任务池
    fn push(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()>;

    /// 将已超时的异步任务加入任务池
    fn push_timed_out(&self, index: u64, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()>;

    /// 异步任务被唤醒时，将异步任务继续加入异步任务池
    fn push_keep(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()>;

    /// 尝试从异步任务池中弹出一个异步任务
    fn try_pop(&self) -> Option<Arc<AsyncTask<O, Self::Pool>>>;

    /// 尝试从异步任务池中弹出所有异步任务
    fn try_pop_all(&self) -> IntoIter<Arc<AsyncTask<O, Self::Pool>>>;

    /// 获取本地线程的唤醒器
    fn get_thread_waker(&self) -> Option<&Arc<(AtomicBool, Mutex<()>, Condvar)>>;
}

///
/// 异步任务池扩展
///
pub trait AsyncTaskPoolExt<O: Default + 'static>: Send + Sync + 'static {
    /// 设置待唤醒的工作者唤醒器队列
    fn set_waits(&mut self,
                 _waits: Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>) {}

    /// 获取待唤醒的工作者唤醒器队列
    fn get_waits(&self) -> Option<&Arc<ArrayQueue<Arc<(AtomicBool, Mutex<()>, Condvar)>>>> {
        //默认没有待唤醒的工作者唤醒器队列
        None
    }

    /// 获取空闲的工作者的数量，这个数量大于0，表示可以新开线程来运行可分派的工作者
    fn idler_len(&self) -> usize {
        //默认不分派
        0
    }

    /// 分派一个空闲的工作者
    fn spawn_worker(&self) -> Option<usize> {
        //默认不分派
        None
    }

    /// 获取工作者的数量
    fn worker_len(&self) -> usize {
        //默认工作者数量和本机逻辑核数相同
        num_cpus::get()
    }

    /// 获取缓冲区的任务数量，缓冲区任务是未分配给工作者的任务
    fn buffer_len(&self) -> usize {
        //默认没有缓冲区
        0
    }

    /// 设置当前绑定本地线程的唤醒器
    fn set_thread_waker(&mut self, _thread_waker: Arc<(AtomicBool, Mutex<()>, Condvar)>) {
        //默认不设置
    }

    /// 复制当前绑定本地线程的唤醒器
    fn clone_thread_waker(&self) -> Option<Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        //默认不复制
        None
    }

    /// 关闭当前工作者
    fn close_worker(&self) {
        //默认不允许关闭工作者
    }
}

///
/// 异步运行时扩展
///
pub trait AsyncRuntimeExt<O: Default + 'static> {
    /// 派发一个指定的异步任务到异步运行时，并指定异步任务的初始化上下文
    fn spawn_with_context<F, C>(&self,
                                task_id: TaskId,
                                future: F,
                                context: C) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static;

    /// 派发一个在指定时间后执行的异步任务到异步运行时，并指定异步任务的初始化上下文，时间单位ms
    fn spawn_timing_with_context<F, C>(&self,
                                       task_id: TaskId,
                                       future: F,
                                       context: C,
                                       time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static;

    /// 立即创建一个指定任务池的异步运行时，并执行指定的异步任务，阻塞当前线程，等待异步任务完成后返回
    fn block_on<RP, F>(&self, future: F) -> Result<F::Output>
        where RP: AsyncTaskPoolExt<F::Output> + AsyncTaskPool<F::Output, Pool = RP>,
              F: Future + Send + 'static,
              <F as Future>::Output: Default + Send + 'static;
}

///
/// 异步运行时
///
pub enum AsyncRuntime<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> {
    Local(SingleTaskRuntime<O, P>),                                                             //本地运行时
    Multi(MultiTaskRuntime<O, P>),                                                              //多线程运行时
    Worker(Arc<AtomicBool>, Arc<(AtomicBool, Mutex<()>, Condvar)>, SingleTaskRuntime<O, P>),    //工作者运行时
}

unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Send for AsyncRuntime<O, P> {}
unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Sync for AsyncRuntime<O, P> {}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Clone for AsyncRuntime<O, P> {
    fn clone(&self) -> Self {
        match self {
            AsyncRuntime::Local(rt) => AsyncRuntime::Local(rt.clone()),
            AsyncRuntime::Multi(rt) => AsyncRuntime::Multi(rt.clone()),
            AsyncRuntime::Worker(wroker_status, worker_waker, rt) => AsyncRuntime::Worker(wroker_status.clone(), worker_waker.clone(), rt.clone()),
        }
    }
}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> AsyncRuntimeExt<O> for AsyncRuntime<O, P> {
    fn spawn_with_context<F, C>(&self,
                                task_id: TaskId,
                                future: F,
                                context: C) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.spawn_with_context(task_id, future, context),
            AsyncRuntime::Multi(rt) => rt.spawn_with_context(task_id, future, context),
            AsyncRuntime::Worker(_, _, rt) => rt.spawn_with_context(task_id, future, context),
        }
    }

    fn spawn_timing_with_context<F, C>(&self,
                                       task_id: TaskId,
                                       future: F,
                                       context: C,
                                       time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.spawn_timing_with_context(task_id, future, context, time),
            AsyncRuntime::Multi(rt) => rt.spawn_timing_with_context(task_id, future, context, time),
            AsyncRuntime::Worker(_, _, rt) => rt.spawn_timing_with_context(task_id, future, context, time),
        }
    }

    fn block_on<RP, F>(&self, future: F) -> Result<F::Output>
        where RP: AsyncTaskPoolExt<F::Output> + AsyncTaskPool<F::Output, Pool = RP>,
              F: Future + Send + 'static,
              <F as Future>::Output: Default + Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.block_on::<RP, F>(future),
            AsyncRuntime::Multi(rt) => rt.block_on::<RP, F>(future),
            AsyncRuntime::Worker(_, _, rt) => rt.block_on::<RP, F>(future),
        }
    }
}

/*
* 异步运行时同步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> AsyncRuntime<O, P> {
    /// 共享运行时内部任务池
    pub(crate) fn shared_pool(&self) -> Arc<P> {
        match self {
            AsyncRuntime::Local(rt) => rt.shared_pool(),
            AsyncRuntime::Multi(rt) => rt.shared_pool(),
            AsyncRuntime::Worker(_, _, rt) => rt.shared_pool(),
        }
    }

    /// 获取当前异步运行时的唯一id
    pub fn get_id(&self) -> usize {
        match self {
            AsyncRuntime::Local(rt) => rt.get_id(),
            AsyncRuntime::Multi(rt) => rt.get_id(),
            AsyncRuntime::Worker(_, _, rt) => rt.get_id(),
        }
    }

    /// 获取当前异步运行时待处理任务数量
    pub fn wait_len(&self) -> usize {
        match self {
            AsyncRuntime::Local(rt) => rt.len(),
            AsyncRuntime::Multi(rt) => rt.len(),
            AsyncRuntime::Worker(_, _, rt) => rt.len(),
        }
    }

    /// 获取当前异步运行时任务数量
    pub fn len(&self) -> usize {
        match self {
            AsyncRuntime::Local(rt) => rt.len(),
            AsyncRuntime::Multi(rt) => rt.len(),
            AsyncRuntime::Worker(_, _, rt) => rt.len(),
        }
    }

    /// 分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        match self {
            AsyncRuntime::Local(rt) => rt.alloc(),
            AsyncRuntime::Multi(rt) => rt.alloc(),
            AsyncRuntime::Worker(_, _, rt) => rt.alloc(),
        }
    }

    /// 派发一个指定的异步任务到异步运行时
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

    /// 派发一个在指定时间后执行的异步任务到异步运行时，时间单位ms
    pub fn spawn_timing<F>(&self, task_id: TaskId, future: F, time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.spawn_timing(task_id, future, time),
            AsyncRuntime::Multi(rt) => rt.spawn_timing(task_id, future, time),
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

    /// 挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: &TaskId, waker: Waker) -> Poll<Output> {
        match self {
            AsyncRuntime::Local(rt) => rt.pending(task_id, waker),
            AsyncRuntime::Multi(rt) => rt.pending(task_id, waker),
            AsyncRuntime::Worker(_, _, rt) => rt.pending(task_id, waker),
        }
    }

    /// 唤醒指定唯一id的异步任务
    pub fn wakeup(&self, task_id: &TaskId) {
        match self {
            AsyncRuntime::Local(rt) => rt.wakeup(task_id),
            AsyncRuntime::Multi(rt) => rt.wakeup(task_id),
            AsyncRuntime::Worker(_, _, rt) => rt.wakeup(task_id),
        }
    }

    /// 构建用于派发多个异步任务到指定运行时的映射归并，需要指定映射归并的容量
    pub fn map_reduce<V: Send + 'static>(&self, capacity: usize) -> AsyncMapReduce<V> {
        match self {
            AsyncRuntime::Local(rt) => rt.map_reduce(capacity),
            AsyncRuntime::Multi(rt) => rt.map_reduce(capacity),
            AsyncRuntime::Worker(_, _, rt) => rt.map_reduce(capacity),
        }
    }

    /// 生成一个异步管道，输入指定流，输入流的每个值通过过滤器生成输出流的值
    pub fn pipeline<S, SO, F, FO>(&self, input: S, mut filter: F) -> BoxStream<'static, FO>
        where S: Stream<Item = SO> + Send + 'static,
              SO: Send + 'static,
              F: FnMut(SO) -> AsyncPipelineResult<FO> + Send + 'static,
              FO: Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.pipeline(input, filter),
            AsyncRuntime::Multi(rt) => rt.pipeline(input, filter),
            AsyncRuntime::Worker(_, _, rt) => rt.pipeline(input, filter),
        }
    }

    /// 关闭异步运行时，返回请求关闭是否成功
    pub fn close(&self) -> bool {
        match self {
            AsyncRuntime::Worker(worker_status, worker_waker, rt) => {
                if cfg!(target_arch = "aarch64") {
                    if let Ok(true) = worker_status.compare_exchange(true,
                                                                     false,
                                                                     Ordering::SeqCst,
                                                                     Ordering::SeqCst) {
                        //设置工作者状态成功，检查运行时所在线程是否需要唤醒
                        wakeup_worker_thread(worker_waker, rt);
                        true
                    } else {
                        false
                    }
                } else {
                    if let Ok(true) = worker_status.compare_exchange_weak(true,
                                                                          false,
                                                                          Ordering::SeqCst,
                                                                          Ordering::SeqCst) {
                        //设置工作者状态成功，检查运行时所在线程是否需要唤醒
                        wakeup_worker_thread(worker_waker, rt);
                        true
                    } else {
                        false
                    }
                }
            },
            _ => false,
        }
    }
}

/*
* 异步运行时同步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> AsyncRuntime<O, P> {
    /// 挂起当前异步运行时的当前任务，等待指定的时间后唤醒当前任务
    pub async fn wait_timeout(&self, timeout: usize) {
        match self {
            AsyncRuntime::Local(rt) => rt.wait_timeout(timeout).await,
            AsyncRuntime::Multi(rt) => rt.wait_timeout(timeout).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait_timeout(timeout).await,
        }
    }

    /// 挂起当前异步运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, RP, V, F>(&self, art: AsyncRuntime<R, RP>, future: F) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.wait(art, future).await,
            AsyncRuntime::Multi(rt) => rt.wait(art, future).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait(art, future).await,
        }
    }

    /// 挂起当前异步运行时的当前任务，并在多个其它运行时上执行多个其它任务，其中任意一个任务完成，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成的任务的值将被忽略
    pub async fn wait_any<R, RP, V>(&self,
                                    futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static  {
        match self {
            AsyncRuntime::Local(rt) => rt.wait_any(futures).await,
            AsyncRuntime::Multi(rt) => rt.wait_any(futures).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait_any(futures).await,
        }
    }

    /// 挂起当前异步运行时的当前任务，并在多个其它运行时上执行多个其它任务，任务返回后需要通过用户指定的检查回调进行检查，其中任意一个任务检查通过，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成或未检查通过的任务的值将被忽略，如果所有任务都未检查通过，则强制唤醒当前运行时的当前任务
    pub async fn wait_any_callback<R, RP, V, F>(&self,
                                                futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>,
                                                callback: F) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static,
              F: Fn(&Result<V>) -> bool + Send + Sync + 'static {
        match self {
            AsyncRuntime::Local(rt) => rt.wait_any_callback(futures, callback).await,
            AsyncRuntime::Multi(rt) => rt.wait_any_callback(futures, callback).await,
            AsyncRuntime::Worker(_, _, rt) => rt.wait_any_callback(futures, callback).await,
        }
    }
}

///
/// 获取本地线程绑定的异步运行时
/// 注意：O如果与本地线程绑定的运行时的O不相同，则无法获取本地线程绑定的运行时
///
pub fn current_async_runtime<O, P>() -> Option<AsyncRuntime<O, P>>
    where O: Default + 'static,
          P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P> {
    match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME.try_with(move |rt| {
        unsafe {
            let ptr = *rt.get();
            if ptr == (0, 0) {
                //本地线程未绑定异步运行时
                None
            } else {
                //本地线程已绑定异步运行时
                let any: Arc<dyn Any + Send + Sync> = unsafe { transmute(ptr) };
                match Arc::downcast::<AsyncRuntime<O, P>>(any.clone()) {
                    Err(shared) => {
                        //造型失败
                        Arc::into_raw(shared); //避免提前释放
                        None
                    },
                    Ok(shared) => {
                        //造型成功
                        let result = shared.as_ref().clone();
                        Arc::into_raw(shared); //避免提前释放
                        Some(result)
                    }
                }
            }
        }
    }) {
        Err(_) => None, //本地线程没有绑定异步运行时
        Ok(rt) => rt,
    }
}

///
/// 派发任务到本地线程绑定的异步运行时，如果本地线程没有异步运行时，则返回错误
/// 注意：F::Output如果与本地线程绑定的运行时的O不相同，则无法执行指定任务
///
pub fn spawn_local<P, F>(future: F) -> Result<()>
    where P: AsyncTaskPoolExt<F::Output> + AsyncTaskPool<F::Output, Pool = P>,
          F: Future + Send + 'static,
          <F as Future>::Output: Default + 'static {
    if let Some(rt) = current_async_runtime::<F::Output, P>() {
        rt.spawn(rt.alloc(), future)
    } else {
        Err(Error::new(ErrorKind::Other, format!("Spawn task to local thread failed, reason: runtime not exist")))
    }
}

///
/// 从本地线程绑定的字典中获取指定类型的值的只读引用
///
pub fn get_local_dict<T: 'static>() -> Option<&'static T> {
    match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME_DICT.try_with(move |dict| {
        unsafe {
            if let Some(any) = (&*dict.get()).get(&TypeId::of::<T>()) {
                //指定类型的值存在
                <dyn Any>::downcast_ref::<T>(&**any)
            } else {
                //指定类型的值不存在
                None
            }
        }
    }) {
        Err(_) => {
            None
        },
        Ok(result) => {
            result
        }
    }
}

///
/// 从本地线程绑定的字典中获取指定类型的值的可写引用
///
pub fn get_local_dict_mut<T: 'static>() -> Option<&'static mut T> {
    match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME_DICT.try_with(move |dict| {
        unsafe {
            if let Some(any) = (&mut *dict.get()).get_mut(&TypeId::of::<T>()) {
                //指定类型的值存在
                <dyn Any>::downcast_mut::<T>(&mut **any)
            } else {
                //指定类型的值不存在
                None
            }
        }
    }) {
        Err(_) => {
            None
        },
        Ok(result) => {
            result
        }
    }
}

///
/// 在本地线程绑定的字典中设置指定类型的值，返回上一个设置的值
///
pub fn set_local_dict<T: 'static>(value: T) -> Option<T> {
    match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME_DICT.try_with(move |dict| {
        unsafe {
            let result = if let Some(any) = (&mut *dict.get()).remove(&TypeId::of::<T>()) {
                //指定类型的上一个值存在
                if let Ok(r) = any.downcast() {
                    //造型成功，则返回
                    Some(*r)
                } else {
                    None
                }
            } else {
                //指定类型的上一个值不存在
                None
            };

            //设置指定类型的新值
            (&mut *dict.get()).insert(TypeId::of::<T>(), Box::new(value) as Box<dyn Any>);

            result
        }
    }) {
        Err(_) => {
            None
        },
        Ok(result) => {
            result
        }
    }
}

///
/// 在本地线程绑定的字典中移除指定类型的值，并返回移除的值
///
pub fn remove_local_dict<T: 'static>() -> Option<T> {
    match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME_DICT.try_with(move |dict| {
        unsafe {
            if let Some(any) = (&mut *dict.get()).remove(&TypeId::of::<T>()) {
                //指定类型的上一个值存在
                if let Ok(r) = any.downcast() {
                    //造型成功，则返回
                    Some(*r)
                } else {
                    None
                }
            } else {
                //指定类型的上一个值不存在
                None
            }
        }
    }) {
        Err(_) => {
            None
        },
        Ok(result) => {
            result
        }
    }
}

///
/// 清空本地线程绑定的字典
///
pub fn clear_local_dict() -> Result<()> {
    match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME_DICT.try_with(move |dict| {
        unsafe {
            (&mut *dict.get()).clear();
        }
    }) {
        Err(e) => {
            Err(Error::new(ErrorKind::Other, format!("Clear local dict failed, reason: {:?}", e)))
        },
        Ok(_) => {
            Ok(())
        }
    }
}

///
/// 异步值
///
pub struct AsyncValue<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    V: Send + 'static,
> {
    rt:         AsyncRuntime<O, P>,         //异步值的运行时
    task_id:    TaskId,                     //异步值的任务唯一id
    value:      Arc<RefCell<Option<V>>>,    //值
    status:     Arc<AtomicU8>,              //值状态
}

unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    V: Send + 'static,
> Send for AsyncValue<O, P, V> {}
unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    V: Send + 'static,
> Sync for AsyncValue<O, P, V> {}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    V: Send + 'static,
> Clone for AsyncValue<O, P, V> {
    fn clone(&self) -> Self {
        AsyncValue {
            rt: self.rt.clone(),
            task_id: self.task_id.clone(),
            value: self.value.clone(),
            status: self.status.clone(),
        }
    }
}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    V: Send + 'static,
> Future for AsyncValue<O, P, V> {
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

/*
* 异步值同步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    V: Send + 'static,
> AsyncValue<O, P, V> {
    /// 构建异步值，默认值为未就绪
    pub fn new(rt: AsyncRuntime<O, P>) -> Self {
        let task_id = rt.alloc();

        AsyncValue {
            rt,
            task_id,
            value: Arc::new(RefCell::new(None)),
            status: Arc::new(AtomicU8::new(0)),
        }
    }

    /// 设置异步值
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

///
/// 等待异步任务运行的结果
///
pub struct AsyncWaitResult<V: Send + 'static>(pub Arc<RefCell<Option<Result<V>>>>);

unsafe impl<V: Send + 'static> Send for AsyncWaitResult<V> {}
unsafe impl<V: Send + 'static> Sync for AsyncWaitResult<V> {}

impl<V: Send + 'static> Clone for AsyncWaitResult<V> {
    fn clone(&self) -> Self {
        AsyncWaitResult(self.0.clone())
    }
}

///
/// 等待异步任务运行的结果集
///
pub struct AsyncWaitResults<V: Send + 'static>(pub Arc<RefCell<Option<Vec<Result<V>>>>>);

unsafe impl<V: Send + 'static> Send for AsyncWaitResults<V> {}
unsafe impl<V: Send + 'static> Sync for AsyncWaitResults<V> {}

impl<V: Send + 'static> Clone for AsyncWaitResults<V> {
    fn clone(&self) -> Self {
        AsyncWaitResults(self.0.clone())
    }
}

///
/// 异步定时器任务
///
pub enum AsyncTimingTask<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> {
    Pended(TaskId),                 //已挂起的定时任务
    WaitRun(Arc<AsyncTask<O, P>>),  //等待执行的定时任务
}

///
/// 异步任务本地定时器
///
pub struct AsyncTaskTimer<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> {
    producor:   Sender<(usize, AsyncTimingTask<O, P>)>,                             //定时任务生产者
    consumer:   Receiver<(usize, AsyncTimingTask<O, P>)>,                           //定时任务消费者
    timer:      Arc<RefCell<LocalTimer<AsyncTimingTask<O, P>, 1000, 60, 60, 24>>>,  //定时器
}

unsafe impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> Send for AsyncTaskTimer<O, P> {}
unsafe impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> Sync for AsyncTaskTimer<O, P> {}

impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> AsyncTaskTimer<O, P> {
    /// 构建异步任务本地定时器
    pub fn new() -> Self {
        let (producor, consumer) = unbounded();
        AsyncTaskTimer {
            producor,
            consumer,
            timer: Arc::new(RefCell::new(LocalTimer::<AsyncTimingTask<O, P>, 1000, 60, 60, 24>::new(1, (minstant::now() as f64 * minstant::nanos_per_cycle() / 1000000.0).trunc() as u64))),
        }
    }

    /// 构建指定间隔的异步任务本地定时器
    pub fn with_interval(time: usize) -> Self {
        let (producor, consumer) = unbounded();
        AsyncTaskTimer {
            producor,
            consumer,
            timer: Arc::new(RefCell::new(LocalTimer::<AsyncTimingTask<O, P>, 1000, 60, 60, 24>::new(time as u64, (minstant::now() as f64 * minstant::nanos_per_cycle() / 1000000.0).trunc() as u64))),
        }
    }

    /// 获取定时任务生产者
    #[inline]
    pub fn get_producor(&self) -> &Sender<(usize, AsyncTimingTask<O, P>)> {
        &self.producor
    }

    /// 获取剩余未到期的定时器任务数量
    #[inline]
    pub fn len(&self) -> usize {
        self.timer.as_ref().borrow().len()
    }

    /// 设置定时器
    pub fn set_timer(&self, task: AsyncTimingTask<O, P>, timeout: usize) -> usize {
        self.timer.borrow_mut().insert(task, timeout as u64)
    }

    /// 取消定时器
    pub fn cancel_timer(&self, timer_ref: usize) -> Option<AsyncTimingTask<O, P>> {
        if let Some(item) = self.timer.borrow_mut().try_remove(timer_ref) {
            Some(item.elem)
        } else {
            None
        }
    }

    /// 消费所有定时任务，返回定时任务数量
    pub fn consume(&self) -> usize {
        let mut len = 0;
        let timer_tasks = self.consumer.try_iter().collect::<Vec<(usize, AsyncTimingTask<O, P>)>>();
        for (timeout, task) in timer_tasks {
            self.set_timer(task, timeout);
            len += 1;
        }

        len
    }

    /// 判断当前时间是否有可以弹出的任务，如果有可以弹出的任务，则返回当前时间，否则返回空
    pub fn is_require_pop(&self) -> Option<u64> {
        let current_time = (minstant::now() as f64 * minstant::nanos_per_cycle() / 1000000.0).trunc() as u64;
        if self.timer.borrow().check_sleep(current_time) == 0 {
            Some(current_time)
        } else {
            None
        }
    }

    /// 从定时器中弹出指定时间的一个到期任务
    pub fn pop(&self, current_time: u64) -> Option<(usize, AsyncTimingTask<O, P>)> {
        if let Some((item, index)) = self.timer.borrow_mut().pop(current_time) {
            Some((index, item.elem))
        } else {
            None
        }
    }

    /// 清空定时器
    pub fn clear(&self) {
        self.timer.borrow_mut().clear();
    }
}

///
/// 等待指定超时
///
pub struct AsyncWaitTimeout<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> {
    rt:         AsyncRuntime<O, P>,                     //当前运行时
    producor:   Sender<(usize, AsyncTimingTask<O, P>)>, //超时请求生产者
    timeout:    usize,                                  //超时时长，单位ms
    expired:    bool,                                   //是否已过期
}

unsafe impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> Send for AsyncWaitTimeout<O, P> {}
unsafe impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> Sync for AsyncWaitTimeout<O, P> {}

impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>> Future for AsyncWaitTimeout<O, P> {
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

impl<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>> AsyncWaitTimeout<O, P> {
    /// 构建等待指定超时任务的方法
    pub fn new(rt: AsyncRuntime<O, P>,
               producor: Sender<(usize, AsyncTimingTask<O, P>)>,
               timeout: usize) -> Self {
        AsyncWaitTimeout {
            rt,
            producor,
            timeout,
            expired: false, //设置初始值
        }
    }
}

///
/// 等待异步任务执行完成
///
pub struct AsyncWait<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static
> {
    wait:   AsyncRuntime<O, OP>,                    //需要等待的异步运行时
    runner: AsyncRuntime<R, RP>,                    //需要运行的异步运行时
    future: Option<BoxFuture<'static, Result<V>>>,  //需要等待执行的异步任务
    result: AsyncWaitResult<V>,                     //需要等待执行的异步任务的结果
}

unsafe impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static
> Send for AsyncWait<O, R, OP, RP, V> {}
unsafe impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static
> Sync for AsyncWait<O, R, OP, RP, V> {}

impl<O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = OP>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
    V: Send + 'static
> Future for AsyncWait<O, R, OP, RP, V> {
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

impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static
> AsyncWait<O, R, OP, RP, V> {
    /// 构建等待异步任务执行完成的方法
    fn new(wait: AsyncRuntime<O, OP>,
           runner: AsyncRuntime<R, RP>,
           future: Option<BoxFuture<'static, Result<V>>>) -> Self {
        AsyncWait {
            wait,
            runner,
            future,
            result: AsyncWaitResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

///
/// 等待任意异步任务执行完成
///
pub struct AsyncWaitAny<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
> {
    wait:       AsyncRuntime<O, OP>,                                                //需要等待的异步运行时
    futures:    Option<Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>>,  //需要运行的异步运行时和等待执行的异步任务
    is_finish:  Arc<AtomicBool>,                                                    //是否有任意的异步任务已执行完成
    result:     AsyncWaitResult<V>,                                                 //需要等待执行的异步任务的结果
}

unsafe impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
> Send for AsyncWaitAny<O, R, OP, RP, V> {}
unsafe impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
> Sync for AsyncWaitAny<O, R, OP, RP, V> {}

impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = OP>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
    V: Send + 'static,
> Future for AsyncWaitAny<O, R, OP, RP, V> {
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
                    if cfg!(target_arch = "aarch64") {
                        if let Ok(false) = is_finish_copy.compare_exchange(false,
                                                                           true,
                                                                           Ordering::SeqCst,
                                                                           Ordering::SeqCst) {
                            //当前任务执行完成，则立即唤醒等待的任务
                            *result_copy.0.borrow_mut() = Some(r);
                            wait_copy.wakeup(&task_id_copy);
                        }
                    } else {
                        if let Ok(false) = is_finish_copy.compare_exchange_weak(false,
                                                                                true,
                                                                                Ordering::SeqCst,
                                                                                Ordering::SeqCst) {
                            //当前任务执行完成，则立即唤醒等待的任务
                            *result_copy.0.borrow_mut() = Some(r);
                            wait_copy.wakeup(&task_id_copy);
                        }
                    }

                    //返回异步任务的默认值
                    Default::default()
                }) {
                    //派发指定的任务失败，则退出派发循环
                    if cfg!(target_arch = "aarch64") {
                        if let Ok(false) = is_finish.compare_exchange(false,
                                                                      true,
                                                                      Ordering::SeqCst,
                                                                      Ordering::SeqCst) {
                            //立即唤醒等待的任务
                            *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Task Runner Error, reason: {:?}", e))));
                            wait.wakeup(&task_id_);
                        }
                    } else {
                        if let Ok(false) = is_finish.compare_exchange_weak(false,
                                                                           true,
                                                                           Ordering::SeqCst,
                                                                           Ordering::SeqCst) {
                            //立即唤醒等待的任务
                            *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Task Runner Error, reason: {:?}", e))));
                            wait.wakeup(&task_id_);
                        }
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

impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
> AsyncWaitAny<O, R, OP, RP, V> {
    /// 构建等待异步任务执行完成的方法
    fn new(wait: AsyncRuntime<O, OP>,
           futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>) -> Self {
        AsyncWaitAny {
            wait,
            futures: Some(futures),
            is_finish: Arc::new(AtomicBool::new(false)),
            result: AsyncWaitResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

///
/// 等待任意异步任务执行完成
///
pub struct AsyncWaitAnyCallback<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
    F: Fn(&Result<V>) -> bool + Send + Sync + 'static,
> {
    wait:       AsyncRuntime<O, OP>,                                                //需要等待的异步运行时
    futures:    Option<Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>>,  //需要运行的异步运行时和等待执行的异步任务
    callback:   Arc<RefCell<Option<F>>>,                                            //检查器回调，F的约束导致无法自动实现DerefMut，所以需要用Arc和RefCell
    is_finish:  Arc<AtomicBool>,                                                    //是否有任意的异步任务已执行完成
    result:     AsyncWaitResult<V>,                                                 //需要等待执行的异步任务的结果
}

unsafe impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
    F: Fn(&Result<V>) -> bool + Send + Sync + 'static,
> Send for AsyncWaitAnyCallback<O, R, OP, RP, V, F> {}
unsafe impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
    F: Fn(&Result<V>) -> bool + Send + Sync + 'static,
> Sync for AsyncWaitAnyCallback<O, R, OP, RP, V, F> {}

impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = OP>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
    V: Send + 'static,
    F: Fn(&Result<V>) -> bool + Send + Sync + 'static,
> Future for AsyncWaitAnyCallback<O, R, OP, RP, V, F> {
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
        let futures_len = futures.len();
        let callback = (&self).callback.borrow_mut().take();
        let is_finish = (&self).is_finish.clone();
        let result = (&self).result.clone();
        let task = async move {
            let checker = create_checker(futures_len, callback);

            while let Some((runner, future)) = futures.pop() {
                let task_id_copy = task_id_.clone();
                let wait_copy = wait.clone();
                let is_finish_copy = is_finish.clone();
                let result_copy = result.clone();

                //将指定任务派发到本地运行时
                let checker_copy = checker.clone();
                if let Err(e) = runner.spawn(runner.alloc(), async move {
                    if is_finish_copy.load(Ordering::Relaxed) {
                        //快速检查，当前已有任务执行完成，则忽略，并立即返回
                        return Default::default();
                    }

                    //执行任务，并检查是否由当前任务唤醒等待的任务
                    let r = future.await;
                    if let Some(check) = checker_copy {
                        //有检查器
                        if check(&r) {
                            //检查通过，则立即唤醒等待的任务，否则等待其它任务唤醒
                            if cfg!(target_arch = "aarch64") {
                                if let Ok(false) = is_finish_copy.compare_exchange(false,
                                                                                   true,
                                                                                   Ordering::SeqCst,
                                                                                   Ordering::SeqCst) {
                                    *result_copy.0.borrow_mut() = Some(r);
                                    wait_copy.wakeup(&task_id_copy);
                                }
                            } else {
                                if let Ok(false) = is_finish_copy.compare_exchange_weak(false,
                                                                                        true,
                                                                                        Ordering::SeqCst,
                                                                                        Ordering::SeqCst) {
                                    *result_copy.0.borrow_mut() = Some(r);
                                    wait_copy.wakeup(&task_id_copy);
                                }
                            }
                        }
                    } else {
                        //无检查器，则立即唤醒等待的任务
                        if cfg!(target_arch = "aarch64") {
                            if let Ok(false) = is_finish_copy.compare_exchange(false,
                                                                               true,
                                                                               Ordering::SeqCst,
                                                                               Ordering::SeqCst) {
                                *result_copy.0.borrow_mut() = Some(r);
                                wait_copy.wakeup(&task_id_copy);
                            }
                        } else {
                            if let Ok(false) = is_finish_copy.compare_exchange_weak(false,
                                                                                    true,
                                                                                    Ordering::SeqCst,
                                                                                    Ordering::SeqCst) {
                                *result_copy.0.borrow_mut() = Some(r);
                                wait_copy.wakeup(&task_id_copy);
                            }
                        }
                    }

                    //返回异步任务的默认值
                    Default::default()
                }) {
                    //派发指定的任务失败，则退出派发循环
                    if cfg!(target_arch = "aarch64") {
                        if let Ok(false) = is_finish.compare_exchange(false,
                                                                      true,
                                                                      Ordering::SeqCst,
                                                                      Ordering::SeqCst) {
                            //立即唤醒等待的任务
                            *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Task Runner Error, reason: {:?}", e))));
                            wait.wakeup(&task_id_);
                        }
                    } else {
                        if let Ok(false) = is_finish.compare_exchange_weak(false,
                                                                           true,
                                                                           Ordering::SeqCst,
                                                                           Ordering::SeqCst) {
                            //立即唤醒等待的任务
                            *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Task Runner Error, reason: {:?}", e))));
                            wait.wakeup(&task_id_);
                        }
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

// 根据用户提供的回调，生成检查器
fn create_checker<V, F>(len: usize,
                        callback: Option<F>) -> Option<Arc<dyn Fn(&Result<V>) -> bool + Send + Sync + 'static>>
    where V: Send + 'static,
          F: Fn(&Result<V>) -> bool + Send + Sync + 'static {
    if let Some(callback) = callback {
        //用户指定了回调
        let check_counter = AtomicUsize::new(len); //初始化检查计数器
        Some(Arc::new(move |result| {
            if check_counter.fetch_sub(1, Ordering::SeqCst) == 1 {
                //最后一个任务的检查，则忽略用户回调，并立即返回成功
                true
            } else {
                //不是最后一个任务的检查，则调用用户回调，并根据用户回调确定是否成功
                callback(result)
            }
        }))
    } else {
        //用户未指定回调，则返回空检查器
        None
    }
}

impl<
    O: Default + 'static,
    R: Default + 'static,
    OP: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
    RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R>,
    V: Send + 'static,
    F: Fn(&Result<V>) -> bool + Send + Sync + 'static,
> AsyncWaitAnyCallback<O, R, OP, RP, V, F> {
    /// 构建等待异步任务执行完成的方法
    fn new(wait: AsyncRuntime<O, OP>,
           futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>,
           callback: Option<F>) -> Self {
        AsyncWaitAnyCallback {
            wait,
            futures: Some(futures),
            callback: Arc::new(RefCell::new(callback)),
            is_finish: Arc::new(AtomicBool::new(false)),
            result: AsyncWaitResult(Arc::new(RefCell::new(None))), //设置初始值
        }
    }
}

///
/// 异步映射归并
///
pub struct AsyncMapReduce<V: Send + 'static> {
    count:          usize,                              //派发的任务数量
    capacity:       usize,                              //派发任务的容量
    producor:       AsyncSender<(usize, Result<V>)>,    //异步返回值生成器
    consumer:       AsyncReceiver<(usize, Result<V>)>,  //异步返回值接收器
}

unsafe impl<V: Send + 'static> Send for AsyncMapReduce<V> {}

/*
* 异步映射归并同步方法
*/
impl<V: Send + 'static> AsyncMapReduce<V> {
    /// 映射指定任务到指定的运行时，并返回任务序号
    pub fn map<O, P, F>(&mut self, rt: AsyncRuntime<O, P>, future: F) -> Result<usize>
        where O: Default + 'static,
              P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
              F: Future<Output = Result<V>> + Send + 'static {
        if self.count >= self.capacity {
            //已派发任务已达可派发任务的限制，则返回错误
            return Err(Error::new(ErrorKind::Other, format!("Map task to runtime failed, capacity: {}, reason: out of capacity", self.capacity)));
        }

        let index = self.count;
        let producor = self.producor.clone();
        rt.spawn(rt.alloc(), async move {
            let value = future.await;
            producor.into_send_async((index, value)).await;

            //返回异步任务的默认值
            Default::default()
        })?;

        self.count += 1; //派发任务成功，则计数
        Ok(index)
    }
}

/*
* 异步映射归并异步方法
*/
impl<V: Send + 'static> AsyncMapReduce<V> {
    /// 归并所有派发的任务
    pub async fn reduce(self, order: bool) -> Result<Vec<Result<V>>> {
        let mut count = self.count;
        let mut results = Vec::with_capacity(count);
        while count > 0 {
            match self.consumer.recv_async().await {
                Err(e) => {
                    //接收错误，则立即返回
                    return Err(Error::new(ErrorKind::Other, format!("Reduce result failed, reason: {:?}", e)));
                },
                Ok((index, result)) => {
                    //接收成功，则继续
                    results.push((index, result));
                    count -= 1;
                },
            }
        }

        if order {
            //需要对结果集进行排序
            results.sort_by_key(|(key, _value)| {
                key.clone()
            });
        }
        let (_, values) = results
            .into_iter()
            .unzip::<usize, Result<V>, Vec<usize>, Vec<Result<V>>>();

        Ok(values)
    }
}

///
/// 异步管道过滤器结果
///
pub enum AsyncPipelineResult<O: 'static> {
    Disconnect,     //关闭管道
    Filtered(O),    //过滤后的值
}

///
/// 派发一个工作线程
/// 返回线程的句柄，可以通过句柄关闭线程
/// 线程在没有任务可以执行时会休眠，当派发任务或唤醒任务时会自动唤醒线程
///
pub fn spawn_worker_thread<F0, F1>(thread_name: &str,
                                   thread_stack_size: usize,
                                   thread_handler: Arc<AtomicBool>,
                                   thread_waker: Arc<(AtomicBool, Mutex<()>, Condvar)>, //用于唤醒运行时所在线程的条件变量
                                   sleep_timeout: u64,                                  //休眠超时时长，单位毫秒
                                   loop_interval: Option<u64>,                          //工作者线程循环的间隔时长，None为无间隔，单位毫秒
                                   loop_func: F0,
                                   get_queue_len: F1) -> Arc<AtomicBool>
    where F0: Fn() -> (bool, Duration) + Send + 'static,
          F1: Fn() -> usize + Send + 'static {
    let thread_status_copy = thread_handler.clone();

    thread::Builder::new()
        .name(thread_name.to_string())
        .stack_size(thread_stack_size).spawn(move || {
        let mut sleep_count = 0;

        while thread_handler.load(Ordering::Relaxed) {
            let (is_no_task, run_time) = loop_func();

            if is_no_task {
                //当前没有任务
                if sleep_count > 1 {
                    //当前没有任务连续达到2次，则休眠线程
                    sleep_count = 0; //重置休眠计数
                    let (is_sleep, lock, condvar) = &*thread_waker;
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

    thread_status_copy
}

/// 唤醒工作者所在线程，如果线程当前正在运行，则忽略
pub fn wakeup_worker_thread<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>>(worker_waker: &Arc<(AtomicBool, Mutex<()>, Condvar)>, rt: &SingleTaskRuntime<O, P>) {
    //检查工作者所在线程是否需要唤醒
    if worker_waker.0.load(Ordering::Relaxed) && rt.len() > 0 {
        let (is_sleep, lock, condvar) = &**worker_waker;
        let locked = lock.lock();
        is_sleep.store(false, Ordering::SeqCst); //设置为未休眠
        let _ = condvar.notify_one();
    }
}

/// 注册全局异常处理器，会替换当前全局异常处理器
pub fn register_global_panic_handler<Handler>(handler: Handler)
    where Handler: Fn(thread::Thread, String, Option<String>, Option<(String, u32, u32)>) -> Option<i32> + Send + Sync + 'static {
    set_hook(Box::new(move |panic_info| {
        let thread_info = thread::current();

        let payload = panic_info.payload();
        let payload_info = match payload.downcast_ref::<&str>() {
            None => {
                //不是String
                match payload.downcast_ref::<String>() {
                    None => {
                        //不是&'static str，则返回未知异常
                        "Unknow panic".to_string()
                    },
                    Some(info) => {
                        info.clone()
                    }
                }
            },
            Some(info) => {
                info.to_string()
            }
        };

        let other_info = if let Some(arg) = panic_info.message() {
            if let Some(s) = arg.as_str() {
                Some(s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let location = if let Some(location) = panic_info.location() {
            Some((location.file().to_string(), location.line(), location.column()))
        } else {
            None
        };

        if let Some(exit_code) = handler(thread_info, payload_info, other_info, location) {
            //需要关闭当前进程
            std::process::exit(exit_code);
        }
    }));
}

///单调递增时间
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MontonicTime {
    Normal(Instant),    //标准单调递增时间
    Fast((i64, i64)),   //快速单调递增时间，只支持类Linux系统
}

unsafe impl Send for MontonicTime {}
unsafe impl Sync for MontonicTime {}

impl MontonicTime {
    /// 构建一个单调递增时间
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    pub fn now() -> Self {
        MontonicTime::Normal(Instant::now())
    }

    /// 构建一个单调递增时间
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn now() -> Self {
        MontonicTime::Fast(now_monotonic())
    }

    /// 获取从构建单调递增时间开始已过去的时间
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    pub fn elapsed(&self) -> Duration {
        if let MontonicTime::Normal(now) = self {
            now.elapsed()
        } else {
            panic!("Take elapsed failed, reason: invalid target os");
        }
    }

    /// 获取从构建单调递增时间开始已过去的时间，精度为ms
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn elapsed(&self) -> Duration {
        if let MontonicTime::Fast((sec0, nsec0)) = self {
            let (sec1, nsec1) = now_monotonic();
            Duration::new((sec1 - sec0) as u64, (nsec1 - nsec0) as u32)
        } else {
            panic!("Take elapsed failed, reason: invalid target os");
        }
    }
}

/// 快速获取单调时间
#[inline]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn now_monotonic() -> (i64, i64) {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC_COARSE, &mut time) };
    assert!(ret == 0);

    (time.tv_sec as i64, time.tv_nsec as i64)
}
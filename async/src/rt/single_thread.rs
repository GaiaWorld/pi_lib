//! 单线程运行时
//!

use std::any::Any;
use std::vec::IntoIter;
use std::cell::RefCell;
use std::future::Future;
use std::mem::transmute;
use std::cell::UnsafeCell;
use std::sync::{Arc, Weak};
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use parking_lot::{Mutex, Condvar};
use crossbeam_channel::{Sender, bounded, unbounded};
use flume::bounded as async_bounded;
use futures::{future::{FutureExt, BoxFuture},
              stream::{Stream, StreamExt, BoxStream},
              task::{ArcWake, waker_ref}};
use async_stream::stream;

use crate::{lock::mpsc_deque::{Sender as MpscSent, Receiver as MpscRecv, mpsc_deque},
            rt::{AsyncWaitResult, AsyncTimingTask}};
use super::{PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME,
            AsyncTaskPool,
            AsyncTaskPoolExt,
            AsyncRuntimeExt,
            TaskId,
            AsyncTask,
            AsyncRuntime,
            AsyncTaskTimer,
            AsyncWaitTimeout,
            AsyncWait,
            AsyncWaitAny,
            AsyncWaitAnyCallback,
            AsyncMapReduce,
            AsyncPipelineResult,
            alloc_rt_uid, current_async_runtime};

///
/// 单线程任务池
///
pub struct SingleTaskPool<O: Default + 'static> {
    id:             usize,                                                          //绑定的线程唯一id
    consumer:       Arc<RefCell<MpscRecv<Arc<AsyncTask<O, SingleTaskPool<O>>>>>>,   //任务消费者
    producer:       Arc<MpscSent<Arc<AsyncTask<O, SingleTaskPool<O>>>>>,            //任务生产者
    consume_count:  Arc<AtomicUsize>,                                               //任务消费计数
    produce_count:  Arc<AtomicUsize>,                                               //任务生产计数
    thread_waker:   Option<Arc<(AtomicBool, Mutex<()>, Condvar)>>,                  //绑定线程的唤醒器
}

unsafe impl<O: Default + 'static> Send for SingleTaskPool<O> {}
unsafe impl<O: Default + 'static> Sync for SingleTaskPool<O> {}

impl<O: Default + 'static> Clone for SingleTaskPool<O> {
    fn clone(&self) -> Self {
        SingleTaskPool {
            id: self.id,
            consumer: self.consumer.clone(),
            producer: self.producer.clone(),
            consume_count: self.consume_count.clone(),
            produce_count: self.produce_count.clone(),
            thread_waker: self.thread_waker.clone(),
        }
    }
}

impl<O: Default + 'static> Default for SingleTaskPool<O> {
    fn default() -> Self {
        let rt_uid = alloc_rt_uid();
        let (producer, consumer) = mpsc_deque();
        let consume_count = Arc::new(AtomicUsize::new(0));
        let produce_count = Arc::new(AtomicUsize::new(0));

        SingleTaskPool {
            id: (rt_uid << 8) & 0xffff | 1,
            consumer: Arc::new(RefCell::new(consumer)),
            producer: Arc::new(producer),
            consume_count,
            produce_count,
            thread_waker: Some(Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()))),
        }
    }
}

impl<O: Default + 'static> AsyncTaskPool<O> for SingleTaskPool<O> {
    type Pool = SingleTaskPool<O>;

    #[inline]
    fn get_thread_id(&self) -> usize {
        self.id
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
        self.producer.send(task);
        self.produce_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    #[inline]
    fn push_timed_out(&self, _index: u64, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        self.consumer.as_ref().borrow_mut().push_front(task);
        self.produce_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    #[inline]
    fn push_keep(&self, task: Arc<AsyncTask<O, Self::Pool>>) -> Result<()> {
        self.push(task)
    }

    #[inline]
    fn try_pop(&self) -> Option<Arc<AsyncTask<O, Self::Pool>>> {
        let task = self.consumer.as_ref().borrow_mut().try_recv();
        if task.is_some() {
            self.consume_count.fetch_add(1, Ordering::Relaxed);
        }
        task
    }

    #[inline]
    fn try_pop_all(&self) -> IntoIter<Arc<AsyncTask<O, Self::Pool>>> {
        let all = self.consumer.as_ref().borrow_mut().try_recv_all();
        self.consume_count.fetch_add(all.len(), Ordering::Relaxed);
        all.into_iter()
    }

    #[inline]
    fn get_thread_waker(&self) -> Option<&Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        self.thread_waker.as_ref()
    }
}

impl<O: Default + 'static> AsyncTaskPoolExt<O> for SingleTaskPool<O> {
    fn set_thread_waker(&mut self, thread_waker: Arc<(AtomicBool, Mutex<()>, Condvar)>) {
        self.thread_waker = Some(thread_waker);
    }
}

///
/// 异步单线程任务运行时
///
pub struct SingleTaskRuntime<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O> = SingleTaskPool<O>,
>(Arc<(
    usize,                                  //运行时唯一id
    Arc<P>,                                 //异步任务池
    Sender<(usize, AsyncTimingTask<O, P>)>, //休眠的异步任务生产者
    Mutex<AsyncTaskTimer<O, P>>,            //本地定时器
)>);

unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Send for SingleTaskRuntime<O, P> {}
unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Sync for SingleTaskRuntime<O, P> {}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Clone for SingleTaskRuntime<O, P> {
    fn clone(&self) -> Self {
        SingleTaskRuntime(self.0.clone())
    }
}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> AsyncRuntimeExt<O> for SingleTaskRuntime<O, P> {
    fn spawn_with_context<F, C>(&self,
                                task_id: TaskId,
                                future: F,
                                context: C) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static {
        let boxed = Box::new(future).boxed();
        if let Err(e) = (self.0)
            .1
            .push(Arc::new(AsyncTask::with_context(task_id,
                                                   (self.0).1.clone(),
                                                   Some(boxed),
                                                   context))) {
            return Err(Error::new(ErrorKind::Other, e));
        }

        Ok(())
    }

    fn spawn_timing_with_context<F, C>(&self,
                                       task_id: TaskId,
                                       future: F,
                                       context: C,
                                       time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static,
              C: 'static {
        let boxed = Box::new(future).boxed();
        (self.0)
            .3
            .lock()
            .set_timer(AsyncTimingTask::WaitRun(Arc::new(AsyncTask::with_context(task_id.clone(),
                                                                                 (self.0).1.clone(),
                                                                                 Some(boxed),
                                                                                 context))), time);

        Ok(())
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
* 异步单线程任务运行时同步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> SingleTaskRuntime<O, P> {
    /// 共享运行时内部任务池
    pub(crate) fn shared_pool(&self) -> Arc<P> {
        (self.0).1.clone()
    }

    /// 获取当前运行时的唯一id
    pub fn get_id(&self) -> usize {
        (self.0).0
    }

    /// 获取当前运行时任务数量
    pub fn len(&self) -> usize {
        (self.0).1.len()
    }

    /// 获取当前运行时中剩余未到期的定时任务数量
    pub fn timing_len(&self) -> usize {
        (self.0).3.lock().len()
    }

    /// 分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        TaskId(Arc::new(AtomicUsize::new(0)))
    }

    /// 派发一个指定的异步任务到异步单线程运行时
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let boxed = Box::new(future).boxed();
        if let Err(e) = (self.0).1.push(Arc::new(AsyncTask::new(task_id, (self.0).1.clone(), Some(boxed)))) {
            return Err(Error::new(ErrorKind::Other, e));
        }

        Ok(())
    }

    /// 派发一个在指定时间后执行的异步任务到异步单线程运行时，时间单位ms
    pub fn spawn_timing<F>(&self, task_id: TaskId, future: F, time: usize) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let boxed = Box::new(future).boxed();
        (self.0).3.lock().set_timer(AsyncTimingTask::WaitRun(Arc::new(AsyncTask::new(task_id.clone(), (self.0).1.clone(), Some(boxed)))), time);

        Ok(())
    }

    /// 挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: &TaskId, waker: Waker) -> Poll<Output> {
        task_id.0.store(Box::into_raw(Box::new(waker)) as usize, Ordering::Relaxed);
        Poll::Pending
    }

    /// 唤醒指定唯一id的异步任务
    pub fn wakeup(&self, task_id: &TaskId) {
        match task_id.0.load(Ordering::Relaxed) {
            0 => panic!("Single runtime wakeup task failed, reason: task id not exist"),
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
* 异步单线程任务运行时异步方法
*/
impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> SingleTaskRuntime<O, P> {
    /// 挂起当前单线程运行时的当前任务，等待指定的时间后唤醒当前任务
    pub async fn wait_timeout(&self, timeout: usize) {
        AsyncWaitTimeout::new(AsyncRuntime::Local(self.clone()), (self.0).2.clone(), timeout).await
    }

    /// 挂起当前单线程运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, RP, V, F>(&self, rt: AsyncRuntime<R, RP>, future: F) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        AsyncWait::new(AsyncRuntime::Local(self.clone()), rt, Some(Box::new(future).boxed())).await
    }

    /// 挂起当前单线程运行时的当前任务，并在多个其它运行时上执行多个其它任务，其中任意一个任务完成，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成的任务的值将被忽略
    pub async fn wait_any<R, RP, V>(&self,
                                    futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static {
        AsyncWaitAny::new(AsyncRuntime::Local(self.clone()), futures).await
    }

    /// 挂起当前单线程运行时的当前任务，并在多个其它运行时上执行多个其它任务，任务返回后需要通过用户指定的检查回调进行检查，其中任意一个任务检查通过，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成或未检查通过的任务的值将被忽略，如果所有任务都未检查通过，则强制唤醒当前运行时的当前任务
    pub async fn wait_any_callback<R, RP, V, F>(&self,
                                                futures: Vec<(AsyncRuntime<R, RP>, BoxFuture<'static, Result<V>>)>,
                                                callback: F) -> Result<V>
        where R: Default + 'static,
              RP: AsyncTaskPoolExt<R> + AsyncTaskPool<R, Pool = RP>,
              V: Send + 'static,
              F: Fn(&Result<V>) -> bool + Send + Sync + 'static {
        AsyncWaitAnyCallback::new(AsyncRuntime::Local(self.clone()), futures, Some(callback)).await
    }
}

///
/// 单线程异步任务执行器
///
pub struct SingleTaskRunner<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O> = SingleTaskPool<O>
> {
    is_running: AtomicBool,                 //是否开始运行
    runtime:    SingleTaskRuntime<O, P>,    //异步单线程任务运行时
}

unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Send for SingleTaskRunner<O, P> {}
unsafe impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
> Sync for SingleTaskRunner<O, P> {}

impl<O: Default + 'static> Default for SingleTaskRunner<O> {
    fn default() -> Self {
        SingleTaskRunner::new(SingleTaskPool::default())
    }
}

impl<
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
> SingleTaskRunner<O, P> {
    /// 用指定的任务池构建单线程异步运行时
    pub fn new(pool: P) -> Self {
        let rt_uid = pool.get_thread_id();
        let pool = Arc::new(pool);

        //构建本地定时器和定时异步任务生产者
        let timer = AsyncTaskTimer::new();
        let producor = timer.producor.clone();
        let timer = Mutex::new(timer);

        //构建单线程任务运行时
        let runtime = SingleTaskRuntime(Arc::new((
            rt_uid,
            pool,
            producor,
            timer,
        )));

        SingleTaskRunner {
            is_running: AtomicBool::new(false),
            runtime,
        }
    }

    /// 获取单线程异步任务执行器的线程唤醒器
    pub fn get_thread_waker(&self) -> Option<Arc<(AtomicBool, Mutex<()>, Condvar)>> {
        (self.runtime.0).1.get_thread_waker().cloned()
    }

    /// 启动单线程异步任务执行器
    pub fn startup(&self) -> Option<SingleTaskRuntime<O, P>> {
        match self
            .is_running
            .compare_exchange_weak(false,
                                   true,
                                   Ordering::SeqCst,
                                   Ordering::SeqCst) {
            Ok(false) => {
                //未启动，则启动，并返回单线程异步运行时
                Some(self.runtime.clone())
            }
            _ => {
                //已启动，则忽略
                None
            }
        }
    }

    /// 绑定单线程异步任务执行器到本地线程
    pub fn bind_local_thread(&self, thread_handelr: Option<Arc<AtomicBool>>) {
        match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME.try_with(move |rt| {
            let runtime = if let Some(thread_handelr) = thread_handelr {
                //设置了线程句柄
                if let Some(thread_waker) = (self.runtime.0).1.get_thread_waker() {
                    //当前线程的唤醒器存在，则构建工作者异步运行时
                    AsyncRuntime::Worker(thread_handelr, thread_waker.clone(), self.runtime.clone())
                } else {
                    //当前线程的唤醒器不存在，则构建本地异步运行时
                    AsyncRuntime::Local(self.runtime.clone())
                }
            } else {
                //未设置线程句柄，则构建本地异步运行时
                AsyncRuntime::Local(self.runtime.clone())
            };

            unsafe { *rt.get() = transmute(Arc::new(runtime) as Arc<dyn Any>); }
        }) {
            Err(e) => {
                panic!("Bind single runtime to local thread failed, reason: {:?}", e);
            },
            Ok(_) => (),
        }
    }

    /// 从本地线程解绑单线程异步任务执行器
    pub fn unbind_local_thread(&self) {
        match PI_ASYNC_LOCAL_THREAD_ASYNC_RUNTIME.try_with(move |rt| {
            unsafe {
                let ptr = *rt.get();
                if ptr == (0, 0) {
                    //本地线程未绑定异步运行时，则忽略
                    return;
                } else {
                    //本地线程已绑定异步运行时，则解除绑定，并释放绑定的异步运行时
                    let _: Arc<dyn Any + Send + Sync> = unsafe { transmute(ptr) };
                    *rt.get() = (0, 0)
                }
            }
        }) {
            Err(e) => (), //未绑定则忽略
            Ok(_) => (),
        }
    }

    /// 运行一次单线程异步任务执行器，返回当前任务池中任务的数量
    pub fn run_once(&self) -> Result<usize> {
        if !self.is_running.load(Ordering::Relaxed) {
            //未启动，则返回错误原因
            return Err(Error::new(ErrorKind::Other, "Single thread runtime not running"));
        }

        //设置新的定时任务，并唤醒已过期的定时任务
        (self.runtime.0).3.lock().consume(); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
        loop {
            let current_time = (self.runtime.0).3.lock().is_require_pop(); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
            if let Some(current_time) = current_time {
                //当前有到期的定时异步任务，则只处理到期的一个定时异步任务
                let timed_out = (self.runtime.0).3.lock().pop(current_time); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
                if let Some((handle, timing_task)) = timed_out {
                    match timing_task {
                        AsyncTimingTask::Pended(expired) => {
                            //唤醒休眠的异步任务，并立即执行
                            self.runtime.wakeup(&expired);
                            if let Some(task) = (self.runtime.0).1.try_pop() {
                                run_task(task);
                            }
                        },
                        AsyncTimingTask::WaitRun(expired) => {
                            //立即执行到期的定时异步任务，并立即执行
                            (self.runtime.0).1.push_timed_out(handle as u64, expired);
                            if let Some(task) = (self.runtime.0).1.try_pop() {
                                run_task(task);
                            }
                        },
                    }
                }
            } else {
                //当前没有到期的定时异步任务，则退出本次定时异步任务处理
                break;
            }
        }

        //继续执行当前任务池中的一个异步任务
        match (self.runtime.0).1.try_pop() {
            None => {
                //当前没有异步任务，则立即返回
                return Ok(0);
            },
            Some(task) => {
                run_task(task);
            },
        }

        Ok((self.runtime.0).1.len())
    }

    /// 运行单线程异步任务执行器，并执行任务池中的所有任务
    pub fn run(&self) -> Result<usize> {
        if !self.is_running.load(Ordering::Relaxed) {
            //未启动，则返回错误原因
            return Err(Error::new(ErrorKind::Other, "Single thread runtime not running"));
        }

        //获取当前任务池中的所有异步任务
        let mut tasks = (self.runtime.0).1.try_pop_all();

        //设置新的定时任务，并唤醒已过期的定时任务
        (self.runtime.0).3.lock().consume(); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
        loop {
            let current_time = (self.runtime.0).3.lock().is_require_pop(); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
            if let Some(current_time) = current_time {
                //当前有到期的定时异步任务，则开始处理到期的所有定时异步任务
                loop {
                    let timed_out = (self.runtime.0).3.lock().pop(current_time); //运行时内部的锁临界区要尽可能的小，避免出现锁重入
                    if let Some((handle, timing_task)) = timed_out {
                        match timing_task {
                            AsyncTimingTask::Pended(expired) => {
                                //唤醒休眠的异步任务，并立即执行
                                self.runtime.wakeup(&expired);
                                if let Some(task) = (self.runtime.0).1.try_pop() {
                                    run_task(task);
                                }
                            },
                            AsyncTimingTask::WaitRun(expired) => {
                                //立即执行到期的定时异步任务，并立即执行
                                (self.runtime.0).1.push_timed_out(handle as u64, expired);
                                if let Some(task) = (self.runtime.0).1.try_pop() {
                                    run_task(task);
                                }
                            },
                        }

                        if let Some(task) = tasks.next() {
                            //执行当前所有异步任务中的一个异步任务，避免定时异步任务占用当前运行时的所有执行时间
                            run_task(task);
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

        loop {
            //继续执行剩余的异步任务
            if let Some(task) = tasks.next() {
                run_task(task);
            } else {
                //没有需要获取的定时任务，且当前异步任务池中的任务已执行完，则退出
                return Ok((self.runtime.0).1.len());
            }
        }
    }
}

//执行异步任务
#[inline]
fn run_task<O: Default + 'static, P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>>(task: Arc<AsyncTask<O, P>>) {
    let waker = waker_ref(&task);
    let mut context = Context::from_waker(&*waker);
    if let Some(mut future) = task.get_inner() {
        if let Poll::Pending = future.as_mut().poll(&mut context) {
            //当前未准备好，则恢复异步任务，以保证异步服务后续访问异步任务和异步任务不被提前释放
            task.set_inner(Some(future));
        }
    }
}

#[test]
fn test_single_runtime() {
    use std::mem;
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::rt::{spawn_local,
                    get_local_dict,
                    get_local_dict_mut,
                    set_local_dict,
                    remove_local_dict,
                    clear_local_dict};

    struct AtomicCounter(AtomicUsize, Instant);
    impl Drop for AtomicCounter {
        fn drop(&mut self) {
            unsafe {
                println!("!!!!!!drop counter, count: {:?}, time: {:?}", self.0.load(Ordering::Relaxed), Instant::now() - self.1);
            }
        }
    }

    let rt_uid = alloc_rt_uid();
    let (producer, consumer) = mpsc_deque();
    let consume_count = Arc::new(AtomicUsize::new(0));
    let produce_count = Arc::new(AtomicUsize::new(0));

    let pool = SingleTaskPool {
        id: (rt_uid << 8) & 0xffff | 1,
        consumer: Arc::new(RefCell::new(consumer)),
        producer: Arc::new(producer),
        consume_count,
        produce_count,
        thread_waker: None,
    };

    let runner = SingleTaskRunner::new(pool);
    let rt = runner.startup().unwrap();
    let rt0 = rt.clone();
    let rt1 = rt.clone();
    let rt2 = rt.clone();
    let rt3 = rt.clone();

    thread::spawn(move || {
        runner.bind_local_thread(None);

        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    rt.spawn(rt.alloc(), async move {
        if let Err(e) = spawn_local::<SingleTaskPool<()>, _>(async move {
            println!("Test spawn local ok");
        }) {
            println!("Test spawn local failed, reason: {:?}", e);
        }
    });

    let rt_copy = rt.clone();
    let thread_handle = thread::spawn(move || {
        match rt_copy.block_on::<SingleTaskPool<String>, _>(async move {
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

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let counter0 = counter.clone();
    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let counter3 = counter.clone();
    mem::drop(counter);

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let counter_copy = counter0.clone();
            if let Err(e) = rt0.spawn(rt0.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn singale task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn single task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let counter_copy = counter1.clone();
            if let Err(e) = rt1.spawn(rt1.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn singale task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn single task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let counter_copy = counter2.clone();
            if let Err(e) = rt2.spawn(rt2.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn singale task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn single task ok, time: {:?}", Instant::now() - start);
    });

    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let counter_copy = counter3.clone();
            if let Err(e) = rt3.spawn(rt3.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn singale task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn single task ok, time: {:?}", Instant::now() - start);
    });

    thread::sleep(Duration::from_millis(1000000000));
}







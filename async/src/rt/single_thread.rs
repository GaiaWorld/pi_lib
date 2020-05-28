use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crossbeam_channel::{Sender, Receiver, unbounded};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};

use crate::AsyncTask;
use super::{TaskId, AsyncRuntime, AsyncTaskTimer, AsyncWaitTimeout, AsyncWait, AsyncWaitAny, AsyncMap, alloc_rt_uid};
use crate::rt::AsyncWaitResult;

/*
* 单线程任务
*/
pub struct SingleTask<O: Default + 'static> {
    uid:    TaskId,
    future: UnsafeCell<Option<BoxFuture<'static, O>>>,
    queue:  Arc<SingleTasks<O>>,
}

unsafe impl<O: Default + 'static> Send for SingleTask<O> {}
unsafe impl<O: Default + 'static> Sync for SingleTask<O> {}

impl<O: Default + 'static> ArcWake for SingleTask<O> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _ = arc_self.queue.push_back(arc_self.clone());
    }
}

impl<O: Default + 'static> AsyncTask for SingleTask<O> {
    type Out = O;

    fn get_inner(&self) -> Option<BoxFuture<'static, Self::Out>> {
        unsafe { (*self.future.get()).take() }
    }

    fn set_inner(&self, inner: Option<BoxFuture<'static, Self::Out>>) {
        unsafe { *self.future.get() = inner; }
    }
}

impl<O: Default + 'static> SingleTask<O> {
    //构建单线程任务
    pub fn new(uid: TaskId, queue: Arc<SingleTasks<O>>, future: Option<BoxFuture<'static, O>>) -> SingleTask<O> {
        SingleTask {
            uid,
            future: UnsafeCell::new(future),
            queue,
        }
    }

    //检查是否允许唤醒
    pub fn is_enable_wakeup(&self) -> bool {
        self.uid.0.load(Ordering::Relaxed) > 0
    }
}

/*
* 单线程任务队列
*/
pub struct SingleTasks<O: Default + 'static> {
    id:             usize,                          //绑定的线程唯一id
    consumer:       Receiver<Arc<SingleTask<O>>>,   //任务消费者
    producer:       Sender<Arc<SingleTask<O>>>,     //任务生产者
}

unsafe impl<O: Default + 'static> Send for SingleTasks<O> {}
unsafe impl<O: Default + 'static> Sync for SingleTasks<O> {}

impl<O: Default + 'static> Clone for SingleTasks<O> {
    fn clone(&self) -> Self {
        SingleTasks {
            id: self.id,
            consumer: self.consumer.clone(),
            producer: self.producer.clone(),
        }
    }
}

impl<O: Default + 'static> SingleTasks<O> {
    //获取任务数量
    #[inline]
    pub fn len(&self) -> usize {
        self.consumer.len()
    }

    //向单线程任务队列推入指定的任务
    pub fn push_back(&self, task: Arc<SingleTask<O>>) -> Result<()> {
        if let Err(e) = self.producer.send(task) {
            return Err(Error::new(ErrorKind::Other, format!("Push SingleTask Failed, reason: {:?}", e)));
        }

        Ok(())
    }
}

/*
* 异步单线程任务运行时
*/
pub struct SingleTaskRuntime<O: Default + 'static>(Arc<(
    usize,                                              //运行时唯一id
    Arc<SingleTasks<O>>,                                //异步任务队列
    AsyncTaskTimer,                                     //本地定时器
)>);

unsafe impl<O: Default + 'static> Send for SingleTaskRuntime<O> {}
unsafe impl<O: Default + 'static> Sync for SingleTaskRuntime<O> {}

impl<O: Default + 'static> Clone for SingleTaskRuntime<O> {
    fn clone(&self) -> Self {
        SingleTaskRuntime(self.0.clone())
    }
}

/*
* 异步单线程任务运行时同步方法
*/
impl<O: Default + 'static> SingleTaskRuntime<O> {
    //获取当前运行时的唯一id
    pub fn get_id(&self) -> usize {
        (self.0).0
    }

    //获取当前运行时待处理任务数量
    pub fn wait_len(&self) -> usize {
        (self.0).1.consumer.len()
    }

    //获取当前运行时任务数量
    pub fn len(&self) -> usize {
        (self.0).1.len()
    }

    //分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        TaskId(Arc::new(AtomicUsize::new(0)))
    }

    //派发一个指定的异步任务到异步单线程运行时
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let queue = (self.0).1.clone();
        let boxed = Box::new(future).boxed();
        if let Err(e) = (self.0).1.push_back(Arc::new(SingleTask::new(task_id, queue, Some(boxed)))) {
            return Err(Error::new(ErrorKind::Other, e));
        }

        Ok(())
    }

    //挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: &TaskId, waker: Waker) -> Poll<Output> {
        task_id.0.store(Box::into_raw(Box::new(waker)) as usize, Ordering::Relaxed);
        Poll::Pending
    }

    //唤醒指定唯一id的异步任务
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

    //构建用于派发多个异步任务到指定运行时的映射
    pub fn map<V: Send + 'static>(&self) -> AsyncMap<O, V> {
        let (producor, consumer) = unbounded();

        AsyncMap {
            count: 0,
            futures: Vec::new(),
            producor,
            consumer,
        }
    }
}

/*
* 异步单线程任务运行时异步方法
*/
impl<O: Default + 'static> SingleTaskRuntime<O> {
    //挂起当前单线程运行时的当前任务，等待指定的时间后唤醒当前任务
    pub async fn wait_timeout(&self, timeout: usize) {
        AsyncWaitTimeout::new(AsyncRuntime::Single(self.clone()), (self.0).2.get_producor(), timeout).await
    }

    //挂起当前单线程运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, V, F>(&self, rt: AsyncRuntime<R>, future: F) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        AsyncWait::new(AsyncRuntime::Single(self.clone()), rt, Some(Box::new(future).boxed())).await
    }

    //挂起当前单线程运行时的当前任务，并在多个其它运行时上执行多个其它任务，其中任意一个任务完成，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成的任务的值将被忽略
    pub async fn wait_any<R, V>(&self, futures: Vec<(AsyncRuntime<R>, BoxFuture<'static, Result<V>>)>) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static  {
        AsyncWaitAny::new(AsyncRuntime::Single(self.clone()), futures).await
    }
}

/*
* 单线程异步任务执行器
*/
pub struct SingleTaskRunner<O: Default + 'static> {
    is_running: AtomicBool,             //是否开始运行
    runtime:    SingleTaskRuntime<O>,   //异步单线程任务运行时
}

unsafe impl<O: Default + 'static> Send for SingleTaskRunner<O> {}
unsafe impl<O: Default + 'static> Sync for SingleTaskRunner<O> {}

impl<O: Default + 'static> SingleTaskRunner<O> {
    //构建单线程异步任务执行器
    pub fn new() -> Self {
        //构建单线程任务队列
        let rt_uid = alloc_rt_uid();
        let (producer, consumer) = unbounded();
        let queue = Arc::new(SingleTasks {
            id: (rt_uid << 8) & 0xffff | 1,
            consumer,
            producer,
        });

        //构建本地定时器
        let timer = AsyncTaskTimer::new();

        //构建单线程任务运行时
        let runtime = SingleTaskRuntime(Arc::new((
            rt_uid,
            queue,
            timer,
        )));

        SingleTaskRunner {
            is_running: AtomicBool::new(false),
            runtime,
        }
    }

    //启动单线程异步任务执行器
    pub fn startup(&self) -> Option<SingleTaskRuntime<O>> {
        if self.is_running.compare_and_swap(false, true, Ordering::SeqCst) {
            //已启动，则忽略
            return None;
        }

        Some(self.runtime.clone())
    }

    //运行一次单线程异步任务执行器，返回当前任务队列中任务的数量
    pub fn run_once(&self) -> Result<usize> {
        if !self.is_running.load(Ordering::Relaxed) {
            //未启动，则返回错误原因
            return Err(Error::new(ErrorKind::Other, "single thread runtime not running"));
        }

        //设置新的定时任务，并唤醒已过期的定时任务
        (self.runtime.0).2.consume();
        for expired in &(self.runtime.0).2.poll() {
            self.runtime.wakeup(expired);
        }

        //执行异步任务
        for task in (self.runtime.0).1.consumer.try_iter() {
            run_task(task);
        }

        Ok((self.runtime.0).1.consumer.len())
    }
}

//执行异步任务
fn run_task<O: Default + 'static>(task: Arc<SingleTask<O>>) {
    let waker = waker_ref(&task);
    let mut context = Context::from_waker(&*waker);
    if let Some(mut future) = task.get_inner() {
        if let Poll::Pending = future.as_mut().poll(&mut context) {
            //当前未准备好，则恢复异步任务，以保证异步服务后续访问异步任务和异步任务不被提前释放
            task.set_inner(Some(future));
        }
    }
}
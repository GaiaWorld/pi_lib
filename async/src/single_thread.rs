use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::time::Duration;
use std::cell::{UnsafeCell, RefCell};
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crossbeam_channel::{Sender, Receiver, unbounded};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};
use twox_hash::RandomXxHashBuilder64;
use dashmap::DashMap;

use crate::{AsyncTask, TaskId, AsyncRuntime};

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
        let task = arc_self.clone();
        arc_self.queue.producer.send(task);
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
}

/*
* 单线程任务队列
*/
pub struct SingleTasks<O: Default + 'static> {
    consumer:       Receiver<Arc<SingleTask<O>>>,   //任务消费者
    producer:       Sender<Arc<SingleTask<O>>>,     //任务生产者
}

unsafe impl<O: Default + 'static> Send for SingleTasks<O> {}
unsafe impl<O: Default + 'static> Sync for SingleTasks<O> {}

impl<O: Default + 'static> Clone for SingleTasks<O> {
    fn clone(&self) -> Self {
        SingleTasks {
            consumer: self.consumer.clone(),
            producer: self.producer.clone(),
        }
    }
}

impl<O: Default + 'static> SingleTasks<O> {
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
    AtomicUsize,                                    //异步任务id生成器
    AtomicUsize,                                    //异步服务id生成器
    Arc<SingleTasks<O>>,                            //异步任务队列
    DashMap<usize, Waker, RandomXxHashBuilder64>,   //异步任务等待表
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
    //分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        TaskId((self.0).0.fetch_add(1, Ordering::Relaxed))
    }

    //派发一个指定的异步任务到异步单线程任务池，返回异步任务的唯一id
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let queue = (self.0).2.clone();
        let boxed = Box::new(future).boxed();
        if let Err(e) = (self.0).2.producer.send(Arc::new(SingleTask::new(task_id, queue, Some(boxed)))) {
            return Err(Error::new(ErrorKind::Other, e));
        }

        Ok(())
    }

    //挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: TaskId, waker: Waker) -> Poll<Output> {
        (self.0).3.insert(task_id.0, waker);
        Poll::Pending
    }

    //唤醒执行指定唯一id的异步任务
    pub fn wakeup(&self, task_id: TaskId) {
        if let Some((_, waker)) = (self.0).3.remove(&task_id.0) {
            waker.wake();
        }
    }
}

/*
* 异步单线程任务运行时异步方法
*/
impl<O: Default + 'static> SingleTaskRuntime<O> {
    //挂起当前单线程运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, V, F>(&self, rt: AsyncRuntime<R>, future: F) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        AsyncWait::new(self.clone(), rt, Some(Box::new(future).boxed())).await
    }
}

/*
* 等待异步任务运行的结果
*/
struct AsyncWaitResult<V: Send + 'static>(Arc<RefCell<Option<Result<V>>>>);

unsafe impl<V: Send + 'static> Send for AsyncWaitResult<V> {}
unsafe impl<V: Send + 'static> Sync for AsyncWaitResult<V> {}

impl<V: Send + 'static> Clone for AsyncWaitResult<V> {
    fn clone(&self) -> Self {
        AsyncWaitResult(self.0.clone())
    }
}

/*
* 等待异步任务执行完成
*/
struct AsyncWait<O: Default + 'static, R: Default + 'static, V: Send + 'static> {
    wait:   SingleTaskRuntime<O>,                   //需要等待的异步运行时
    runner: AsyncRuntime<R>,                        //需要运行的异步运行时
    future: Option<BoxFuture<'static, Result<V>>>,  //需要等待执行的异步任务
    result: AsyncWaitResult<V>,                     //需要等待执行的异步任务的结果
}

unsafe impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Send for AsyncWait<O, R, V> {}
unsafe impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Sync for AsyncWait<O, R, V> {}

impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> Future for AsyncWait<O, R, V> {
    type Output = Result<V>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //任务已完成，则返回
            return Poll::Ready(result);
        }

        //在指定运行时运行指定的任务
        let task_id = self.as_ref().wait.alloc();
        let wait = self.as_ref().wait.clone();
        let runner = self.as_ref().runner.clone();
        let future = self.as_mut().future.take();
        let result = self.as_ref().result.clone();
        let task = async move {
            let wait_copy = wait.clone();
            let result_copy = result.clone();
            match runner {
                AsyncRuntime::Single(rt) => {
                    //将指定任务派发到单线程运行时
                    if let Err(e) = rt.spawn(rt.alloc(), async move {
                        if let Some(f) = future {
                            //指定了任务
                            *result_copy.0.borrow_mut() = Some(f.await);
                        } else {
                            //未指定任务
                            *result_copy.0.borrow_mut() = Some(Err(Error::new(ErrorKind::NotFound, "invalid future")));
                        }

                        wait_copy.wakeup(task_id);

                        //返回异步任务的默认值
                        Default::default()
                    }) {
                        //派发指定的任务失败，则立即唤醒等待的任务
                        *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async SingleTask Runner Error, reason: {:?}", e))));
                        wait.wakeup(task_id);
                    }
                },
                AsyncRuntime::Multi(rt) => {
                    //将指定任务派发到多线程运行时
                    if let Err(e) = rt.spawn(rt.alloc(), async move {
                        if let Some(f) = future {
                            //指定了任务
                            *result_copy.0.borrow_mut() = Some(f.await);
                        } else {
                            //未指定任务
                            *result_copy.0.borrow_mut() = Some(Err(Error::new(ErrorKind::NotFound, "invalid future")));
                        }

                        wait_copy.wakeup(task_id);

                        //返回异步任务的默认值
                        Default::default()
                    }) {
                        //派发指定的任务失败，则立即唤醒等待的任务
                        *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async MultiTask Runner Error, reason: {:?}", e))));
                        wait.wakeup(task_id);
                    }
                },
            }

            //返回异步任务的默认值
            Default::default()
        };
        if let Err(e) = self.as_ref().wait.spawn(task_id, task) {
            //派发异步等待的任务失败，则立即返回错误原因
            return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Error, reason: {:?}", e))));
        }

        //挂起当前任务，并返回值未就绪
        self.as_ref().wait.pending(task_id, cx.waker().clone())
    }
}

impl<O: Default + 'static, R: Default + 'static, V: Send + 'static> AsyncWait<O, R, V> {
    //构建等待异步任务执行完成的方法
    fn new(wait: SingleTaskRuntime<O>,
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
        let (producer, consumer) = unbounded();
        let queue = Arc::new(SingleTasks {
            consumer,
            producer,
        });

        //构建单线程任务运行时
        let runtime = SingleTaskRuntime(Arc::new((
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            queue,
            DashMap::with_hasher(Default::default()),
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

        for task in (self.runtime.0).2.consumer.try_iter() {
            run_task(task);
        }

        Ok((self.runtime.0).2.consumer.len())
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
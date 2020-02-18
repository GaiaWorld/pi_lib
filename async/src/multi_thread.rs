use std::thread;
use std::future::Future;
use std::time::Duration;
use std::cell::UnsafeCell;
use std::thread::{Builder, Thread};
use std::sync::{Arc, Mutex, Condvar};
use std::task::{Waker, Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_channel::{Sender, Receiver, unbounded};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};
use twox_hash::RandomXxHashBuilder64;
use dashmap::DashMap;

use crate::AsyncTask;

/*
* 多线程任务唯一id
*/
#[derive(Debug, Clone, Copy)]
pub struct TaskId(usize);

/*
* 多线程任务
*/
pub struct MultiTask<O: Default + 'static> {
    uid:    TaskId,
    future: UnsafeCell<Option<BoxFuture<'static, O>>>,
    queue:  Arc<MultiTasks<O>>,
}

unsafe impl<O: Default + 'static> Send for MultiTask<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTask<O> {}

impl<O: Default + 'static> ArcWake for MultiTask<O> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = arc_self.clone();
        arc_self.queue.producer.send(task);
    }
}

impl<O: Default + 'static> AsyncTask for MultiTask<O> {
    type Out = O;

    fn get_inner(&self) -> Option<BoxFuture<'static, Self::Out>> {
        unsafe { (*self.future.get()).take() }
    }

    fn set_inner(&self, inner: Option<BoxFuture<'static, Self::Out>>) {
        unsafe { *self.future.get() = inner; }
    }
}

impl<O: Default + 'static> MultiTask<O> {
    //构建多线程任务
    pub fn new(uid: TaskId, queue: Arc<MultiTasks<O>>, future: Option<BoxFuture<'static, O>>) -> MultiTask<O> {
        MultiTask {
            uid,
            future: UnsafeCell::new(future),
            queue,
        }
    }
}

/*
* 多线程任务队列
*/
pub struct MultiTasks<O: Default + 'static> {
    consumer:       Receiver<Arc<MultiTask<O>>>,    //任务消费者
    producer:       Sender<Arc<MultiTask<O>>>,      //任务生产者
    worker_waker:   Arc<(Mutex<bool>, Condvar)>,    //工作者唤醒器
}

unsafe impl<O: Default + 'static> Send for MultiTasks<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTasks<O> {}

impl<O: Default + 'static> Clone for MultiTasks<O> {
    fn clone(&self) -> Self {
        MultiTasks {
            consumer: self.consumer.clone(),
            producer: self.producer.clone(),
            worker_waker: self.worker_waker.clone(),
        }
    }
}

impl<O: Default + 'static> MultiTasks<O> {
    //向多线程任务队列推入指定的任务，并唤醒休眠的工作者
    pub fn push_back(&self, task: Arc<MultiTask<O>>) -> Result<()> {
        if let Err(e) = self.producer.send(task) {
            return Err(Error::new(ErrorKind::Other, format!("Push MultiTask Failed, reason: {:?}", e)));
        }

        //唤醒工作者
        let (lock, cvar) = &**&self.worker_waker;
        let mut status = lock.lock().unwrap();
        *status = true;
        cvar.notify_one();

        Ok(())
    }
}

/*
* 异步多线程任务运行时
*/
pub struct MultiTaskRuntime<O: Default + 'static>(Arc<(
    AtomicUsize,                                    //异步任务id生成器
    AtomicUsize,                                    //异步服务id生成器
    Arc<MultiTasks<O>>,                             //异步任务队列
    DashMap<usize, Waker, RandomXxHashBuilder64>,   //异步任务等待表
)>);

unsafe impl<O: Default + 'static> Send for MultiTaskRuntime<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTaskRuntime<O> {}

impl<O: Default + 'static> Clone for MultiTaskRuntime<O> {
    fn clone(&self) -> Self {
        MultiTaskRuntime(self.0.clone())
    }
}

impl<O: Default + 'static> MultiTaskRuntime<O> {
    //分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        TaskId((self.0).0.fetch_add(1, Ordering::Relaxed))
    }

    //派发一个指定的异步任务到异步多线程任务池，返回异步任务的唯一id
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let queue = (self.0).2.clone();
        let boxed = Box::new(future).boxed();
        if let Err(e) = (self.0).2.producer.send(Arc::new(MultiTask::new(task_id, queue, Some(boxed)))) {
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
* 异步多线程任务池
*/
pub struct MultiTaskPool<O: Default + 'static> {
    runtime:        MultiTaskRuntime<O>,            //异步多线程任务运行时
    worker_waker:   Arc<(Mutex<bool>, Condvar)>,    //工作者唤醒器
    timeout:        u64,                            //工作者空闲时最长休眠时间
    builders:       Vec<Builder>,                   //线程构建器列表
}

unsafe impl<O: Default + 'static> Send for MultiTaskPool<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTaskPool<O> {}

impl<O: Default + 'static> MultiTaskPool<O> {
    //构建指定线程名前缀、线程数量、线程栈大小和线程空闲时最长休眠时间的多线程任务池
    pub fn new(prefix: String, size: usize, stack_size: usize, timeout: u64) -> Self {
        let mut builders = Vec::with_capacity(size);
        for idx in 0..size {
            let builder = Builder::new()
                .name(prefix.to_string() + "-" + idx.to_string().as_str())
                .stack_size(stack_size);
            builders.push(builder);
        }

        //构建工作者唤醒器
        let worker_waker = Arc::new((Mutex::new(false), Condvar::new()));

        //构建多线程任务队列
        let (producer, consumer) = unbounded();
        let queue = Arc::new(MultiTasks {
            consumer,
            producer,
            worker_waker: worker_waker.clone(),
        });

        //构建多线程任务运行时
        let runtime = MultiTaskRuntime(Arc::new((
            AtomicUsize::new(0),
            AtomicUsize::new(0),
            queue,
            DashMap::with_hasher(Default::default()),
        )));

        MultiTaskPool {
            runtime,
            worker_waker,
            timeout,
            builders,
        }
    }

    //启动异步多线程任务池
    pub fn startup(mut self) -> MultiTaskRuntime<O> {
        for idx in 0..self.builders.len() {
            let builder = self.builders.remove(0);
            let runtime = self.runtime.clone();
            let worker_waker = self.worker_waker.clone();
            let timeout = self.timeout;
            builder.spawn(move || {
                work_loop(runtime, worker_waker, timeout);
            });
        }

        self.runtime
    }
}

//线程工作循环
fn work_loop<O: Default + 'static>(runtime: MultiTaskRuntime<O>,
                         worker_waker: Arc<(Mutex<bool>, Condvar)>,
                         timeout: u64) {
    let worker_waker_ref = &worker_waker;
    loop {
        match (runtime.0).2.consumer.try_recv() {
            Err(ref e) if e.is_disconnected() => {
                //多线程任务队列已关闭，则立即退出当前线程工作循环
                break;
            },
            Err(_) => {
                //当前没有任务
                let (lock, cvar) = &**worker_waker_ref;
                let mut status = lock.lock().unwrap();
                while !*status {
                    //让当前工作者休眠，等待有任务时被唤醒或超时后自动唤醒
                    let (new_status, wait) = cvar.wait_timeout(status, Duration::from_millis(timeout)).unwrap();

                    if wait.timed_out() {
                        //超时后自动唤醒，则更新工作者唤醒状态，并退出唤醒状态的检查
                        status = new_status;
                        break;
                    } else {
                        //有任务时被唤醒，则更新工作者唤醒状态，并继续唤醒状态的检查
                        status = new_status;
                    }
                }

                *status = false; //重置工作者唤醒状态，并继续工作
            },
            Ok(task) => {
                run_task(task);
            },
        }
    }
}

//执行异步任务
fn run_task<O: Default + 'static>(task: Arc<MultiTask<O>>) {
    let waker = waker_ref(&task);
    let mut context = Context::from_waker(&*waker);
    if let Some(mut future) = task.get_inner() {
        if let Poll::Pending = future.as_mut().poll(&mut context) {
            //当前未准备好，则恢复异步任务，以保证异步服务后续访问异步任务和异步任务不被提前释放
            task.set_inner(Some(future));
        }
    }
}
extern crate futures;
extern crate crossbeam_channel;
extern crate twox_hash;
extern crate dashmap;

pub mod task;
pub mod local_queue;
pub mod single_thread;
pub mod multi_thread;

use std::thread;
use std::pin::Pin;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Duration;
use std::future::Future;
use std::task::{Context, Poll};
use std::io::{Error, Result, ErrorKind};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

use futures::{future::BoxFuture, task::ArcWake, FutureExt};
use crossbeam_channel::{Sender, Receiver, unbounded};

use single_thread::SingleTaskRuntime;
use multi_thread::MultiTaskRuntime;

/*
* 异步任务
*/
pub trait AsyncTask: ArcWake {
    type Out;

    //获取内部任务
    fn get_inner(&self) -> Option<BoxFuture<'static, Self::Out>>;

    //设置内部任务
    fn set_inner(&self, inner: Option<BoxFuture<'static, Self::Out>>);
}

/*
* 异步任务派发器
*/
pub trait AsyncSpawner<T: AsyncTask<Out = O>, O> {
    //是否可以继续派发
    fn can_spawn(&self) -> bool;

    //派发一个异步任务
    fn spawn(&self, task: T) -> Result<()>;
}

/*
* 异步任务执行器
*/
pub trait AsyncExecutor {
    type Out;
    type Task: AsyncTask<Out = Self::Out>;
    type Spawner: AsyncSpawner<Self::Task, Self::Out>;

    //获取执行器的派发器
    fn get_spawner(&self) -> Self::Spawner;

    //运行一次执行器
    fn run_once(&mut self) -> AsyncExecutorResult;

    //持续运行执行器
    fn run(&mut self) -> Result<()> {
        loop {
            match self.run_once() {
                AsyncExecutorResult::Sleep(timeout) => {
                    thread::sleep(Duration::from_millis(timeout as u64));
                    continue;
                },
                AsyncExecutorResult::Stop(result) => {
                    return result;
                },
                AsyncExecutorResult::Ok => {
                    continue;
                },
            }
        }
    }
}

/*
* 异步执行返回值
*/
#[derive(Debug)]
pub enum AsyncExecutorResult {
    Sleep(usize),       //休眠指定毫秒数后，继续运行
    Stop(Result<()>),   //关闭当前执行器
    Ok,                 //执行成功
}

/*
* 异步任务唯一id
*/
#[derive(Debug, Clone, Copy)]
pub struct TaskId(usize);

/*
* 异步运行时
*/
pub enum AsyncRuntime<O: Default + 'static> {
    Single(SingleTaskRuntime<O>),   //单线程运行时
    Multi(MultiTaskRuntime<O>),     //多线程运行时
}

unsafe impl<O: Default + 'static> Send for AsyncRuntime<O> {}
unsafe impl<O: Default + 'static> Sync for AsyncRuntime<O> {}

impl<O: Default + 'static> Clone for AsyncRuntime<O> {
    fn clone(&self) -> Self {
        match self {
            AsyncRuntime::Single(rt) => AsyncRuntime::Single(rt.clone()),
            AsyncRuntime::Multi(rt) => AsyncRuntime::Multi(rt.clone()),
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
    evaluated:  Arc<AtomicBool>,            //是否已求值
}

unsafe impl<O: Default + 'static, V: Send + 'static> Send for AsyncValue<O, V> {}
unsafe impl<O: Default + 'static, V: Send + 'static> Sync for AsyncValue<O, V> {}

impl<O: Default + 'static, V: Send + 'static> Clone for AsyncValue<O, V> {
    fn clone(&self) -> Self {
        AsyncValue {
            rt: self.rt.clone(),
            task_id: self.task_id,
            value: self.value.clone(),
            evaluated: self.evaluated.clone(),
        }
    }
}

impl<O: Default + 'static, V: Send + 'static> Future for AsyncValue<O, V> {
    type Output = V;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.as_ref().value.borrow_mut().take() {
            //异步值已就绪
            return Poll::Ready(value);
        }

        match &self.as_ref().rt {
            AsyncRuntime::Single(rt) => {
                rt.pending(self.as_ref().task_id, cx.waker().clone())
            },
            AsyncRuntime::Multi(rt) => {
                rt.pending(self.as_ref().task_id, cx.waker().clone())
            },
        }
    }
}

impl<O: Default + 'static, V: Send + 'static> AsyncValue<O, V> {
    //构建异步值，默认值为未就绪
    pub fn new(rt: AsyncRuntime<O>) -> Self {
        let task_id = match &rt {
            AsyncRuntime::Single(rt) => {
                rt.alloc()
            },
            AsyncRuntime::Multi(rt) => {
                rt.alloc()
            },
        };

        AsyncValue {
            rt,
            task_id,
            value: Arc::new(RefCell::new(None)),
            evaluated: Arc::new(AtomicBool::new(false)),
        }
    }

    //设置异步值
    pub fn set(self, value: V) {
        if self.evaluated.compare_and_swap(false, true, Ordering::SeqCst) {
            //已求值，则忽略
            return;
        }

        //设置后立即释放可写引用，防止唤醒时出现冲突
        {
            *self.value.borrow_mut() = Some(value);
        }

        //唤醒异步值
        match &self.rt {
            AsyncRuntime::Single(rt) => {
                rt.wakeup(self.task_id)
            },
            AsyncRuntime::Multi(rt) => {
                rt.wakeup(self.task_id)
            },
        }
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
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //任务已完成，则返回
            return Poll::Ready(result);
        }

        //在指定运行时运行指定的任务
        let task_id = match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => wait_rt.alloc(),
            AsyncRuntime::Multi(wait_rt) => wait_rt.alloc(),
        };
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

                        match wait_copy {
                            AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                            AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                        }

                        //返回异步任务的默认值
                        Default::default()
                    }) {
                        //派发指定的任务失败，则立即唤醒等待的任务
                        *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Runner Error by Wait, reason: {:?}", e))));
                        match wait {
                            AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                            AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                        }
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

                        match wait_copy {
                            AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                            AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                        }

                        //返回异步任务的默认值
                        Default::default()
                    }) {
                        //派发指定的任务失败，则立即唤醒等待的任务
                        *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async Runner Error by Wait, reason: {:?}", e))));
                        match wait {
                            AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                            AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                        }
                    }
                },
            }

            //返回异步任务的默认值
            Default::default()
        };

        match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => {
                if let Err(e) = wait_rt.spawn(wait_rt.alloc(), task) {
                    //派发异步等待的任务失败，则立即返回错误原因
                    return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Error, reason: {:?}", e))));
                }
            },
            AsyncRuntime::Multi(wait_rt) => {
                if let Err(e) = wait_rt.spawn(wait_rt.alloc(), task) {
                    //派发异步等待的任务失败，则立即返回错误原因
                    return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Error, reason: {:?}", e))));
                }
            },
        }

        //挂起当前异步等待任务，并返回值未就绪
        match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => wait_rt.pending(task_id, cx.waker().clone()),
            AsyncRuntime::Multi(wait_rt) => wait_rt.pending(task_id, cx.waker().clone()),
        }
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
        if let Some(result) = self.as_ref().result.0.borrow_mut().take() {
            //任务已完成，则返回
            return Poll::Ready(result);
        }

        //在指定运行时运行指定的任务
        let task_id = match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => wait_rt.alloc(),
            AsyncRuntime::Multi(wait_rt) => wait_rt.alloc(),
        };
        let wait = self.as_ref().wait.clone();
        let mut futures = self.as_mut().futures.take().unwrap();
        let is_finish = self.as_ref().is_finish.clone();
        let result = self.as_ref().result.clone();
        let task = async move {
            while let Some((runner, future)) = futures.pop() {
                let wait_copy = wait.clone();
                let is_finish_copy = is_finish.clone();
                let result_copy = result.clone();
                match runner {
                    AsyncRuntime::Single(rt) => {
                        //将指定任务派发到单线程运行时
                        if let Err(e) = rt.spawn(rt.alloc(), async move {
                            if is_finish_copy.load(Ordering::Relaxed) {
                                //快速检查，当前已有任务执行完成，则忽略，并立即返回
                                return Default::default();
                            }

                            //执行任务，并检查是否由当前任务唤醒等待的任务
                            let r = future.await;
                            if !is_finish_copy.compare_and_swap(false, true, Ordering::SeqCst) {
                                //当前没有任务执行完成，则立即唤醒等待的任务
                                *result_copy.0.borrow_mut() = Some(r);
                                match wait_copy {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }

                            //返回异步任务的默认值
                            Default::default()
                        }) {
                            //派发指定的任务失败，则退出派发循环
                            if !is_finish.compare_and_swap(false, true, Ordering::SeqCst) {
                                //当前没有任务执行完成，则立即唤醒等待的任务
                                *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async SingleTask Runner Error, reason: {:?}", e))));
                                match wait {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }
                            break;
                        }
                    },
                    AsyncRuntime::Multi(rt) => {
                        //将指定任务派发到多线程运行时
                        if let Err(e) = rt.spawn(rt.alloc(), async move {
                            if is_finish_copy.load(Ordering::Relaxed) {
                                //快速检查，当前已有任务执行完成，则忽略，并立即返回
                                return Default::default();
                            }

                            //执行任务，并检查是否由当前任务唤醒等待的任务
                            let r = future.await;
                            if !is_finish_copy.compare_and_swap(false, true, Ordering::SeqCst) {
                                //当前没有任务执行完成，则立即唤醒等待的任务
                                *result_copy.0.borrow_mut() = Some(r);
                                match wait_copy {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }

                            //返回异步任务的默认值
                            Default::default()
                        }) {
                            //派发指定的任务失败，则退出派发循环
                            if !is_finish.compare_and_swap(false, true, Ordering::SeqCst) {
                                //当前没有任务执行完成，则立即唤醒等待的任务
                                *result.0.borrow_mut() = Some(Err(Error::new(ErrorKind::Other, format!("Async MultiTask Runner Error, reason: {:?}", e))));
                                match wait {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }
                            break;
                        }
                    },
                }
            }

            //返回异步任务的默认值
            Default::default()
        };

        match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => {
                if let Err(e) = wait_rt.spawn(wait_rt.alloc(), task) {
                    //派发异步等待的任务失败，则立即返回错误原因
                    return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Any Error, reason: {:?}", e))));
                }
            },
            AsyncRuntime::Multi(wait_rt) => {
                if let Err(e) = wait_rt.spawn(wait_rt.alloc(), task) {
                    //派发异步等待的任务失败，则立即返回错误原因
                    return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Wait Any Error, reason: {:?}", e))));
                }
            },
        }

        //挂起当前异步等待任务，并返回值未就绪
        match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => wait_rt.pending(task_id, cx.waker().clone()),
            AsyncRuntime::Multi(wait_rt) => wait_rt.pending(task_id, cx.waker().clone()),
        }
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
    futures:    Vec<(usize, AsyncRuntime<O>, BoxFuture<'static, V>)>,   //待派发任务
    producor:   Sender<(usize, Result<V>)>,                             //异步返回值生成器
    consumer:   Receiver<(usize, Result<V>)>,                           //异步返回值接收器
}

unsafe impl<O: Default + 'static, V: Send + 'static> Send for AsyncMap<O, V> {}

impl<O: Default + 'static, V: Send + 'static> AsyncMap<O, V> {
    pub fn join<F>(&mut self, rt: AsyncRuntime<O>, future: F)
        where F: Future<Output = V> + Send + 'static {
        let count = self.count;
        self.futures.push((count, rt, Box::new(future).boxed()));
        self.count += 1;
    }

    //映射所有任务，并返回指定异步运行时的异步归并
    pub fn map(self, wait: AsyncRuntime<O>, is_order: bool) -> AsyncReduce<O, V> {
        let count = Arc::new(AtomicUsize::new(self.count));
        let producor = self.producor.clone();
        let consumer = self.consumer.clone();

        AsyncReduce {
            futures: Some(self.futures),
            producor: Box::new(producor), //通过Box实现Pin
            consumer: Box::new(consumer), //通过Box实现Pin
            wait,
            is_order,
            count,
        }
    }
}

/*
* 异步归并，用于归并多个任务的返回值
*/
pub struct AsyncReduce<O: Default + 'static, V: Send + 'static> {
    futures:    Option<Vec<(usize, AsyncRuntime<O>, BoxFuture<'static, V>)>>,   //待派发任务
    producor:   Box<Sender<(usize, Result<V>)>>,                                //异步返回值生成器
    consumer:   Box<Receiver<(usize, Result<V>)>>,                              //异步返回值接收器
    wait:       AsyncRuntime<O>,                                                //等待的异步运行时
    is_order:   bool,                                                           //是否对返回的值排序
    count:      Arc<AtomicUsize>,                                               //需要归并的异步任务数量
}

unsafe impl<O: Default + 'static, V: Send + 'static> Send for AsyncReduce<O, V> {}
unsafe impl<O: Default + 'static, V: Send + 'static> Sync for AsyncReduce<O, V> {}

impl<O: Default + 'static, V: Send + 'static> Future for AsyncReduce<O, V> {
    type Output = Result<Vec<Result<V>>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.as_ref().count.load(Ordering::Relaxed) == 0 {
            //任务已完成，则返回
            let values = if self.as_ref().is_order {
                //需要对返回值排序
                let mut buf = self.as_ref().consumer.try_iter().collect::<Vec<(usize, Result<V>)>>();
                buf.sort_by(|(x, _), (y, _)| x.cmp(y));
                let (_, values) = buf.into_iter().unzip::<_, _, Vec<usize>, Vec<Result<V>>>();
                values
            } else {
                //不需要对返回值排序
                let buf = self.as_ref().consumer.try_iter().collect::<Vec<(usize, Result<V>)>>();
                let (_, values) = buf.into_iter().unzip::<_, _, Vec<usize>, Vec<Result<V>>>();
                values
            };

            return Poll::Ready(Ok(values));
        }

        //在归并任务所在运行时中派发所有异步任务，以保证归并任务在执行前不会被唤醒
        let task_id = match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => wait_rt.alloc(),
            AsyncRuntime::Multi(wait_rt) => wait_rt.alloc(),
        };
        let mut futures = self.as_mut().futures.take().unwrap();
        let producor = self.as_ref().producor.clone();
        let wait = self.as_ref().wait.clone();
        let count = self.as_ref().count.clone();
        let task = async move {
            while let Some((index, runtime, future)) = futures.pop() {
                let wait_copy = wait.clone();
                let count_copy = count.clone();
                let producor_copy = producor.clone();
                match runtime {
                    AsyncRuntime::Single(rt) => {
                        if let Err(e) = rt.spawn(rt.alloc(), async move {
                            let value = future.await;
                            producor_copy.send((index, Ok(value)));
                            if count_copy.fetch_sub(1, Ordering::SeqCst) == 1 {
                                //最后一个任务已执行完成，则立即唤醒等待的归并任务
                                match wait_copy {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }

                            //返回异步任务的默认值
                            Default::default()
                        }) {
                            //派发异步任务失败，则退出派发循环
                            producor.send((index, Err(e)));
                            if count.clone().fetch_sub(1, Ordering::SeqCst) == 1 {
                                //最后一个任务已执行完成，则立即唤醒等待的归并任务
                                match &wait {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }
                        }
                    },
                    AsyncRuntime::Multi(rt) => {
                        if let Err(e) = rt.spawn(rt.alloc(), async move {
                            let value = future.await;
                            producor_copy.send((index, Ok(value)));
                            if count_copy.fetch_sub(1, Ordering::SeqCst) == 1 {
                                //最后一个任务已执行完成，则立即唤醒等待的归并任务
                                match wait_copy {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }

                            //返回异步任务的默认值
                            Default::default()
                        }) {
                            //派发异步任务失败，则退出派发循环
                            producor.send((index, Err(e)));
                            if count.clone().fetch_sub(1, Ordering::SeqCst) == 1 {
                                //最后一个任务已执行完成，则立即唤醒等待的归并任务
                                match &wait {
                                    AsyncRuntime::Single(wait_rt) => wait_rt.wakeup(task_id),
                                    AsyncRuntime::Multi(wait_rt) => wait_rt.wakeup(task_id),
                                }
                            }
                        }
                    },
                }
            }

            //返回异步任务的默认值
            Default::default()
        };

        match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => {
                if let Err(e) = wait_rt.spawn(wait_rt.alloc(), task) {
                    //派发异步映射任务失败，则立即返回错误原因
                    return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Map Error, reason: {:?}", e))));
                }
            },
            AsyncRuntime::Multi(wait_rt) => {
                if let Err(e) = wait_rt.spawn(wait_rt.alloc(), task) {
                    //派发异步映射任务失败，则立即返回错误原因
                    return Poll::Ready(Err(Error::new(ErrorKind::Other, format!("Async Map Error, reason: {:?}", e))));
                }
            },
        }

        //挂起当前归并任务，并返回值未就绪
        match &self.as_ref().wait {
            AsyncRuntime::Single(wait_rt) => wait_rt.pending(task_id, cx.waker().clone()),
            AsyncRuntime::Multi(wait_rt) => wait_rt.pending(task_id, cx.waker().clone()),
        }
    }
}

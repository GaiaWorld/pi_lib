extern crate futures;
extern crate crossbeam_channel;
extern crate twox_hash;
extern crate dashmap;

pub mod task;
pub mod local_queue;
pub mod single_thread;
pub mod multi_thread;

use std::thread;
use std::io::Result;
use std::time::Duration;

use futures::{future::BoxFuture, task::ArcWake};

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
* 多线程任务唯一id
*/
#[derive(Debug, Clone, Copy)]
pub struct TaskId(usize);

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
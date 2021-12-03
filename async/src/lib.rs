//! # 基于Future(MVP)，用于为外部提供基础的通用异步运行时和工具
//!

#![allow(warnings)]
#![feature(panic_info_message)]

extern crate futures;
extern crate crossbeam_channel;
extern crate parking_lot;
extern crate log;
extern crate local_timer;

pub mod lock;
pub mod rt;
pub mod task;
pub mod local_queue;

use std::thread;
use std::io::Result;
use std::time::Duration;

use futures::{future::BoxFuture, task::ArcWake, FutureExt};

///
/// 异步任务
///
pub trait AsyncTask: ArcWake {
    type Out;

    /// 获取内部任务
    fn get_inner(&self) -> Option<BoxFuture<'static, Self::Out>>;

    /// 设置内部任务
    fn set_inner(&self, inner: Option<BoxFuture<'static, Self::Out>>);
}

///
/// 异步任务派发器
///
pub trait AsyncSpawner<T: AsyncTask<Out = O>, O> {
    /// 是否可以继续派发
    fn can_spawn(&self) -> bool;

    /// 派发一个异步任务
    fn spawn(&self, task: T) -> Result<()>;
}

///
/// 异步任务执行器
///
pub trait AsyncExecutor {
    type Out;
    type Task: AsyncTask<Out = Self::Out>;
    type Spawner: AsyncSpawner<Self::Task, Self::Out>;

    /// 获取执行器的派发器
    fn get_spawner(&self) -> Self::Spawner;

    /// 运行一次执行器
    fn run_once(&mut self) -> AsyncExecutorResult;

    /// 持续运行执行器
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

///
/// 异步执行返回值
///
#[derive(Debug)]
pub enum AsyncExecutorResult {
    Sleep(usize),       //休眠指定毫秒数后，继续运行
    Stop(Result<()>),   //关闭当前执行器
    Ok,                 //执行成功
}
use std::fmt::Debug;

/// 索引类型
#[derive(Debug, Clone)]
pub enum IndexType{
    Sync, //不可删除的同步任务
    Async,//不可删除的异步任务
    Delay, //延迟任务
    Queue, //任务的队列
    LockQueue, //被锁的任务队列
    HalfLockQueue, //被锁的任务队列, 一旦向其中push任务， 就会解锁
}

// //任务
// pub enum Task<T: 'static> {
//     Async(T),
//     Sync(T, isize),
// }

// //任务类型
// #[derive(Clone)]
// pub enum TaskType {
//     Async(usize),      //异步任务, Async(任务优先级, 能否删除)
//     Sync(isize, Direction),       //同步任务Sync(队列id, push方向)
// }

//同步任务push的方向
#[derive(Clone)]
pub enum Direction {
    Front,
    Back,
}

/// 任务
#[derive(Debug)]
pub enum Task<T: Debug> {
    Sync(T, isize),
    Async(T),
}

/// 释放标记
#[derive(Debug)]
pub enum FreeSign {
    Success,
    Error,
    Ignore,
}

/// 队列类型
#[derive(Debug)]
pub enum QueueType {
    DynSync,
    StaticSync,
    DynAsync,
    StaticAsync,
}
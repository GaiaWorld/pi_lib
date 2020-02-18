use std::sync::Arc;
use std::cell::RefCell;
use std::rc::{Weak, Rc};
use std::marker::PhantomData;
use std::task::{Context, Poll};
use std::collections::VecDeque;
use std::io::{Error, Result, ErrorKind};

use futures::{task::waker_ref};

use crate::{AsyncTask, AsyncSpawner, AsyncExecutor, AsyncExecutorResult};

/*
* 本地异步任务派发器
*/
pub struct LocalQueueSpawner<T: AsyncTask<Out = O>, O> {
    inner: Weak<RefCell<VecDeque<Arc<T>>>>,
    marker: PhantomData<O>,
}

unsafe impl<T: AsyncTask<Out = O>, O> Send for LocalQueueSpawner<T, O> {}
unsafe impl<T: AsyncTask<Out = O>, O> Sync for LocalQueueSpawner<T, O> {}

impl<T: AsyncTask<Out = O>, O> AsyncSpawner<T, O> for LocalQueueSpawner<T, O> {
    fn can_spawn(&self) -> bool {
        self.inner.upgrade().is_some()
    }

    fn spawn(&self, task: T) -> Result<()> {
        match self.inner.upgrade() {
            None => {
                Err(Error::new(ErrorKind::Interrupted, "conflict spawn local async task"))
            },
            Some(inner) => {
                inner.borrow_mut().push_back(Arc::new(task));
                Ok(())
            },
        }
    }
}

impl<T: AsyncTask<Out = O>, O> LocalQueueSpawner<T, O> {
    //唤醒指定的本地异步任务
    pub fn wakeup(&self, task: Arc<T>) -> Result<()> {
        match self.inner.upgrade() {
            None => {
                Err(Error::new(ErrorKind::Interrupted, "conflict wakeup local async task"))
            },
            Some(inner) => {
                inner.borrow_mut().push_back(task);
                Ok(())
            },
        }
    }
}

/*
* 本地异步任务队列
*/
pub struct LocalQueue<T: AsyncTask<Out = O>, O> {
    queue:  Rc<RefCell<VecDeque<Arc<T>>>>,
}

unsafe impl<T: AsyncTask<Out = O>, O> Send for LocalQueue<T, O> {}
unsafe impl<T: AsyncTask<Out = O>, O> Sync for LocalQueue<T, O> {}

impl<T: AsyncTask<Out = O>, O> AsyncExecutor for LocalQueue<T, O> {
    type Out = O;
    type Task = T;
    type Spawner = LocalQueueSpawner<T, O>;

    fn get_spawner(&self) -> Self::Spawner {
        LocalQueueSpawner {
            inner: Rc::downgrade(&self.queue),
            marker: PhantomData,
        }
    }

    fn run_once(&mut self) -> AsyncExecutorResult {
        match self.queue.try_borrow_mut() {
            Err(_e) => {
                AsyncExecutorResult::Ok
            },
            Ok(mut queue) => {
                for task in queue.pop_front() {
                    let waker = waker_ref(&task);
                    let mut context = Context::from_waker(&*waker);
                    if let Some(mut future) = task.get_inner() {
                        if let Poll::Pending = future.as_mut().poll(&mut context) {
                            //当前未准备好，则恢复异步任务，以保证异步任务不会被提前释放
                            task.set_inner(Some(future));
                        }
                    }
                }

                AsyncExecutorResult::Ok
            },
        }
    }
}

impl<T: AsyncTask<Out = O>, O> LocalQueue<T, O> {
    //构建一个初始容量的本地异步任务队列
    pub fn with_capacity(size: usize) -> Self {
        LocalQueue {
            queue: Rc::new(RefCell::new(VecDeque::with_capacity(size))),
        }
    }

    //获取当前本地异步任务队列的长度
    pub fn size(&self) -> usize {
        self.queue.borrow().len()
    }
}

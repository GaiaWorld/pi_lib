use std::rc::Rc;
use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;

use futures::{future::{FutureExt, BoxFuture}, task::ArcWake};

use crate::{AsyncTask, local_queue::LocalQueueSpawner};

/*
* 本地异步任务
*/
pub struct LocalTask<O> {
    future: UnsafeCell<Option<BoxFuture<'static, O>>>,
    spawner: Rc<LocalQueueSpawner<Self, O>>,
}

unsafe impl<O> Send for LocalTask<O> {}
unsafe impl<O> Sync for LocalTask<O> {}

impl<O> ArcWake for LocalTask<O> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = arc_self.clone();
        arc_self.spawner.wakeup(task).unwrap();
    }
}

impl<O> AsyncTask for LocalTask<O> {
    type Out = O;

    fn get_inner(&self) -> Option<BoxFuture<'static, Self::Out>> {
        unsafe { (*self.future.get()).take() }
    }

    fn set_inner(&self, inner: Option<BoxFuture<'static, Self::Out>>) {
        unsafe { *self.future.get() = inner; }
    }
}

impl<O> LocalTask<O> {
    //构建一个指定本地异步任务派发器和指定的Future的本地异步任务
    pub fn new(spawner: Rc<LocalQueueSpawner<Self, O>>, future: impl Future<Output = O> + Send + 'static) -> Self {
        LocalTask {
            future: UnsafeCell::new(Some(future.boxed())),
            spawner,
        }
    }
}
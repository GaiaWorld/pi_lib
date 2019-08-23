#![feature(async_await)]

extern crate futures;
extern crate crossbeam_channel;
extern crate r#async;

use std::thread;
use std::rc::{Weak, Rc};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::future::Future;
use std::cell::{UnsafeCell, RefCell};
use std::collections::HashMap;
use std::task::{Context, Poll, Waker};

use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};
use crossbeam_channel::Sender;

use r#async::{AsyncTask, AsyncExecutorResult, AsyncExecutor, AsyncSpawner,
              local_queue::{LocalQueueSpawner, LocalQueue}, task::LocalTask};

struct TestFuture(usize, Weak<RefCell<HashMap<usize, Waker>>>);

unsafe impl Send for TestFuture {}
unsafe impl Sync for TestFuture {}

impl Future for TestFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = self.as_ref().0;
        if index % 2 == 0 {
            match self.as_ref().1.upgrade() {
                None => {
                    println!("!!!> future poll failed, index: {}", index);
                },
                Some(handle) => {
                    self.as_mut().0 += 1;
                    handle.borrow_mut().insert(index, cx.waker().clone());
                },
            }
            Poll::Pending
        } else {
            Poll::Ready("future ready".to_string())
        }
    }
}

impl TestFuture {
    pub fn new(handle: Rc<RefCell<HashMap<usize, Waker>>>, index: usize) -> Self {
        TestFuture(index, Rc::downgrade(&handle))
    }
}

#[test]
fn test_async_task() {
    let handle = Rc::new(RefCell::new(HashMap::new()));
    let mut queue = LocalQueue::with_capacity(10);
    let spawner = Rc::new(queue.get_spawner());

    for index in 0..100 {
        let future = TestFuture::new(handle.clone(), index); //handle是Rc，不允许跨线程，需要在外部用TestFuture封装后再move到async block中，否则handle将无法move到async block中
        if let Err(e) = spawner.spawn(LocalTask::new(spawner.clone(), async move {
            println!("!!!!!!async task start, index: {}", index);
            let r = future.await;
            println!("!!!!!!async task finish, index: {}, r: {:?}", index, r);
        })) {
            println!("!!!> spawn task failed, index: {}, reason: {:?}", index, e);
        }

        queue.run_once();
    }

    let keys = &mut handle.borrow().keys().map(|key| {
        key.clone()
    }).collect::<Vec<usize>>()[..];
    keys.sort();
    keys.reverse();
    for key in keys {
        //唤醒中止任务
        if let Some(waker) = handle.borrow_mut().remove(key) {
            waker.wake();
        }

        queue.run_once();
    }
}
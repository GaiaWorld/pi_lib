#![feature(async_stream)]

extern crate futures;
extern crate crossbeam_channel;
extern crate twox_hash;
extern crate dashmap;
extern crate tokio;
extern crate r#async;

#[allow(unused_imports)]
#[macro_use]
extern crate env_logger;

use std::thread;
use std::rc::{Weak, Rc};
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::cell::{UnsafeCell, RefCell};
use std::task::{Context, Poll, Waker};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize, Ordering};

use futures::{pin_mut, stream::{Stream, StreamExt, BoxStream}, future::{FutureExt, BoxFuture},
              task::{SpawnExt, ArcWake, waker_ref},
              lock::Mutex as FuturesMutex, executor::LocalPool};
use parking_lot::{Mutex as ParkingLotMutex, Condvar};
use crossbeam_channel::{Sender, unbounded};
use twox_hash::RandomXxHashBuilder64;
use dashmap::DashMap;
use rand::prelude::*;
use future_parking_lot::{mutex::{Mutex as FutureMutex, FutureLockable}, rwlock::{RwLock as FutureRwLock, FutureReadable, FutureWriteable}};
use tokio::runtime::Builder as TokioRtBuilder;
use async_stream::stream;

use r#async::{AsyncExecutorResult, AsyncExecutor, AsyncSpawner,
                  lock::{mpmc_deque::MpmcDeque,
                         mpsc_deque::mpsc_deque,
                         spin_lock::SpinLock,
                         mutex_lock::Mutex,
                         rw_lock::RwLock},
                  rt::{TaskId, AsyncTask, AsyncRuntime, AsyncValue, spawn_worker_thread, AsyncPipelineResult, register_global_panic_handler,
                       single_thread::{SingleTaskRuntime, SingleTaskRunner},
                       multi_thread::{MultiTaskRuntime, MultiTaskRuntimeBuilder}},
                  local_queue::{LocalQueueSpawner, LocalQueue}, task::LocalTask};

#[test]
fn test_other_rt() {
    use std::mem;

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..10000000 {
            let counter_copy = counter.clone();
            let obj = Box::new(async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }).boxed();
            spawner.spawn(obj);
        }
        println!("!!!!!!spawn time: {:?}", Instant::now() - start);
    }
    pool.run();

    thread::sleep(Duration::from_millis(10000));

    let runtime = Arc::new(TokioRtBuilder::new_current_thread()
        .enable_time()
        .thread_stack_size(2 * 1024 * 1024)
        .build()
        .unwrap());
    let rt0 = runtime.clone();
    let rt1 = runtime.clone();
    let rt2 = runtime.clone();
    let rt3 = runtime.clone();

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let counter0 = counter.clone();
    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let counter3 = counter.clone();
    mem::drop(counter);

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2500000 {
                let counter_copy = counter0.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt0.block_on(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2500000 {
                let counter_copy = counter1.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt1.block_on(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2500000 {
                let counter_copy = counter2.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt2.block_on(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2500000 {
                let counter_copy = counter3.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt3.block_on(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::sleep(Duration::from_millis(10000));

    let runtime = Arc::new(TokioRtBuilder::new_multi_thread()
        .enable_all()
        .worker_threads(8)
        .thread_stack_size(2 * 1024 * 1024)
        .build()
        .unwrap());
    let rt0 = runtime.clone();
    let rt1 = runtime.clone();
    let rt2 = runtime.clone();
    let rt3 = runtime.clone();
    let rt4 = runtime.clone();
    let rt5 = runtime.clone();
    let rt6 = runtime.clone();
    let rt7 = runtime.clone();

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let counter0 = counter.clone();
    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let counter3 = counter.clone();
    let counter4 = counter.clone();
    let counter5 = counter.clone();
    let counter6 = counter.clone();
    let counter7 = counter.clone();
    mem::drop(counter);

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter0.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt0.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter1.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt1.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter2.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt2.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter3.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt3.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter4.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt4.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter5.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt5.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter6.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt6.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::spawn(move || {
        {
            let start = Instant::now();
            for _ in 0..2000000 {
                let counter_copy = counter7.clone();
                let obj = Box::new(async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }).boxed();
                rt7.spawn(obj);
            }
            println!("!!!!!!spawn time: {:?}", Instant::now() - start);
        }
    });

    thread::sleep(Duration::from_millis(1000000000));
}

#[test]
fn test_thread_local() {
    thread_local! {
        static TMP_THREAD_LOCAL: AtomicUsize = AtomicUsize::new(0);
    }

    let join1 = thread::spawn(move || {
        TMP_THREAD_LOCAL.try_with(move |local| {
            println!("!!!!!!local: {:?}", local.load(Ordering::Relaxed));
            local.store(1, Ordering::Relaxed);
        })
    });
    join1.join();

    let join2 = thread::spawn(move || {
        TMP_THREAD_LOCAL.try_with(move |local| {
            println!("!!!!!!local: {:?}", local.load(Ordering::Relaxed));
            local.store(2, Ordering::Relaxed);
        })
    });
    join2.join();

    let join3 = thread::spawn(move || {
        TMP_THREAD_LOCAL.try_with(move |local| {
            println!("!!!!!!local: {:?}", local.load(Ordering::Relaxed));
            local.store(3, Ordering::Relaxed);
        })
    });
    join3.join();

    let start = Instant::now();
    for index in 0..10000000 {
        if let Err(e) = TMP_THREAD_LOCAL.try_with(move |local| {
            local.fetch_add(1, Ordering::Relaxed);
        }) {
            println!("!!!!!!index: {:?}, e: {:?}", index, e);
            break;
        }
    }
    println!("!!!!!!time: {:?}", Instant::now() - start);
    TMP_THREAD_LOCAL.with(move |local| {
        println!("!!!!!!local: {:?}", local.load(Ordering::Relaxed));
    });
}

struct TestStream(usize, usize);

unsafe impl Send for TestStream {}

impl Stream for TestStream {
    type Item = usize;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.1 >= 0xffffffff {
            //数值过大，则停止生成新的Fibonacci数
            return Poll::Ready(None);
        }

        //生成新的Fibonacci数，并更新当前状态
        let value = self.0 + self.1;
        self.0 = self.1;
        self.1 = value;

        Poll::Ready(Some(value))
    }
}

#[test]
fn test_async_stream() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let rt_copy = rt.clone();
    rt.spawn(rt.alloc(), async move {
        let s = new_stream(3);

        pin_mut!(s);

        while let Some(value) = s.next().await {
            rt_copy.wait_timeout(1000).await;
            println!("got {}", value);
        }

        let mut s = new_boxed_stream(11);
        let mut input = Vec::new();
        let mut s = rt_copy.pipeline(s, move |n| {
            if n < 10 {
                input.push(n);
                AsyncPipelineResult::Filtered((n * 1000).to_string())
            } else {
                println!("input: {:?}", input);
                AsyncPipelineResult::Disconnect
            }
        });

        while let Some(value) = s.next().await {
            rt_copy.wait_timeout(1000).await;
            println!("got {:?}", value);
        }

        let mut s = TestStream(0, 1);
        while let Some(value) = s.next().await {
            println!("got {}", value);
        }
    });

    thread::sleep(Duration::from_millis(1000000000));
}

fn new_stream(len: usize) -> impl Stream<Item = usize> {
    stream! {
        for i in 0..len {
            yield i;
        }
    }
}

fn new_boxed_stream(len: usize) -> BoxStream<'static, usize> {
    let stream = stream! {
        for i in 0..len {
            yield i;
        }
    };

    stream.boxed()
}

#[test]
fn test_channel() {
    let (sender, receiver) = unbounded();
    let sender0 = sender.clone();
    let sender1 = sender.clone();
    let sender2 = sender.clone();
    let sender3 = sender.clone();
    let sender4 = sender.clone();
    let sender5 = sender.clone();
    let sender6 = sender.clone();
    let sender7 = sender.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender0.send(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender1.send(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender2.send(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender3.send(val);
        }
    });

    let join4 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender4.send(val);
        }
    });

    let join5 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender5.send(val);
        }
    });

    let join6 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender6.send(val);
        }
    });

    let join7 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender7.send(val);
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("!!!!!!len: {:?}, time: {:?}", receiver.len(), Instant::now() - start);

    let sender0 = sender.clone();
    let sender1 = sender.clone();
    let sender2 = sender.clone();
    let sender3 = sender.clone();
    let receiver0 = receiver.clone();
    let receiver1 = receiver.clone();
    let receiver2 = receiver.clone();
    let receiver3 = receiver.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender0.send(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender1.send(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender2.send(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender3.send(val);
        }
    });

    let join4 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver0.try_recv();
        }
    });

    let join5 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver1.try_recv();
        }
    });

    let join6 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver2.try_recv();
        }
    });

    let join7 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver3.try_recv();
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("!!!!!!len: {:?}, time: {:?}", receiver.len(), Instant::now() - start);
}

#[test]
fn test_mpmc_deque() {
    let queue = MpmcDeque::new();
    let sender0 = queue.clone();
    let sender1 = queue.clone();
    let sender2 = queue.clone();
    let sender3 = queue.clone();
    let sender4 = queue.clone();
    let sender5 = queue.clone();
    let sender6 = queue.clone();
    let sender7 = queue.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender0.push_back(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender1.push_back(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender2.push_back(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender3.push_back(val);
        }
    });

    let join4 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender4.push_back(val);
        }
    });

    let join5 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender5.push_back(val);
        }
    });

    let join6 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender6.push_back(val);
        }
    });

    let join7 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender7.push_back(val);
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("!!!!!!len: {:?}, time: {:?}", queue.tail_len(), Instant::now() - start);

    let sender0 = queue.clone();
    let sender1 = queue.clone();
    let sender2 = queue.clone();
    let sender3 = queue.clone();
    let receiver0 = queue.clone();
    let receiver1 = queue.clone();
    let receiver2 = queue.clone();
    let receiver3 = queue.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender0.push_back(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender1.push_back(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender2.push_back(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender3.push_back(val);
        }
    });

    let join4 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver0.pop();
        }
    });

    let join5 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver1.pop();
        }
    });

    let join6 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver2.pop();
        }
    });

    let join7 = thread::spawn(move || {
        for _ in 0..4000000 {
            receiver3.pop();
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("!!!!!!len: {:?}, time: {:?}", queue.tail_len() + queue.head_len(), Instant::now() - start);
}

#[test]
fn test_mpsc_deque() {
    let (sender, mut receiver) = mpsc_deque();
    let sender0 = sender.clone();
    let sender1 = sender.clone();
    let sender2 = sender.clone();
    let sender3 = sender.clone();
    let sender4 = sender.clone();
    let sender5 = sender.clone();
    let sender6 = sender.clone();
    let sender7 = sender.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender0.send(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender1.send(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender2.send(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender3.send(val);
        }
    });

    let join4 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender4.send(val);
        }
    });

    let join5 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender5.send(val);
        }
    });

    let join6 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender6.send(val);
        }
    });

    let join7 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender7.send(val);
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("!!!!!!len: {:?}, time: {:?}", receiver.len(), Instant::now() - start);

    let sender0 = sender.clone();
    let sender1 = sender.clone();
    let sender2 = sender.clone();
    let sender3 = sender.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender0.send(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender1.send(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender2.send(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender3.send(val);
        }
    });

    let join4 = thread::spawn(move || {
        for _ in 0..16000000 {
            receiver.try_recv();
        }
        println!("!!!!!!len: {:?}, time: {:?}", receiver.len(), Instant::now() - start);
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
}

#[test]
fn test_steal_deque() {
    let (sender, mut receiver) = mpsc_deque();
    let sender0 = sender.clone();
    let sender1 = sender.clone();
    let sender2 = sender.clone();
    let sender3 = sender.clone();
    let sender4 = sender.clone();
    let sender5 = sender.clone();
    let sender6 = sender.clone();
    let sender7 = sender.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender0.send(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender1.send(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender2.send(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender3.send(val);
        }
    });

    let join4 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender4.send(val);
        }
    });

    let join5 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender5.send(val);
        }
    });

    let join6 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender6.send(val);
        }
    });

    let join7 = thread::spawn(move || {
        for index in 0..1000000 {
            let val = Arc::new((index, index, index));
            sender7.send(val);
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();
    join4.join();
    join5.join();
    join6.join();
    join7.join();
    println!("!!!!!!len: {:?}, time: {:?}", receiver.len(), Instant::now() - start);

    let sender0 = sender.clone();
    let sender1 = sender.clone();
    let sender2 = sender.clone();
    let sender3 = sender.clone();
    let start = Instant::now();

    let join0 = thread::spawn(move || {
        for index in 0..8000000 {
            let val = Arc::new((index, index, index));
            sender0.send(val);
        }
    });

    let join1 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender1.send(val);
        }
    });

    let join2 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender2.send(val);
        }
    });

    let join3 = thread::spawn(move || {
        for index in 0..2000000 {
            let val = Arc::new((index, index, index));
            sender3.send(val);
        }
    });

    join0.join();
    join1.join();
    join2.join();
    join3.join();

    let join4 = thread::spawn(move || {
        while let Some(_) = receiver.try_recv() {}
        println!("!!!!!!len: {:?}, time: {:?}", receiver.len(), Instant::now() - start);
    });

    join4.join();
}

struct TestAsyncTask {
    uid:    usize,
    future: UnsafeCell<Option<BoxFuture<'static, ()>>>,
    queue:  Sender<Arc<TestAsyncTask>>,
}

unsafe impl Send for TestAsyncTask {}
unsafe impl Sync for TestAsyncTask {}

impl ArcWake for TestAsyncTask {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.queue.send(arc_self.clone());
    }
}

#[test]
fn test_waker() {
    let start = Instant::now();
    let (send, recv) = unbounded();
    let mut vec = Vec::with_capacity(10000000);
    for uid in 0..10000000 {
        let future = Box::new(async move {

        }).boxed();

        vec.push(Arc::new(TestAsyncTask {
            uid,
            future: UnsafeCell::new(Some(future)),
            queue: send.clone(),
        }));
    }
    println!("!!!!!!init task ok, time: {:?}", Instant::now() - start);

    let start = Instant::now();
    for index in 0..10000000 {
        let waker = waker_ref(&vec[index]);
    }
    println!("!!!!!!init waker ok, time: {:?}", Instant::now() - start);
}

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

#[test]
fn test_dashmap() {
    let map: Arc<DashMap<usize, usize, RandomXxHashBuilder64>> = Arc::new(DashMap::with_hasher(Default::default()));

    let map0 = map.clone();
    let handle0 = thread::spawn(move || {
        let start = Instant::now();
        for key in 0..10000000 {
            map0.insert(key, key);
        }
        println!("!!!!!!handle0, insert time: {:?}", Instant::now() - start);
    });

    let map1 = map.clone();
    let handle1 = thread::spawn(move || {
        let start = Instant::now();
        for key in 10000000..20000000 {
            map1.insert(key, key);
        }
        println!("!!!!!!handle1, insert time: {:?}", Instant::now() - start);
    });

    let map2 = map.clone();
    let handle2 = thread::spawn(move || {
        let start = Instant::now();
        for key in 20000000..30000000 {
            map2.insert(key, key);
        }
        println!("!!!!!!handle0, insert time: {:?}", Instant::now() - start);
    });

    let map3 = map.clone();
    let handle3 = thread::spawn(move || {
        let start = Instant::now();
        for key in 0..30000000 {
            map3.get(&key);
        }
        println!("!!!!!!handle3, get time: {:?}", Instant::now() - start);
    });

    let map4 = map.clone();
    let handle4 = thread::spawn(move || {
        let start = Instant::now();
        for key in 0..30000000 {
            map4.get(&key);
        }
        println!("!!!!!!handle3, get time: {:?}", Instant::now() - start);
    });

    let map5 = map.clone();
    let handle5 = thread::spawn(move || {
        let start = Instant::now();
        for key in 0..30000000 {
            map5.get(&key);
        }
        println!("!!!!!!handle3, get time: {:?}", Instant::now() - start);
    });

    handle0.join();
    handle1.join();
    handle2.join();
    handle3.join();
    handle4.join();
    handle5.join();

    println!("!!!!!!map len: {:?}", map.len());
    let mut total = 0;
    let start = Instant::now();
    for key in 0..map.len() {
        map.get(&key);
        total += key;
    }
    println!("!!!!!!finish, total: {:?}, get time: {:?}", total, Instant::now() - start);
}

struct Counter(i32, Instant);
impl Drop for Counter {
    fn drop(&mut self) {
        println!("!!!!!!drop counter, count: {:?}, time: {:?}", self.0, Instant::now() - self.1);
    }
}

#[test]
fn test_atomic() {
    let atomic = AtomicBool::new(false);
    let start = Instant::now();
    for _ in 0..100000000 {
        atomic.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
    }
    println!("!!!!!!atomic bool time: {:?}", Instant::now() - start);

    let atomic = AtomicU8::new(0);
    let start = Instant::now();
    for _ in 0..100000000 {
        atomic.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(1, 2, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(2, 3, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(3, 4, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(4, 5, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(5, 6, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(6, 7, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(7, 8, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(8, 9, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(9, 0, Ordering::Acquire, Ordering::Relaxed);
    }
    println!("!!!!!!atomic u8 time: {:?}", Instant::now() - start);

    let atomic = AtomicU16::new(0);
    let start = Instant::now();
    for _ in 0..100000000 {
        atomic.compare_exchange(0, 1000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(1000, 2000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(2000, 3000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(3000, 4000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(4000, 5000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(5000, 6000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(6000, 7000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(7000, 8000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(8000, 9000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(9000, 0, Ordering::Acquire, Ordering::Relaxed);
    }
    println!("!!!!!!atomic u16 time: {:?}", Instant::now() - start);

    let atomic = AtomicU32::new(0);
    let start = Instant::now();
    for _ in 0..100000000 {
        atomic.compare_exchange(0, 100000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(100000, 200000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(200000, 300000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(300000, 400000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(400000, 500000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(500000, 600000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(600000, 700000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(700000, 800000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(800000, 900000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(900000, 0, Ordering::Acquire, Ordering::Relaxed);
    }
    println!("!!!!!!atomic u32 time: {:?}", Instant::now() - start);

    let atomic = AtomicU64::new(0);
    let start = Instant::now();
    for _ in 0..100000000 {
        atomic.compare_exchange(0, 10000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(10000000000, 20000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(20000000000, 30000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(30000000000, 40000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(40000000000, 50000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(50000000000, 60000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(60000000000, 70000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(70000000000, 80000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(80000000000, 90000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(90000000000, 0, Ordering::Acquire, Ordering::Relaxed);
    }
    println!("!!!!!!atomic u64 time: {:?}", Instant::now() - start);

    let atomic = AtomicUsize::new(0);
    let start = Instant::now();
    for _ in 0..100000000 {
        atomic.compare_exchange(0, 10000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(10000000000, 20000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(20000000000, 30000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(30000000000, 40000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(40000000000, 50000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(50000000000, 60000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(60000000000, 70000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(70000000000, 80000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(80000000000, 90000000000, Ordering::Acquire, Ordering::Relaxed);
        atomic.compare_exchange(90000000000, 0, Ordering::Acquire, Ordering::Relaxed);
    }
    println!("!!!!!!atomic usize time: {:?}", Instant::now() - start);
}

//future_parking_lot的Mutex无法在临界区内执行异步任务等待
#[test]
fn test_future_mutex() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let start = Instant::now();
    let shared = Arc::new(FutureMutex::new(Counter(0, start)));

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..1000000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                {
                    let mut v = shared_.future_lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Local(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 1000000..2000000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                {
                    let mut v = shared0_.future_lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 2000000..3000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let mut v = shared1_.future_lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_future_rwlock() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    let start = Instant::now();
    let shared = Arc::new(FutureRwLock::new(Counter(0, start)));

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..1500000 {
            let shared_ = shared_copy.clone();
            {
                let mut v = shared_.write();
                (*v).0 += 1;
            }

            let v = shared_.read();
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 1500000..3000000 {
            let shared_ = shared_copy.clone();
            rt.spawn(rt.alloc(), async move {
                {
                    let mut v = shared_.future_write().await;
                    (*v).0 += 1;
                }

                let v = shared_.future_read().await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..1500000 {
            let shared0_ = shared_copy.clone();
            rt0.spawn(rt0.alloc(), async move {
                let v = shared0_.future_read().await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 1500000..3000000 {
            let shared1_ = shared.clone();
            rt1.spawn(rt1.alloc(), async move {
                let v = shared1_.future_read().await;
            });
        }
    });

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_futures_mutex() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let start = Instant::now();
    let shared = Arc::new(FuturesMutex::new(Counter(0, start)));

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                {
                    let mut v = shared_.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Local(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000..20000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                {
                    let mut v = shared0_.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000..30000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let mut v = shared1_.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });
    thread::sleep(Duration::from_micros(5000));

    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let start = Instant::now();
    let shared = Arc::new(FuturesMutex::new(Counter(0, start)));
    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Local(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000..20000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000..30000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_spin_lock() {
    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(2)
        .set_worker_limit(2, 2);
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(2)
        .set_worker_limit(2, 2);
    let rt1 = pool.build();

    {
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();
        let rt1_0 = rt1.clone();
        let rt1_1 = rt1.clone();
        let rt1_2 = rt1.clone();
        let rt1_3 = rt1.clone();

        let start = Instant::now();
        let shared = Arc::new(SpinLock::new(Counter(0, start)));
        let shared0 = shared.clone();
        let shared1 = shared.clone();
        let shared2 = shared.clone();
        let shared3 = shared.clone();

        thread::spawn(move || {
            for _ in 0..2500000 {
                let rt0_copy = rt0_0.clone();
                let shared = shared0.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_0.clone();
                let shared = shared0.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let rt0_copy = rt0_1.clone();
                let shared = shared1.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_1.clone();
                let shared = shared1.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let rt0_copy = rt0_2.clone();
                let shared = shared2.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_2.clone();
                let shared = shared2.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let rt0_copy = rt0_3.clone();
                let shared = shared3.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_3.clone();
                let shared = shared3.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock();
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }
    thread::sleep(Duration::from_millis(20000));

    //锁不跨临界区传递，不会产生deadlock
    {
        let start = Instant::now();
        let shared = Arc::new(SpinLock::new(Counter(0, start)));
        for _ in 0..10000000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                {
                    let mut v = shared0.lock();
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared0_copy = shared0.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.lock();
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                {
                    let mut v = shared1.lock();
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared1_copy = shared1.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared1_copy.lock();
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
        }
    }
    thread::sleep(Duration::from_millis(50000));

    //锁跨临界区传递，且不需要等待此跨临界区的锁，不会产生deadlock
    {
        let start = Instant::now();
        let shared = Arc::new(SpinLock::new(Counter(0, start)));
        for _ in 0..10000000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                let mut v = shared0.lock();
                (*v).0 += 1;

                let shared0_copy = shared0.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.lock();
                    (*v).0 += 1;
                });
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                let mut v = shared1.lock();
                (*v).0 += 1;

                let shared1_copy = shared1.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared1_copy.lock();
                    (*v).0 += 1;
                });
            });
        }
    }
    thread::sleep(Duration::from_millis(50000));
    println!("!!!!!!valid test finish");

    //锁不跨临界区传递，但临界区内需要执行异步任务等待，会产生deadlock
    {
        let start = Instant::now();
        let shared = Arc::new(SpinLock::new(Counter(0, start)));
        for _ in 0..10000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                let mut v = shared0.lock();
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                let mut v = shared1.lock();
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    }

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_spin_lock_bench() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    println!("!!!!!!Start lock test for single thread");
    let start = Instant::now();
    let shared = Arc::new(SpinLock::new(Counter(0, start)));
    let rt_ = rt.clone();

    thread::spawn(move || {
        for _ in 0..30000000 {
            let shared_ = shared.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock();
                (*v).0 += 1;
            });
        }
    });

    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!Finish lock test for single thread, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start lock test for multi thread");
    let start = Instant::now();
    let shared = Arc::new(SpinLock::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000000 {
            let shared_ = shared_copy.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock();
                (*v).0 += 1;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000000..20000000 {
            let shared0_ = shared_copy.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock();
                (*v).0 += 1;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000000..30000000 {
            let shared1_ = shared.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock();
                (*v).0 += 1;
            });
        }
    });

    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!Finish lock test for multi thread, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_mutex_lock() {
    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(2)
        .set_worker_limit(2, 2);
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(2)
        .set_worker_limit(2, 2);
    let rt1 = pool.build();

    {
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();
        let rt1_0 = rt1.clone();
        let rt1_1 = rt1.clone();
        let rt1_2 = rt1.clone();
        let rt1_3 = rt1.clone();

        let start = Instant::now();
        let shared = Arc::new(Mutex::new(Counter(0, start)));
        let shared0 = shared.clone();
        let shared1 = shared.clone();
        let shared2 = shared.clone();
        let shared3 = shared.clone();

        thread::spawn(move || {
            for _ in 0..2500000 {
                let rt0_copy = rt0_0.clone();
                let shared = shared0.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_0.clone();
                let shared = shared0.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            for _ in 2500000..5000000 {
                let rt0_copy = rt0_1.clone();
                let shared = shared1.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_1.clone();
                let shared = shared1.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            for _ in 5000000..7500000 {
                let rt0_copy = rt0_2.clone();
                let shared = shared2.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_2.clone();
                let shared = shared2.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            for _ in 7500000..10000000 {
                let rt0_copy = rt0_3.clone();
                let shared = shared3.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });

                let rt1_copy = rt1_3.clone();
                let shared = shared3.clone();
                rt1_copy.spawn(rt1_copy.alloc(), async move {
                    let mut v = shared.lock().await;
                    (*v).0 += 1;
                });
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }
    thread::sleep(Duration::from_millis(30000));

    {
        let start = Instant::now();
        let shared = Arc::new(Mutex::new(Counter(0, start)));
        for _ in 0..10000000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                {
                    let mut v = shared0.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                {
                    let mut v = shared1.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    }
    thread::sleep(Duration::from_millis(50000));

    {
        let start = Instant::now();
        let shared = Arc::new(Mutex::new(Counter(0, start)));
        for _ in 0..1000000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                let mut v = shared0.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                let mut v = shared1.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    }
    thread::sleep(Duration::from_millis(30000));

    //锁不跨临界区传递，不会产生deadlock
    {
        let start = Instant::now();
        let shared = Arc::new(Mutex::new(Counter(0, start)));
        for _ in 0..1000000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                {
                    let mut v = shared0.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared0_copy = shared0.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.lock().await;
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                {
                    let mut v = shared1.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared1_copy = shared1.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared1_copy.lock().await;
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
        }
    }
    thread::sleep(Duration::from_millis(10000));

    //锁跨临界区传递，且不需要等待此跨临界区的锁，不会产生deadlock
    {
        let start = Instant::now();
        let shared = Arc::new(Mutex::new(Counter(0, start)));
        for _ in 0..1000000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                let mut v = shared0.lock().await;
                (*v).0 += 1;

                let shared0_copy = shared0.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.lock().await;
                    (*v).0 += 1;
                });
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                let mut v = shared1.lock().await;
                (*v).0 += 1;

                let shared1_copy = shared1.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared1_copy.lock().await;
                    (*v).0 += 1;
                });
            });
        }
    }
    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!valid test finish");

    //锁跨临界区传递，且需要等待此跨临界区的锁，会产生deadlock
    {
        let start = Instant::now();
        let shared = Arc::new(Mutex::new(Counter(0, start)));
        for _ in 0..10000 {
            let shared0 = shared.clone();
            let rt_copy = rt0.clone();
            rt0.spawn(rt0.alloc(), async move {
                let mut v = shared0.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared0_copy = shared0.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.lock().await;
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
            let shared1 = shared.clone();
            let rt_copy = rt1.clone();
            rt1.spawn(rt1.alloc(), async move {
                let mut v = shared1.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared1_copy = shared1.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared1_copy.lock().await;
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
        }
    }

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_mutex_lock_bench() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    println!("!!!!!!Start lock test for single thread");
    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();

    thread::spawn(move || {
        for _ in 0..10000000 {
            let shared_ = shared.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;
            });
        }
    });

    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!Finish lock test for single thread, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start lock test for multi thread");
    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000000 {
            let shared_ = shared_copy.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000000..20000000 {
            let shared0_ = shared_copy.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock().await;
                (*v).0 += 1;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000000..30000000 {
            let shared1_ = shared.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock().await;
                (*v).0 += 1;
            });
        }
    });

    thread::sleep(Duration::from_millis(15000));
    println!("!!!!!!Finish lock test for multi thread, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start small scope lock test for AsyncValue");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                {
                    let mut v = shared_.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Local(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000000..20000000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                {
                    let mut v = shared0_.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000000..30000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let mut v = shared1_.lock().await;
                    (*v).0 += 1;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::sleep(Duration::from_millis(60000));
    println!("!!!!!!Finish small scope lock test for AsyncValue, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start full scope lock test for AsyncValue");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Local(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000..20000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000..30000 {
            let rt_copy = rt1_.clone();
            let shared1_ = shared.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock().await;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                (*v).0 += 1;
            });
        }
    });

    thread::sleep(Duration::from_millis(30000));
    println!("!!!!!!Finish full scope lock test for AsyncValue, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start small scope lock test for wait");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..1000000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                {
                    let mut v = shared_.lock().await;
                    (*v).0 += 1;
                }

                rt_copy.wait(AsyncRuntime::Local(rt_copy.clone()), async move {
                    Ok(true)
                }).await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 1000000..2000000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                {
                    let mut v = shared0_.lock().await;
                    (*v).0 += 1;
                }

                rt_copy.wait(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                }).await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 2000000..3000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let mut v = shared1_.lock().await;
                    (*v).0 += 1;
                }

                rt_copy.wait(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                }).await;
            });
        }
    });

    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!Finish small scope lock test for wait, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start full scope lock test for wait");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;

                rt_copy.wait(AsyncRuntime::Local(rt_copy.clone()), async move {
                    Ok(true)
                }).await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000..20000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock().await;
                (*v).0 += 1;

                rt_copy.wait(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                }).await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000..30000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock().await;
                (*v).0 += 1;

                rt_copy.wait(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                }).await;
            });
        }
    });

    thread::sleep(Duration::from_millis(30000));
    println!("!!!!!!Finish full scope lock test for wait, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start small scope lock test for wait any");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..1000000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                {
                    let mut v = shared_.lock().await;
                    (*v).0 += 1;
                }

                rt_copy.wait_any(vec![(AsyncRuntime::Local(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed()), (AsyncRuntime::Local(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed())]).await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 1000000..2000000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                {
                    let mut v = shared0_.lock().await;
                    (*v).0 += 1;
                }

                rt_copy.wait_any(vec![(AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed()), (AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed())]).await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 2000000..3000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let mut v = shared1_.lock().await;
                    (*v).0 += 1;
                }

                rt_copy.wait_any(vec![(AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed()), (AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed())]).await;
            });
        }
    });

    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!Finish small scope lock test for wait any, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start full scope lock test for wait any");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;

                rt_copy.wait_any(vec![(AsyncRuntime::Local(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed()), (AsyncRuntime::Local(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed())]).await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000..20000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock().await;
                (*v).0 += 1;

                rt_copy.wait_any(vec![(AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed()), (AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed())]).await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000..30000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock().await;
                (*v).0 += 1;

                rt_copy.wait_any(vec![(AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed()), (AsyncRuntime::Multi(rt_copy.clone()), Box::new(async move {
                    Ok(true)
                }).boxed())]).await;
            });
        }
    });

    thread::sleep(Duration::from_millis(30000));
    println!("!!!!!!Finish full scope lock test for wait any, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start small scope lock test for wait all");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..1000000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                {
                    let mut v = shared_.lock().await;
                    (*v).0 += 1;
                }

                let mut map_reduce = rt_copy.map_reduce(2);
                map_reduce.map(AsyncRuntime::Local(rt_copy.clone()), async move {
                    Ok(true)
                });
                map_reduce.map(AsyncRuntime::Local(rt_copy.clone()), async move {
                    Ok(true)
                });
                let _ = map_reduce.reduce(true).await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 1000000..2000000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                {
                    let mut v = shared0_.lock().await;
                    (*v).0 += 1;
                }

                let mut map_reduce = rt_copy.map_reduce(2);
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                let _ = map_reduce.reduce(true).await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 2000000..3000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let mut v = shared1_.lock().await;
                    (*v).0 += 1;
                }

                let mut map_reduce = rt_copy.map_reduce(2);
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                let _ = map_reduce.reduce(true).await;
            });
        }
    });

    thread::sleep(Duration::from_millis(20000));
    println!("!!!!!!Finish small scope lock test for wait all, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));
    println!("!!!!!!Start full scope lock test for wait all");

    let start = Instant::now();
    let shared = Arc::new(Mutex::new(Counter(0, start)));
    let rt_ = rt.clone();
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 0..10000 {
            let shared_ = shared_copy.clone();
            let rt_copy = rt_.clone();
            rt_.spawn(rt_.alloc(), async move {
                let mut v = shared_.lock().await;
                (*v).0 += 1;

                let mut map_reduce = rt_copy.map_reduce(2);
                map_reduce.map(AsyncRuntime::Local(rt_copy.clone()), async move {
                    Ok(true)
                });
                map_reduce.map(AsyncRuntime::Local(rt_copy.clone()), async move {
                    Ok(true)
                });
                let _ = map_reduce.reduce(true).await;
            });
        }
    });

    let shared_copy = shared.clone();
    thread::spawn(move || {
        for _ in 10000..20000 {
            let shared0_ = shared_copy.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.lock().await;
                (*v).0 += 1;

                let mut map_reduce = rt_copy.map_reduce(2);
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                let _ = map_reduce.reduce(true).await;
            });
        }
    });

    thread::spawn(move || {
        for _ in 20000..30000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let mut v = shared1_.lock().await;
                (*v).0 += 1;

                let mut map_reduce = rt_copy.map_reduce(2);
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                map_reduce.map(AsyncRuntime::Multi(rt_copy.clone()), async move {
                    Ok(true)
                });
                let _ = map_reduce.reduce(true).await;
            });
        }
    });

    thread::sleep(Duration::from_millis(30000));
    println!("!!!!!!Finish full scope lock test for wait all, task: {:?}", (rt.alloc(), rt0.alloc(), rt1.alloc()));

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_rw_lock() {
    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(2)
        .set_worker_limit(2, 2);
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(2)
        .set_worker_limit(2, 2);
    let rt1 = pool.build();

    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let start = Instant::now();
    let shared0 = Arc::new(RwLock::new(Counter(0, start)));
    let shared1 = Arc::new(RwLock::new(Counter(0, start)));
    thread::spawn(move || {
        for _ in 0..2000000 {
            let shared0_ = shared0.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.write().await;
                (*v).0 += 1;
            });
        }
    });
    thread::spawn(move || {
        for _ in 0..8000000 {
            let shared1_ = shared1.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let v = shared1_.read().await;
                (*v).0;
            });
        }
    });
    thread::sleep(Duration::from_millis(10000));

    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let start = Instant::now();
    let shared = Arc::new(RwLock::new(Counter(0, start)));
    let shared0 = shared.clone();
    thread::spawn(move || {
        for _ in 0..2000000 {
            let shared0_ = shared0.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.write().await;
                (*v).0 += 1;
            });
        }
    });
    thread::spawn(move || {
        for _ in 0..8000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let v = shared1_.read().await;
                    (*v).0;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });
    thread::sleep(Duration::from_millis(30000));

    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let start = Instant::now();
    let shared = Arc::new(RwLock::new(Counter(0, start)));
    let shared0 = shared.clone();
    thread::spawn(move || {
        for _ in 0..200000 {
            let shared0_ = shared0.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.write().await;
                (*v).0 += 1;
            });
        }
    });
    thread::spawn(move || {
        for _ in 0..800000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let v = shared1_.read().await;
                (*v).0;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });
    thread::sleep(Duration::from_millis(10000));

    //锁不跨临界区传递，不会产生deadlock
    let start = Instant::now();
    let shared = Arc::new(RwLock::new(Counter(0, start)));
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let shared0 = shared.clone();
    thread::spawn(move || {
        for _ in 0..2000000 {
            let shared0_ = shared0.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.write().await;
                (*v).0 += 1;
            });
        }
    });
    thread::spawn(move || {
        for _ in 0..8000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                {
                    let v = shared1_.read().await;
                    (*v).0;
                }

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared1_copy = shared1_.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let v = shared1_copy.read().await;
                    (*v).0;
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });
    thread::sleep(Duration::from_millis(30000));

    //锁跨临界区传递，且不需要等待此跨临界区的锁，不会产生deadlock
    let start = Instant::now();
    let shared = Arc::new(RwLock::new(Counter(0, start)));
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let shared0 = shared.clone();
    thread::spawn(move || {
        for _ in 0..2000000 {
            let shared0_ = shared0.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.write().await;
                (*v).0 += 1;

                let shared0_copy = shared0_.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.write().await;
                    (*v).0 += 1;
                });
            });
        }
    });
    thread::spawn(move || {
        for _ in 0..8000000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let v = shared1_.read().await;
                (*v).0;

                let shared1_copy = shared1_.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let v = shared1_copy.read().await;
                    (*v).0;
                });
            });
        }
    });
    thread::sleep(Duration::from_millis(30000));
    println!("!!!!!!valid test finish");

    //锁跨临界区传递，且需要等待此跨临界区的锁，会产生deadlock
    let start = Instant::now();
    let shared = Arc::new(RwLock::new(Counter(0, start)));
    let rt0_ = rt0.clone();
    let rt1_ = rt1.clone();
    let shared0 = shared.clone();
    thread::spawn(move || {
        for _ in 0..2000 {
            let shared0_ = shared0.clone();
            let rt_copy = rt0_.clone();
            rt0_.spawn(rt0_.alloc(), async move {
                let mut v = shared0_.write().await;
                (*v).0 += 1;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared0_copy = shared0_.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let mut v = shared0_copy.write().await;
                    (*v).0 += 1;
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });
    thread::spawn(move || {
        for _ in 0..8000 {
            let shared1_ = shared.clone();
            let rt_copy = rt1_.clone();
            rt1_.spawn(rt1_.alloc(), async move {
                let v = shared1_.read().await;
                (*v).0;

                let value = AsyncValue::new(AsyncRuntime::Multi(rt_copy.clone()));
                let value_copy = value.clone();
                let shared1_copy = shared1_.clone();
                rt_copy.spawn(rt_copy.alloc(), async move {
                    let v = shared1_copy.read().await;
                    (*v).0;
                    value_copy.set(true);
                });
                value.await;
            });
        }
    });
    thread::sleep(Duration::from_millis(5000));

    thread::sleep(Duration::from_millis(100000000));
}

#[derive(Clone)]
struct SyncUsize(Arc<RefCell<usize>>);

unsafe impl Send for SyncUsize {}
unsafe impl Sync for SyncUsize {}

struct TestFuture0(SyncUsize, TaskId, SingleTaskRuntime<()>);

unsafe impl Send for TestFuture0 {}
unsafe impl Sync for TestFuture0 {}

impl Future for TestFuture0 {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = *(self.as_ref().0).0.borrow();
        if index % 2 == 0 {
            self.2.pending(&self.1, cx.waker().clone())
        } else {
            Poll::Ready("future ready".to_string())
        }
    }
}

impl TestFuture0 {
    pub fn new(rt: SingleTaskRuntime<()>, index: SyncUsize, uid: TaskId) -> Self {
        TestFuture0(index, uid, rt)
    }
}

#[test]
fn test_single_task() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let mut ids = Vec::with_capacity(50);
    for index in 0..100 {
        let uid = rt.alloc();
        let uid_copy = uid.clone();
        let value = SyncUsize(Arc::new(RefCell::new(index)));
        let future = TestFuture0::new(rt.clone(), value.clone(), uid.clone());
        if let Err(e) = rt.spawn(uid.clone(), async move {
            println!("!!!!!!async task start, uid: {:?}", uid_copy);
            let r = future.await;
            println!("!!!!!!async task finish, uid: {:?}, r: {:?}", uid_copy, r);
        }) {
            println!("!!!> spawn task failed, uid: {:?}, reason: {:?}", uid, e);
        }

        if index % 2 == 0 {
            ids.push((uid, value));
        }
    }

    thread::sleep(Duration::from_millis(3000));

    for (id, value) in ids {
        let id_copy = id.clone();
        let uid = rt.alloc();
        let uid_copy = uid.clone();
        let rt_copy = rt.clone();
        if let Err(e) = rt.spawn(uid, async move {
            //修改值，并继续中止的任务
            *value.0.borrow_mut() += 1;
            rt_copy.wakeup(&id_copy);
        }) {
            println!("!!!> spawn waker failed, id: {:?}, uid: {:?}, reason: {:?}", id, uid_copy, e);
        }
    }

    thread::sleep(Duration::from_millis(100000000));
}

struct TestFuture1(SyncUsize, TaskId, MultiTaskRuntime<()>);

unsafe impl Send for TestFuture1 {}
unsafe impl Sync for TestFuture1 {}

impl Future for TestFuture1 {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = *(self.as_ref().0).0.borrow();
        if index % 2 == 0 {
            self.2.pending(&self.1, cx.waker().clone())
        } else {
            Poll::Ready("future ready".to_string())
        }
    }
}

impl TestFuture1 {
    pub fn new(rt: MultiTaskRuntime<()>, index: SyncUsize, uid: TaskId) -> Self {
        TestFuture1(index, uid, rt)
    }
}

#[test]
fn test_multi_task() {
    let pool = MultiTaskRuntimeBuilder::default();
    let rt = pool.build();

    let mut ids = Vec::with_capacity(50);
    for index in 0..100 {
        let uid = rt.alloc();
        let uid_copy = uid.clone();
        let value = SyncUsize(Arc::new(RefCell::new(index)));
        let future = TestFuture1::new(rt.clone(), value.clone(), uid.clone());
        if let Err(e) = rt.spawn(uid.clone(), async move {
            println!("!!!!!!async task start, uid: {:?}", uid_copy);
            let r = future.await;
            println!("!!!!!!async task finish, uid: {:?}, r: {:?}", uid_copy, r);
        }) {
            println!("!!!> spawn task failed, uid: {:?}, reason: {:?}", uid, e);
        }

        if index % 2 == 0 {
            ids.push((uid, value));
        }
    }

    thread::sleep(Duration::from_millis(3000));

    for (id, value) in ids {
        let id_copy = id.clone();
        let uid = rt.alloc();
        let rt_copy = rt.clone();
        if let Err(e) = rt.spawn(uid.clone(), async move {
            //修改值，并继续中止的任务
            *value.0.borrow_mut() += 1;
            rt_copy.wakeup(&id_copy);
        }) {
            println!("!!!> spawn waker failed, id: {:?}, uid: {:?}, reason: {:?}", id, uid, e);
        }
    }

    thread::sleep(Duration::from_millis(100000000));
}

struct AtomicCounter(AtomicUsize, Instant);
impl Drop for AtomicCounter {
    fn drop(&mut self) {
        unsafe {
            println!("!!!!!!drop counter, count: {:?}, time: {:?}", self.0.load(Ordering::Relaxed), Instant::now() - self.1);
        }
    }
}

#[test]
fn test_empty_single_task() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    //测试派发定时任务的性能
    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..10000000 {
            let counter_copy = counter.clone();
            if let Err(e) = rt.spawn(rt.alloc(), async move {
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            }) {
                println!("!!!> spawn empty singale task failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!spawn single timing task ok, time: {:?}", Instant::now() - start);
    }

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_empty_multi_task() {
    let pool = MultiTaskRuntimeBuilder::default();;
    let rt = pool.build();
    let rt0 = rt.clone();
    let rt1 = rt.clone();
    let rt2 = rt.clone();
    let rt3 = rt.clone();

    //测试派发定时任务的性能
    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let counter0 = counter.clone();
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..2500000 {
                let counter_copy = counter0.clone();
                if let Err(e) = rt0.spawn(rt0.alloc(), async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }) {
                    println!("!!!> spawn empty singale task failed, reason: {:?}", e);
                }
            }
            println!("!!!!!!spawn single timing task ok 0, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let counter_copy = counter1.clone();
                if let Err(e) = rt1.spawn(rt1.alloc(), async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }) {
                    println!("!!!> spawn empty singale task failed, reason: {:?}", e);
                }
            }
            println!("!!!!!!spawn single timing task ok 1, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let counter_copy = counter2.clone();
                if let Err(e) = rt2.spawn(rt2.alloc(), async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }) {
                    println!("!!!> spawn empty singale task failed, reason: {:?}", e);
                }
            }
            println!("!!!!!!spawn single timing task ok 2, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let counter_copy = counter3.clone();
                if let Err(e) = rt3.spawn(rt3.alloc(), async move {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }) {
                    println!("!!!> spawn empty singale task failed, reason: {:?}", e);
                }
            }
            println!("!!!!!!spawn single timing task ok 3, time: {:?}", Instant::now() - start);
        });
    }

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_single_timing_task() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    //测试派发定时异步任务和取消定时异步任务的功能
    {
        for index in 0..10 {
            match rt.spawn_timing(rt.alloc(), async move {
                println!("!!!!!!run timing task ok, index: {}", index);
            }, 5000) {
                Err(e) => {
                    println!("!!!> spawn task failed, index: {:?}, reason: {:?}", index, e);
                },
                Ok(handle) => {
                    if index % 2 != 0 {
                        // rt.cancel_timing(handle);
                    }
                },
            }
        }
    }
    thread::sleep(Duration::from_millis(8000));

    //测试派发定时任务的性能
    let mut handles = Vec::with_capacity(10000000);
    let start = Instant::now();
    for index in 0..10000000 {
        match rt.spawn_timing(rt.alloc(), async move {
            println!("!!!!!!run timing task ok, index: {}", index);
        }, 10000) {
            Err(e) => {
                println!("!!!> spawn task failed, reason: {:?}", e);
            },
            Ok(handle) => {
                handles.push(handle);
            },
        }
    }
    println!("!!!!!!spawn single timing task ok, time: {:?}", Instant::now() - start);

    //测试取消定时任务的性能
    let start = Instant::now();
    for handle in handles {
        // rt.cancel_timing(handle);
    }
    println!("!!!!!!cancel single timing task ok, time: {:?}", Instant::now() - start);

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_multi_timing_task() {
    let pool = MultiTaskRuntimeBuilder::default();
    let rt = pool.build();

    //测试派发定时异步任务和取消定时异步任务的功能
    {
        for index in 0..10 {
            match rt.spawn_timing(rt.alloc(), async move {
                println!("!!!!!!run timing task ok, index: {}", index);
            }, 5000) {
                Err(e) => {
                    println!("!!!> spawn task failed, index: {:?}, reason: {:?}", index, e);
                },
                Ok(handle) => {
                    if index % 2 != 0 {
                        // rt.cancel_timing(handle);
                    }
                },
            }
        }
    }
    thread::sleep(Duration::from_millis(6000));

    //测试派发定时任务的性能
    let mut handles = Vec::with_capacity(10000000);
    let start = Instant::now();
    for index in 0..10000000 {
        match rt.spawn_timing(rt.alloc(), async move {
            println!("!!!!!!run timing task ok, index: {}", index);
        }, 10000) {
            Err(e) => {
                println!("!!!> spawn task failed, reason: {:?}", e);
            },
            Ok(handle) => {
                handles.push(handle);
            },
        }
    }
    println!("!!!!!!spawn multi timing task ok, time: {:?}", Instant::now() - start);

    //测试取消定时任务的性能
    let start = Instant::now();
    for handle in handles {
        // rt.cancel_timing(handle);
    }
    println!("!!!!!!cancel multi timing task ok, time: {:?}", Instant::now() - start);

    thread::sleep(Duration::from_millis(100000000));
}

//一个AsyncValue任务由2个异步任务组成，不包括创建AsyncValue的异步任务
#[test]
fn test_async_value() {
    use std::mem;

    let runner = SingleTaskRunner::default();
    let rt0 = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default()
        .init_worker_size(4)
        .set_worker_limit(4, 4);
    let rt1 = pool.build();

    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..10000000 {
            let rt0_copy = rt0.clone();
            let counter_copy = counter.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Local(rt0_copy.clone()));
                let value_copy = value.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt0.spawn(rt0.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    }
    thread::sleep(Duration::from_millis(30000));

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let counter0 = counter.clone();
    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let counter3 = counter.clone();
    mem::drop(counter);

    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter0.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt1_clone.spawn(rt1_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });

    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter1.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt1_clone.spawn(rt1_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });

    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter2.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt1_clone.spawn(rt1_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });

    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter3.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt1_clone.spawn(rt1_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });
    thread::sleep(Duration::from_millis(30000));

    let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
    let counter0 = counter.clone();
    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let counter3 = counter.clone();
    mem::drop(counter);

    let rt0_copy = rt0.clone();
    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt0_clone = rt0_copy.clone();
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter0.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt0_clone.spawn(rt0_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });

    let rt0_copy = rt0.clone();
    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt0_clone = rt0_copy.clone();
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter1.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt0_clone.spawn(rt0_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });

    let rt0_copy = rt0.clone();
    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt0_clone = rt0_copy.clone();
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter2.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt0_clone.spawn(rt0_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });

    let rt0_copy = rt0.clone();
    let rt1_copy = rt1.clone();
    thread::spawn(move || {
        let start = Instant::now();
        for _ in 0..2500000 {
            let rt0_clone = rt0_copy.clone();
            let rt1_clone = rt1_copy.clone();
            let counter_copy = counter3.clone();
            let future = async move {
                let value = AsyncValue::new(AsyncRuntime::Multi(rt1_clone.clone()));
                let value_copy = value.clone();
                rt0_clone.spawn(rt0_clone.alloc(), async move {
                    value_copy.set(true);
                });
                value.await;
                counter_copy.0.fetch_add(1, Ordering::Relaxed);
            };
            rt1_copy.spawn(rt1_copy.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    });
    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_async_wait_timeout() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..1000 {
        let rt_copy = rt.clone();
        let counter_copy = counter.clone();
        rt.spawn(rt.alloc(), async move {
            rt_copy.wait_timeout(5000).await;
            counter_copy.fetch_add(1, Ordering::Relaxed);
        });
    }

    thread::sleep(Duration::from_millis(20000));
    println!("!!!!!!count: {:?}", counter.load(Ordering::Relaxed));

    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..1000 {
        let rt0_copy = rt0.clone();
        let counter_copy = counter.clone();
        rt0.spawn(rt0.alloc(), async move {
            rt0_copy.wait_timeout(3000).await;
            counter_copy.fetch_add(1, Ordering::Relaxed);
        });
    }

    thread::sleep(Duration::from_millis(20000));
    println!("!!!!!!count: {:?}", counter.load(Ordering::Relaxed));
}

//一个AsyncWait任务由3个异步任务组成，不包括创建AsyncWait的异步任务
#[test]
fn test_async_wait() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskRuntimeBuilder::<()>::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::<()>::default();
    let rt1 = pool.build();

    {
        let rt_copy = rt.clone();
        let rt0_copy = rt0.clone();
        let rt1_copy = rt1.clone();
        let future = async move {
            let r = rt_copy.clone().wait(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                rt0_copy.wait(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                    rt1_copy.wait(AsyncRuntime::Local(rt_copy), async move {
                        Ok(true)
                    }).await
                }).await
            }).await;

            match r {
                Err(e) => {
                    println!("!!!!!!wait failed, reason: {:?}", e);
                },
                Ok(result) => {
                    println!("!!!!!!wait ok, result: {:?}", result);
                },
            }
        };
        rt.spawn(rt.alloc(), future);
    }
    thread::sleep(Duration::from_millis(1000));

    {
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();

        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let counter0 = counter.clone();
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..2500000 {
                let rt0_copy = rt0_0.clone();
                let counter_copy = counter0.clone();
                let future = async move {
                    if let Ok(r) = rt0_copy.clone().wait(AsyncRuntime::Multi(rt0_copy), async move {
                        Ok(1)
                    }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_0.spawn(rt0_0.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let rt0_copy = rt0_1.clone();
                let counter_copy = counter1.clone();
                let future = async move {
                    if let Ok(r) = rt0_copy.clone().wait(AsyncRuntime::Multi(rt0_copy), async move {
                        Ok(1)
                    }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_1.spawn(rt0_1.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let rt0_copy = rt0_2.clone();
                let counter_copy = counter2.clone();
                let future = async move {
                    if let Ok(r) = rt0_copy.clone().wait(AsyncRuntime::Multi(rt0_copy), async move {
                        Ok(1)
                    }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_2.spawn(rt0_2.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let rt0_copy = rt0_3.clone();
                let counter_copy = counter3.clone();
                let future = async move {
                    if let Ok(r) = rt0_copy.clone().wait(AsyncRuntime::Multi(rt0_copy), async move {
                        Ok(1)
                    }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_3.spawn(rt0_3.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }
    thread::sleep(Duration::from_millis(60000));

    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..1000000 {
            let rt_copy = rt.clone();
            let rt0_copy = rt0.clone();
            let rt1_copy = rt1.clone();
            let counter_copy = counter.clone();
            let future = async move {
                if let Ok(r) = rt_copy.clone().wait(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    rt0_copy.wait(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                        rt1_copy.clone().wait(AsyncRuntime::Local(rt_copy), async move {
                            Ok(1)
                        }).await
                    }).await
                }).await {
                    counter_copy.0.fetch_add(r, Ordering::Relaxed);
                }
            };
            rt.spawn(rt.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    }

    thread::sleep(Duration::from_millis(100000000));
}

//一个AsyncWaitAny任务由2 * n个异步任务组成，不包括创建AsyncWaitAny的异步任务
#[test]
fn test_async_wait_any() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    {
        let rt_copy = rt.clone();
        let rt0_copy = rt0.clone();
        let rt1_copy = rt1.clone();
        let future = async move {
            let f0 = Box::new(async move {
                let mut rng = rand::thread_rng();
                let timeout: u64 = rng.gen_range(0, 10000);
                thread::sleep(Duration::from_millis(timeout));
                Ok("rt0-".to_string() + timeout.to_string().as_str())
            }).boxed();
            let f1 = Box::new(async move {
                let mut rng = rand::thread_rng();
                let timeout: u64 = rng.gen_range(0, 10000);
                thread::sleep(Duration::from_millis(timeout));
                Ok("rt1-".to_string() + timeout.to_string().as_str())
            }).boxed();

            match rt_copy.wait_any(vec![(AsyncRuntime::Multi(rt0_copy), f0), (AsyncRuntime::Multi(rt1_copy), f1)]).await {
                Err(e) => {
                    println!("!!!!!!wait any failed, reason: {:?}", e);
                },
                Ok(result) => {
                    println!("!!!!!!wait any ok, result: {:?}", result);
                },
            }
        };
        rt.spawn(rt.alloc(), future);
    }
    thread::sleep(Duration::from_millis(10000));

    {
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();

        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let counter0 = counter.clone();
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..2500000 {
                let rt0_copy = rt0_0.clone();
                let counter_copy = counter0.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_0.spawn(rt0_0.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let rt0_copy = rt0_1.clone();
                let counter_copy = counter1.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_1.spawn(rt0_1.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let rt0_copy = rt0_2.clone();
                let counter_copy = counter2.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_2.spawn(rt0_2.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let rt0_copy = rt0_3.clone();
                let counter_copy = counter3.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_3.spawn(rt0_3.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }
    thread::sleep(Duration::from_millis(70000));

    {
        let rt_0 = rt.clone();
        let rt_1 = rt.clone();
        let rt_2 = rt.clone();
        let rt_3 = rt.clone();
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();
        let rt1_0 = rt1.clone();
        let rt1_1 = rt1.clone();
        let rt1_2 = rt1.clone();
        let rt1_3 = rt1.clone();

        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let counter0 = counter.clone();
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..2500000 {
                let rt_copy = rt_0.clone();
                let rt0_copy = rt0_0.clone();
                let rt1_copy = rt1_0.clone();
                let counter_copy = counter0.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let rt_copy = rt_1.clone();
                let rt0_copy = rt0_1.clone();
                let rt1_copy = rt1_1.clone();
                let counter_copy = counter1.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let rt_copy = rt_2.clone();
                let rt0_copy = rt0_2.clone();
                let rt1_copy = rt1_2.clone();
                let counter_copy = counter2.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let rt_copy = rt_3.clone();
                let rt0_copy = rt0_3.clone();
                let rt1_copy = rt1_3.clone();
                let counter_copy = counter3.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)]).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }

    thread::sleep(Duration::from_millis(100000000));
}

//一个AsyncWaitAnyCallbck任务由2 * n个异步任务组成，不包括创建AsyncWaitAnyCallback的异步任务
#[test]
fn test_async_wait_any_callback() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    {
        let rt_copy = rt.clone();
        let rt0_copy = rt0.clone();
        let rt1_copy = rt1.clone();
        let future = async move {
            let f0 = Box::new(async move {
                let mut rng = rand::thread_rng();
                let timeout: u64 = rng.gen_range(0, 10000);
                thread::sleep(Duration::from_millis(timeout));
                Ok("rt0-".to_string() + timeout.to_string().as_str())
            }).boxed();
            let f1 = Box::new(async move {
                let mut rng = rand::thread_rng();
                let timeout: u64 = rng.gen_range(0, 10000);
                thread::sleep(Duration::from_millis(timeout));
                Ok("rt1-".to_string() + timeout.to_string().as_str())
            }).boxed();

            match rt_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt0_copy), f0), (AsyncRuntime::Multi(rt1_copy), f1)],
                                            move |result| {
                                                true
                                            }).await {
                Err(e) => {
                    println!("!!!!!!wait any failed, reason: {:?}", e);
                },
                Ok(result) => {
                    println!("!!!!!!wait any ok, result: {:?}", result);
                },
            }
        };
        rt.spawn(rt.alloc(), future);
    }
    thread::sleep(Duration::from_millis(10000));

    {
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();

        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let counter0 = counter.clone();
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..2500000 {
                let rt0_copy = rt0_0.clone();
                let counter_copy = counter0.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_0.spawn(rt0_0.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let rt0_copy = rt0_1.clone();
                let counter_copy = counter1.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_1.spawn(rt0_1.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let rt0_copy = rt0_2.clone();
                let counter_copy = counter2.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_2.spawn(rt0_2.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let rt0_copy = rt0_3.clone();
                let counter_copy = counter3.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt0_copy.clone()), f0), (AsyncRuntime::Multi(rt0_copy.clone()), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt0_3.spawn(rt0_3.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }
    thread::sleep(Duration::from_millis(70000));

    {
        let rt_0 = rt.clone();
        let rt_1 = rt.clone();
        let rt_2 = rt.clone();
        let rt_3 = rt.clone();
        let rt0_0 = rt0.clone();
        let rt0_1 = rt0.clone();
        let rt0_2 = rt0.clone();
        let rt0_3 = rt0.clone();
        let rt1_0 = rt1.clone();
        let rt1_1 = rt1.clone();
        let rt1_2 = rt1.clone();
        let rt1_3 = rt1.clone();

        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let counter0 = counter.clone();
        let counter1 = counter.clone();
        let counter2 = counter.clone();
        let counter3 = counter.clone();

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 0..2500000 {
                let rt_copy = rt_0.clone();
                let rt0_copy = rt0_0.clone();
                let rt1_copy = rt1_0.clone();
                let counter_copy = counter0.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 2500000..5000000 {
                let rt_copy = rt_1.clone();
                let rt0_copy = rt0_1.clone();
                let rt1_copy = rt1_1.clone();
                let counter_copy = counter1.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 5000000..7500000 {
                let rt_copy = rt_2.clone();
                let rt0_copy = rt0_2.clone();
                let rt1_copy = rt1_2.clone();
                let counter_copy = counter2.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });

        thread::spawn(move || {
            let start = Instant::now();
            for _ in 7500000..10000000 {
                let rt_copy = rt_3.clone();
                let rt0_copy = rt0_3.clone();
                let rt1_copy = rt1_3.clone();
                let counter_copy = counter3.clone();
                let future = async move {
                    let f0 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    let f1 = Box::new(async move {
                        Ok(1)
                    }).boxed();
                    if let Ok(r) = rt0_copy.wait_any_callback(vec![(AsyncRuntime::Multi(rt1_copy.clone()), f0), (AsyncRuntime::Multi(rt1_copy), f1)],
                                                              move |result| {
                                                                  true
                                                              }).await {
                        counter_copy.0.fetch_add(r, Ordering::Relaxed);
                    }
                };
                rt_copy.spawn(rt_copy.alloc(), future);
            }
            println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
        });
    }

    thread::sleep(Duration::from_millis(100000000));
}

//一个AsyncWaitAll任务由2 * n个异步任务组成，不包括创建AsyncWaitAll的异步任务
#[test]
fn test_async_wait_all() {
    let runner = SingleTaskRunner::default();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool: MultiTaskRuntimeBuilder<()> = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    let pool: MultiTaskRuntimeBuilder<()> = MultiTaskRuntimeBuilder::default();
    let rt1 = pool.build();

    {
        struct SendableFn(Box<dyn FnOnce(&mut Vec<u8>) -> Vec<u8> + Send + 'static>);

        let rt_copy = rt.clone();
        let rt0_copy = rt0.clone();
        rt.spawn(rt.alloc(), async move {
            let mut map_reduce = rt_copy.map_reduce(10);

            let cb: SendableFn = SendableFn(Box::new(move |v: &mut Vec<u8>| {
                v.clone()
            }));
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(cb)
            });

            let cb: SendableFn = SendableFn(Box::new(move |v: &mut Vec<u8>| {
                v.clone()
            }));
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(cb)
            });

            let mut vec = vec![0xff, 0xff, 0xff];
            for r in map_reduce.reduce(true).await.unwrap() {
                if let Ok(cb) = r {
                    assert_eq!(cb.0(&mut vec), vec);
                }
            }
        });
    }
    thread::sleep(Duration::from_millis(1000));

    {
        let rt_copy = rt.clone();
        let rt0_copy = rt0.clone();
        let rt1_copy = rt1.clone();
        let future = async move {
            let mut map_reduce = rt_copy.map_reduce(10);
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(0)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(1)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(2)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(3)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(4)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(5)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(6)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(7)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(8)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(9)
            });

            println!("!!!!!!map result: {:?}", map_reduce.reduce(false).await);

            let mut map_reduce = rt_copy.map_reduce(10);
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(0)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(1)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(2)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(3)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(4)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(5)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(6)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(7)
            });
            map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                Ok(8)
            });
            map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                Ok(9)
            });

            println!("!!!!!!map result by order: {:?}", map_reduce.reduce(true).await);
        };
        rt.spawn(rt.alloc(), future);
    }
    thread::sleep(Duration::from_millis(1000));

    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..1000000 {
            let rt0_copy = rt0.clone();
            let counter_copy = counter.clone();
            let future = async move {
                let mut map_reduce = rt0_copy.map_reduce(10);
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(0)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(1)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(2)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(3)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(4)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(5)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(6)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(7)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(8)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(9)
                });
                if let Ok(_) = map_reduce.reduce(true).await {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }
            };
            rt0.spawn(rt0.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    }
    thread::sleep(Duration::from_millis(30000));

    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..1000000 {
            let rt_copy = rt.clone();
            let rt0_copy = rt0.clone();
            let rt1_copy = rt1.clone();
            let counter_copy = counter.clone();
            let future = async move {
                let mut map_reduce = rt_copy.map_reduce(10);
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(0)
                });
                map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                    Ok(1)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(2)
                });
                map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                    Ok(3)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(4)
                });
                map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                    Ok(5)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(6)
                });
                map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                    Ok(7)
                });
                map_reduce.map(AsyncRuntime::Multi(rt0_copy.clone()), async move {
                    Ok(8)
                });
                map_reduce.map(AsyncRuntime::Multi(rt1_copy.clone()), async move {
                    Ok(9)
                });
                if let Ok(_) = map_reduce.reduce(true).await {
                    counter_copy.0.fetch_add(1, Ordering::Relaxed);
                }
            };
            rt.spawn(rt.alloc(), future);
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    }

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_worker_runtime() {
    let thread_status = Arc::new(AtomicBool::new(true));
    let runner = SingleTaskRunner::default();
    let thread_waker = runner.get_thread_waker().unwrap();
    let rt = AsyncRuntime::Worker(Arc::new(AtomicBool::new(true)), thread_waker.clone(), runner.startup().unwrap());

    let rt_copy = rt.clone();
    spawn_worker_thread("Test-Worker-Runtime",
                        1024 * 1024,
                        thread_status,
                        thread_waker,
                        1000,
                        None,
                        move || {
                            let start = Instant::now();
                            if let Ok(len) = runner.run() {
                                if len > 0 {
                                    (false, Instant::now() - start)
                                } else {
                                    (true, Instant::now() - start)
                                }
                            } else {
                                (true, Instant::now() - start)
                            }
                        },
                        move || {
                            rt_copy.len()
                        });

    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();

    {
        let counter = Arc::new(AtomicCounter(AtomicUsize::new(0), Instant::now()));
        let start = Instant::now();
        for _ in 0..10000000 {
            let rt0_copy = rt0.clone();
            let counter_copy = counter.clone();
            rt.spawn(rt.alloc(), async move {
                let result = AsyncValue::new(AsyncRuntime::Multi(rt0_copy.clone()));
                let result_copy = result.clone();
                rt0_copy.spawn(rt0_copy.alloc(), async move {
                    result_copy.set(1);
                });
                counter_copy.0.fetch_add(result.await, Ordering::Relaxed);
            });
        }
        println!("!!!!!!spawn ok, time: {:?}", Instant::now() - start);
    }

    thread::sleep(Duration::from_millis(1000000000));
}

#[test]
fn test_panic_handler() {
    register_global_panic_handler(|thread: thread::Thread, info, other, location| {
        println!("!!!!!!thread: {:?}", thread);
        println!("!!!!!!info: {}", info);
        println!("!!!!!!other: {:?}", other);
        println!("!!!!!!location: {:?}", location);

        Some(0)
    });

    fn test() {
        test0();
    }

    fn test0() {
        panic!("Test panic!, {}", true);
    }

    thread::Builder::new()
        .name("Test panic thread".to_string())
        .spawn(|| {
        test();
    });

    thread::sleep(Duration::from_millis(10000));
}
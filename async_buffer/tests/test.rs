use std::thread;
use std::sync::Arc;
use std::time::{Duration, Instant};

use r#async::rt::{AsyncRuntime, multi_thread::MultiTaskRuntimeBuilder};

use async_buffer::{CallbackResult, AsyncBytesBufferBuilder};

#[test]
fn test_async_buffer() {
    let builder = MultiTaskRuntimeBuilder::default();
    let rt = builder.build();

    let builder = AsyncBytesBufferBuilder::new("Test-Async-Buffer",
                                              8192,
                                              10);
    let buffer = builder.build(AsyncRuntime::Multi(rt.clone()),
                              move |buf: Vec<Arc<Vec<u8>>>, buf_size: usize, capacity: usize| {
                                    let mut real_size = 0;
                                    for b in &buf {
                                        real_size += b.len();
                                    }
                                    assert_eq!(buf_size, real_size);
                                    println!("!!!!!!capacity callback ok, buf_size: {}", buf_size);

                                    capacity
                                },
                              move |buf: Vec<Arc<Vec<u8>>>, buf_size: usize, timeout: usize| {
                                    let mut real_size = 0;
                                    for b in &buf {
                                        real_size += b.len();
                                    }
                                    assert_eq!(buf_size, real_size);
                                    println!("!!!!!!timeout callback ok, buf_size: {}", buf_size);

                                    CallbackResult::Continue(timeout)
                                });
    let buffer0 = buffer.clone();
    let buffer1 = buffer.clone();

    rt.spawn(rt.alloc(), async move {
        for _ in 0..10000 {
            if let Err(e) = buffer0.async_push(Arc::new(vec![0xff; 32])).await {
                panic!("!!!!!!test_async_buffer failed, reason: {:?}", e);
            }
        }
    });

    for _ in 0..10000 {
        buffer1.push(Arc::new(vec![0xff; 32]));
    }

    thread::sleep(Duration::from_millis(1000000000));
}

#[test]
fn test_async_buffer_performance() {
    let builder = MultiTaskRuntimeBuilder::default();
    let rt = builder.build();

    let builder = AsyncBytesBufferBuilder::new("Test-Async-Buffer",
                                               8192,
                                               10);
    let buffer = builder.build(AsyncRuntime::Multi(rt.clone()),
                               move |buf: Vec<Arc<Vec<u8>>>, _buf_size: usize, capacity: usize| {
                                   capacity
                               },
                               move |buf: Vec<Arc<Vec<u8>>>, _buf_size: usize, timeout: usize| {
                                   CallbackResult::Continue(timeout)
                               });

    rt.spawn(rt.alloc(), async move {
        let now = Instant::now();
        for _ in 0..10000000 {
            if let Err(e) = buffer.async_push(Arc::new(vec![0xff; 32])).await {
                panic!("!!!!!!test_async_buffer failed, reason: {:?}", e);
            }
        }
        println!("!!!!!!time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(1000000000));
}
use std::thread;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_channel::{Sender, Receiver, unbounded};

use local_timer::LocalTimer;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Token(pub usize);

#[test]
fn test_local_timer() {
    let mut timeout = 0;
    let mut timer = LocalTimer::with_tick(10);
    for n in 0..20 {
        timeout = n * 5;
        timer.set_timeout(Token(timeout), timeout);
    }

    let now = Instant::now();
    for _ in 0..10 {
        println!("time: {:?}", now.elapsed().as_millis());
        thread::sleep(Duration::from_millis(10));
        let mut tokens = timer.poll();
        tokens.sort();
        for token in tokens {
            println!("\ttoken: {:?}", token);
        }
    }
}

#[test]
fn test_try_poll() {
    let mut timeout = 0;
    let mut timer = LocalTimer::with_tick(10);
    for n in 0..20 {
        timeout = n * 5;
        timer.set_timeout(Token(timeout), timeout);
    }

    let mut timeout = 10;
    let now = Instant::now();
    for _ in 0..50 {
        println!("time: {:?}, remaining len: {}", now.elapsed().as_millis(), timer.len());
        if let Some(mut token) = timer.try_pop() {
            println!("\ttoken: {:?}", token);
        }

        let diff_time = timer.try_poll();
        if diff_time >= 10 {
            timeout = 0;
        } else {
            timeout = 10 - diff_time;
        }

        thread::sleep(Duration::from_millis(timeout));
    }
}

#[test]
fn test_set_and_pop_task() {
    struct TimerTask(usize);

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_copy = counter.clone();
    let mut timer = LocalTimer::with_tick(10);
    let (sender, receiver) = unbounded();
    thread::Builder::new().spawn(move || {
        let counter_ref = &counter_copy;
        loop {
            //设置定时任务
            for (timeout, task) in receiver.try_iter().collect::<Vec<(usize, TimerTask)>>() {
                timer.set_timeout(task, timeout);
            }

            //获取到期的定时任务
            let mut diff_time = 0;
            let task = timer.try_pop();
            if task.is_none() {
                //没有到期的定时任务，则尝试推动定时器
                diff_time = timer.try_poll();
            }

            //执行到期的定时任务
            if let Some(task) = task {
                counter_ref.fetch_add(1, Ordering::Relaxed);
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    for index in 0..1000 {
        sender.send((10, TimerTask(index)));
    }

    thread::sleep(Duration::from_millis(10000));
    println!("!!!!!!task count: {}", counter.load(Ordering::Relaxed));
}
extern crate lfstack;

use std::thread;
use std::collections::LinkedList;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, RwLock};

use lfstack::{CollectResult, LFStack, pause};

#[test]
fn test_mutex_stack() {
    let stack = Arc::new(Mutex::new(LinkedList::new()));

    let stack0 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack0.lock().unwrap().push_front(n);
        }
        println!("!!!!!!stack0 push finish, time: {:?}", Instant::now() - now);
    });

    let stack1 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 1000000..2000000 {
            stack1.lock().unwrap().push_front(n);
        }
        println!("!!!!!!stack1 push finish, time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(100));

    let stack3 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            if let Some(_) = stack3.lock().unwrap().pop_front() {
                continue;
            } else {
                println!("!!!!!!stack3 pop error, n: {}", n);
                return;
            }
        }
        println!("!!!!!!stack3 pop finish, time: {:?}", Instant::now() - now);
    });

    let stack4 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 1000000..2000000 {
            if let Some(_) = stack4.lock().unwrap().pop_front() {
                continue;
            } else {
                println!("!!!!!!stack4 pop error, n: {}", n);
                return;
            }
        }
        println!("!!!!!!stack4 pop finish, time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(10000000000));
}

#[test]
fn test_rwlock_stack() {
    let stack = Arc::new(RwLock::new(LinkedList::new()));

    let stack0 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack0.write().unwrap().push_front(n);
        }
        println!("!!!!!!stack0 push finish, time: {:?}", Instant::now() - now);
    });

    let stack1 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 1000000..2000000 {
            stack1.write().unwrap().push_front(n);
        }
        println!("!!!!!!stack1 push finish, time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(100));

    let stack3 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            if let Some(_) = stack3.write().unwrap().pop_front() {
                continue;
            } else {
                println!("!!!!!!stack3 pop error, n: {}", n);
                return;
            }
        }
        println!("!!!!!!stack3 pop finish, time: {:?}", Instant::now() - now);
    });

    let stack4 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 1000000..2000000 {
            if let Some(_) = stack4.write().unwrap().pop_front() {
                continue;
            } else {
                println!("!!!!!!stack4 pop error, n: {}", n);
                return;
            }
        }
        println!("!!!!!!stack4 pop finish, time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(10000000000));
}

#[test]
fn test_lfstack() {
    println!("======test push and pop");
    let stack = Arc::new(LFStack::new());

    let stack0 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack0.push(n);
        }
        println!("!!!!!!stack0 push finish, time: {:?}", Instant::now() - now);
    });

    let stack1 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 1000000..2000000 {
            stack1.push(n);
        }
        println!("!!!!!!stack1 push finish, time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(100));

    let stack3 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            if let Some(val) = stack3.pop() {
                continue;
            } else {
                println!("!!!!!!stack3 pop error, n: {}", n);
                return;
            }
        }
        println!("!!!!!!stack3 pop finish, time: {:?}", Instant::now() - now);
    });

    let stack4 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 1000000..2000000 {
            if let Some(val) = stack4.pop() {
                continue;
            } else {
                println!("!!!!!!stack4 pop error, n: {}", n);
                return;
            }
        }
        println!("!!!!!!stack4 pop finish, time: {:?}", Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(3000));
    println!("======test collect_from_top by continue");

    let stack7 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack7.push(n);
        }
        println!("!!!!!!stack7 push finish, stack size: {:?}, time: {:?}", stack7.size(), Instant::now() - now);
    });

    let cache = Arc::new(Mutex::new(Vec::new()));
    let cache_copy = cache.clone();
    let stack11 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        stack11.collect_from_top(Arc::new(move |frame| {
            cache_copy.lock().unwrap().push(frame.clone());
            CollectResult::Continue(true)
        }));
        println!("!!!!!!stack11 collect finish, stack size: {:?}, cache size: {:?}, time: {:?}", stack11.size(), cache.lock().unwrap().len(), Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(3000));
    println!("======test collect_from_bottom by continue");

    //为下个测试清理栈
    for _ in 0..stack.size() {
        stack.pop();
    }

    let stack13 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack13.push(n);
        }
        println!("!!!!!!stack13 push finish, stack size: {:?}, time: {:?}", stack13.size(), Instant::now() - now);
    });

    let cache = Arc::new(Mutex::new(Vec::new()));
    let cache_copy = cache.clone();
    let stack14 = stack.clone();
    thread::spawn(move || {
        let now = Instant::now();
        stack14.collect_from_bottom(Arc::new(move |frame| {
            cache_copy.lock().unwrap().push(frame.clone());
            CollectResult::Continue(true)
        }));
        println!("!!!!!!stack14 collect finish, stack size: {:?}, cache size: {:?}, time: {:?}", stack14.size(), cache.lock().unwrap().len(), Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(3000));
    println!("======test collect_from_top by break");

    //为下个测试清理栈
    for _ in 0..stack.size() {
        stack.pop();
    }

    let stack15 = stack.clone();
    let t0 = thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack15.push(n);
        }
        println!("!!!!!!stack15 push finish, stack size: {:?}, time: {:?}", stack15.size(), Instant::now() - now);
    });

    let cache = Arc::new(Mutex::new(Vec::new()));
    let cache_copy = cache.clone();
    let stack17 = stack.clone();
    let t1 = thread::spawn(move || {
        let now = Instant::now();
        let mut n = 10; //重试最大次数
        while n > 0 {
            if stack17.size() == 0 {
                n -= 1;
                pause();
                continue;
            }

            let cache_clone = cache_copy.clone();
            stack17.collect_from_top(Arc::new(move |frame| {
                cache_clone.lock().unwrap().push((*frame));
                CollectResult::Break(true)
            }));
        }
        println!("!!!!!!stack17 collect finish, stack size: {:?}, cache size: {:?}, time: {:?}", stack17.size(), cache_copy.lock().unwrap().len(), Instant::now() - now);
    });

    t0.join();
    t1.join();
    assert_eq!(cache.lock().unwrap().len(), 1000000);
    assert_eq!(cache.lock().unwrap().iter().map(|x| { x.clone() }).sum::<i32>(), (0..1000000).map(|x| { x.clone() }).sum());

    println!("======test collect_from_bottom by break");

    //为测试上个整理是否将栈清空，直接开始推入数据
    let stack21 = stack.clone();
    let t0 = thread::spawn(move || {
        let now = Instant::now();
        for n in 0..1000000 {
            stack21.push(n);
        }
        println!("!!!!!!stack21 push finish, stack size: {:?}, time: {:?}", stack21.size(), Instant::now() - now);
    });

    let cache = Arc::new(Mutex::new(Vec::new()));
    let cache_copy = cache.clone();
    let stack23 = stack.clone();
    let t1 = thread::spawn(move || {
        let now = Instant::now();
        let mut n = 10; //重试最大次数
        while n > 0 {
            if stack23.size() == 0 {
                n -= 1;
                pause();
                continue;
            }

            let cache_clone = cache_copy.clone();
            stack23.collect_from_bottom(Arc::new(move |frame| {
                cache_clone.lock().unwrap().push((*frame));
                CollectResult::Break(true)
            }));
        }
        println!("!!!!!!stack23 collect finish, stack size: {:?}, cache size: {:?}, time: {:?}", stack23.size(), cache_copy.lock().unwrap().len(), Instant::now() - now);
    });

    t0.join();
    t1.join();
    assert_eq!(cache.lock().unwrap().len(), 1000000);
    assert_eq!(cache.lock().unwrap().iter().map(|x| { x.clone() }).sum::<i32>(), (0..1000000).map(|x| { x.clone() }).sum());
}
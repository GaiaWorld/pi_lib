extern crate lfstack;

use std::thread;
use std::collections::LinkedList;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, RwLock};

use lfstack::LFStack;

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
        stack11.collect(Arc::new(move |frame| {
            cache_copy.lock().unwrap().push(frame.clone());
            true
        }));
        println!("!!!!!!stack11 collect finish, stack size: {:?}, cache size: {:?}, time: {:?}", stack11.size(), cache.lock().unwrap().len(), Instant::now() - now);
    });

    thread::sleep(Duration::from_millis(10000000000));
}
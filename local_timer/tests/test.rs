use std::thread;
use std::time::{Instant, Duration};

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

    let now = Instant::now();
    for _ in 0..10 {
        println!("time: {:?}", now.elapsed().as_millis());
        thread::sleep(Duration::from_millis(10));
        while let Some(mut token) = timer.try_poll() {
            println!("\ttoken: {:?}", token);
        }
    }
}
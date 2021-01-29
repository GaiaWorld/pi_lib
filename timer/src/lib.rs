#![feature(integer_atomics)]

extern crate atom;
extern crate apm;
extern crate wheel;
extern crate time;

#[macro_use]
extern crate lazy_static;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicU64};
use std::mem::{transmute};
use std::marker::Send;
use std::fmt::{Debug, Formatter, Result as FResult};

use atom::Atom;
use apm::counter::{GLOBAL_PREF_COLLECT, PrefCounter, PrefTimer};
use wheel::slab_wheel::Wheel;
use wheel::wheel::Item;
use time::{run_millis};

lazy_static! {
    //定时器数量
    pub static ref TIMER_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("timer_count"), 0).unwrap();
    //创建定时任务数量
    pub static ref TIMER_CREATE_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("timer_create_count"), 0).unwrap();
    //取消定时任务数量
    pub static ref TIMER_CANCEL_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("timer_cancel_count"), 0).unwrap();
    //定时任务运行数量
    pub static ref TIMER_RUN_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("timer_run_count"), 0).unwrap();
    //定时任务运行总时长
	pub static ref TIMER_RUN_TIME: PrefTimer = GLOBAL_PREF_COLLECT.new_static_timer(Atom::from("timer_run_time"), 0).unwrap();
}

pub trait Runer{
    fn run(self, index: usize);
}

impl<T: 'static + Send + Runer> Timer<T>{
    pub fn new(clock_ms: u64) -> Self {
        TIMER_COUNT.sum(1);

        Timer(Arc::new(Mutex::new(TimerImpl::new(clock_ms))))
    }

    #[allow(unused_must_use)]
    pub fn run(&self){
        let s = self.0.clone();
		thread::Builder::new()
            .name("Timer".to_string())
            .spawn(move ||{
                let mut sleep_time = s.lock().unwrap().clock_ms;
				let start_time = run_millis();
                loop {
                    thread::sleep(Duration::from_millis(sleep_time));
                    let mut now = run_millis();
                    run_zero(&s);//运行0毫秒任务
                    loop {
                        let mut r = {
							let mut s = s.lock().unwrap();
							let next_roll_time = s.clock_ms + s.wheel.get_time() + start_time;
                            match now >= next_roll_time{
                                true => s.wheel.roll(),
                                false => {
                                    sleep_time = next_roll_time - now;
                                    break;
                                }
                            }
						};
						
                        run_task(&s, &mut r);
						run_zero(&s);//运行0毫秒任务
						now = run_millis();
                    }
                }
		});
	}

    pub fn set_timeout(&self, elem: T, ms: u32) -> usize{
        TIMER_CREATE_COUNT.sum(1);

        let mut lock = self.0.lock().unwrap();
		let time =  lock.wheel.get_time();
		lock.wheel.insert(Item{elem: elem, time_point: time + (ms as u64)})
	}

    pub fn cancel(&self, index: usize) -> Option<T>{
        let mut lock = self.0.lock().unwrap();
		match lock.wheel.try_remove(index) {
			Some(v) => {
                TIMER_CANCEL_COUNT.sum(1);

                Some(v.elem)
            },
			None => {None},
		}
	}

    pub fn clear(&self){
        self.0.lock().unwrap().clear();
	}
}

pub struct TimerImpl<T: Send + Runer>{
	wheel: Wheel<T>,
	_statistics: Statistics,
	clock_ms: u64,
}

impl<T: Send + Runer> TimerImpl<T>{
	pub fn new(mut clock_ms: u64) -> Self{
        if clock_ms < 10{
            clock_ms = 10;
        }
		TimerImpl{
			wheel: Wheel::new(), 
			_statistics: Statistics::new(),
			clock_ms: clock_ms,
		}
    }

    pub fn clear(&mut self){
        self.wheel.clear();
	}
}

pub struct FuncRuner(usize, usize);

impl FuncRuner{
    pub fn new(f: Box<dyn FnOnce()>) -> Self {
        unsafe { transmute(f) }
    }
}

impl Runer for FuncRuner {
    fn run(self, _index: usize){
        let func: Box<dyn FnOnce()> = unsafe { transmute((self.0, self.1)) };
        func();
    }
}

impl Debug for FuncRuner {
	fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,"FuncRuner")
    }
}


lazy_static! {
	pub static ref TIMER: Timer<FuncRuner> = Timer::new(10);
}

pub struct Timer<T: 'static + Send + Runer>(pub Arc<Mutex<TimerImpl<T>>>);

impl<T: 'static + Send + Runer> Clone for Timer<T> {
    fn clone(&self) -> Self{
        Timer(self.0.clone())
    }
}

#[derive(Clone)]
struct Statistics {
	pub all_count: Arc<AtomicUsize>,
    pub cancel_count: Arc<AtomicUsize>,
	pub run_count: Arc<AtomicUsize>,
	pub run_time: Arc<AtomicU64>,
}

impl Statistics{
	pub fn new() -> Statistics{
		Statistics{
			all_count: Arc::new(AtomicUsize::new(0)),
            cancel_count: Arc::new(AtomicUsize::new(0)),
			run_count: Arc::new(AtomicUsize::new(0)),
			run_time: Arc::new(AtomicU64::new(0)),
		}
	}
}


fn run_zero<T: Send + Runer>(timer: &Arc<Mutex<TimerImpl<T>>>){
    loop {
        let mut r = {
            let mut s = timer.lock().unwrap();
            match s.wheel.zero_size() > 0{
                true => s.wheel.get_zero(),
                false => {
                    break;
                }
            }
        };
        run_task(timer, &mut r);
        r.clear();
        timer.lock().unwrap().wheel.set_zero_cache(r);
    }
}

//执行任务，返回任务执行完的时间
fn run_task<T: Send + Runer>(_timer: &Arc<Mutex<TimerImpl<T>>>, r: &mut Vec<(Item<T>, usize)>){
    let start = TIMER_RUN_TIME.start();
	let mut j = r.len();
	TIMER_RUN_COUNT.sum(r.len());
    for _ in 0..r.len(){
        j -= 1;
        let e = r.remove(j);
        e.0.elem.run(e.1);
    }
	
    
    TIMER_RUN_TIME.timing(start);
}
#[test]
fn test(){
    TIMER.run();
    //thread::sleep(Duration::from_millis(8));
    //let now = now_millis();
    TIMER.set_timeout(FuncRuner::new(Box::new(move||{
        println!("test timer Success");
	})), 10);
	//let index = TIMER.set_timeout(Box::new(f), 1000);
    //println!("index-------------{}", index.load(Ordering::Relaxed));
	thread::sleep(Duration::from_millis(500));
}

#[cfg(test)]
extern crate rand;

// 测试定时任务弹出数量是否和插入数量保持一致
#[test]
fn test_count() {
    use rand::thread_rng;
    use rand::Rng;
	use rand::seq::SliceRandom;
	use std::collections::HashMap;

    TIMER.run();
    let mut rng = thread_rng();

    let count = Arc::new(AtomicUsize::new(0));

	let mut timer_refs = vec![];
	let mut timer_map = HashMap::new();
	let total = 100000;
    for _i in 1..total + 1 {
		let count = count.clone();
		let t = rng.gen_range(10, 5000);
        let timer_ref  =TIMER.set_timeout(FuncRuner::new(Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        })), t);

		timer_refs.push(timer_ref);
		timer_map.insert(timer_ref, true);
	}


	let cancel: Vec<usize> = timer_refs.choose_multiple(&mut rng, 50).cloned().collect();
	// let cancel = Vec::new();

    println!("shuffled timer_refs = {:?}", cancel);

	let mut cancel_success = Vec::new();
	let mut cancel_fail = Vec::new();
    for c in cancel {
        if let Some(_) = TIMER.cancel(c) {
			// println!("cancel success {:?}", c);
			cancel_success.push(c);
        } else {
			// println!("cancel failed {:?}", c);
			cancel_fail.push(c);
        }
        thread::sleep(Duration::from_millis(rng.gen_range(10, 100)));
    }

    thread::sleep(Duration::from_millis(5100));
	println!("run: {:?}, total: {:?}, cancel_success: {}, cancel_fail:{}", count, total, cancel_success.len(), cancel_fail.len());
}


// 测试定时器得延时情况
#[test]
fn test_timer_delay() {
    use rand::thread_rng;
    use rand::Rng;

	TIMER.run();
    let mut rng = thread_rng();
    for _i in 1..100000 {
		let t = rng.gen_range(10, 10000);
		let time = run_millis();
        TIMER.set_timeout(FuncRuner::new(Box::new(move || {
			if run_millis() as isize - time as isize - t as isize > 11 || -11 > run_millis() as isize - time as isize - t as isize {
				println!("task delay================{}, {}", run_millis() as isize - time as isize - t as isize, t as u64 + time);
			}
		})), t);
	}
	
	thread::sleep(Duration::from_millis(11000));
}

// test_timer_delay中如果存在某个时间延迟较多，则可在此测试中单独测试该时间任务是否准时
#[test] 
fn test_timer_delay_single() {
	TIMER.set_timeout(FuncRuner::new(Box::new(move || {
		println!("task================{}", run_millis());
	})), 6692);

	println!("start================{}", run_millis());
	TIMER.run();
	thread::sleep(Duration::from_millis(11000));
}

// #[test]
// fn test_timer2() {
//     use rand::thread_rng;
//     use rand::Rng;
// 	use rand::seq::SliceRandom;
// 	use std::collections::HashMap;

// 	TIMER.run();
//     let mut rng = thread_rng();

//     let count = Arc::new(AtomicUsize::new(0));

// 	// let mut timer_refs = Arc::new(Mutex::new(vec![]));
// 	// let mut timer_map = HashMap::new();

// 		let count = count.clone();
// 		let t = 9231;
// 		let time = run_millis();
//         let timer_ref  =TIMER.set_timeout(FuncRuner::new(Box::new(move || {
// 			// count.fetch_add(1, Ordering::SeqCst);
// 			// t1.lock().unwrap().push(( run_millis() as isize - time as isize - t as isize, t, run_millis()));
// 			println!("==============={}, {}, now:{}", run_millis() as isize - time as isize - t as isize, t, run_millis());
// 		})), t);
	
	
// 	// 	timer_refs.push(timer_ref);
// 	// 	timer_map.insert(timer_ref, true);
// 	// }

//     // thread::sleep(Duration::from_millis(11000));
// 	// println!("count = {:?}, {:?}", count, TIMER_RUN_COUNT.get());

// 	// let time = run_millis();
// 	// let timer_ref  =TIMER.set_timeout(FuncRuner::new(Box::new(move || {
		
// 	// 	println!("=================={}", run_millis() - time);
// 	// })), 10000);
	
// 	thread::sleep(Duration::from_millis(13000));
// 	println!("timer_refs1======================{:?}", timer_refs1.lock().unwrap().as_slice());
// }

// #[test]
// fn test_wheel() {
//     use rand::thread_rng;
//     use rand::Rng;
// 	use rand::seq::SliceRandom;
// 	use std::collections::HashMap;
// 	use wheel::wheel::Item;
// 	let mut rng = thread_rng();
// 	let mut timer_refs = Arc::new(Mutex::new(vec![]));

// 	let mut r = Wheel::new();
// 	for i in 1..1000 {
// 		let t = rng.gen_range(10, 10000);
// 		let time = run_millis();
// 		let t1 = timer_refs.clone();
// 		r.insert(Item{
// 			elem: FuncRuner::new(Box::new(move || {
// 				// t1.lock().unwrap().push(( run_millis() as isize - time as isize - t as isize, t));
// 			})),
// 			time_point: run_millis() + t,
// 		});
// 	}

// 	let mut count=  0;
// 	while count < 1000 {
// 		count +=  1;
// 		let time = run_millis();
// 		r.roll();
// 		timer_refs.lock().unwrap().push( run_millis() as isize - time as isize);
// 	}
// 	// let mut timer_map = HashMap::new();
    
	
// 	// 	timer_refs.push(timer_ref);
// 	// 	timer_map.insert(timer_ref, true);
// 	// }

//     // thread::sleep(Duration::from_millis(11000));
// 	// println!("count = {:?}, {:?}", count, TIMER_RUN_COUNT.get());

// 	// let time = run_millis();
// 	// let timer_ref  =TIMER.set_timeout(FuncRuner::new(Box::new(move || {
		
// 	// 	println!("=================={}", run_millis() - time);
// 	// })), 10000);
// 	thread::sleep(Duration::from_millis(13000));
// 	println!("======================{:?}", timer_refs);
// }

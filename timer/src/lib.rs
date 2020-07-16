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
                let mut sleep_time = {
                    let mut lock = s.lock().unwrap();
                    lock.wheel.set_time(run_millis());
                    lock.clock_ms
                };
                loop {
                    thread::sleep(Duration::from_millis(sleep_time));
                    let mut now = run_millis();
                    now = run_zero(&s, now);//运行0毫秒任务
                    loop {
                        let mut r = {
                            let mut s = s.lock().unwrap();
                            match now >= s.clock_ms + s.wheel.get_time(){
                                true => s.wheel.roll(),
                                false => {
                                    sleep_time = s.clock_ms + s.wheel.get_time()- now;
                                    break;
                                }
                            }
                        };
                        now = run_task(&s, &mut r);
                        now = run_zero(&s, now);//运行0毫秒任务
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
	statistics: Statistics,
	clock_ms: u64,
}

impl<T: Send + Runer> TimerImpl<T>{
	pub fn new(mut clock_ms: u64) -> Self{
        if clock_ms < 10{
            clock_ms = 10;
        }
		TimerImpl{
			wheel: Wheel::new(), 
			statistics: Statistics::new(),
			clock_ms: clock_ms,
		}
	}

    pub fn clear(&mut self){
        self.wheel.clear();
	}
}

pub struct FuncRuner(usize, usize);

impl FuncRuner{
    pub fn new(f: Box<FnOnce()>) -> Self {
        unsafe { transmute(f) }
    }
}

impl Runer for FuncRuner {
    fn run(self, _index: usize){
        let func: Box<FnOnce()> = unsafe { transmute((self.0, self.1)) };
        func();
    }
}

impl Debug for FuncRuner {
	fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,"F")
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


fn run_zero<T: Send + Runer>(timer: &Arc<Mutex<TimerImpl<T>>>, mut now: u64) -> u64{
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
        now = run_task(timer, &mut r);
        r.clear();
        timer.lock().unwrap().wheel.set_zero_cache(r);
    }
    now
}

//执行任务，返回任务执行完的时间
fn run_task<T: Send + Runer>(timer: &Arc<Mutex<TimerImpl<T>>>, r: &mut Vec<(Item<T>, usize)>) -> u64{
    let start = TIMER_RUN_TIME.start();
	let mut j = r.len();
	TIMER_RUN_COUNT.sum(r.len());
    for _ in 0..r.len(){
        j -= 1;
        let e = r.remove(j);
        e.0.elem.run(e.1);
    }
	
    
    TIMER_RUN_TIME.timing(start);
    run_millis()
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

#[test]
fn test_timer() {
    use rand::thread_rng;
    use rand::Rng;
	use rand::seq::SliceRandom;
	use std::collections::HashMap;

    TIMER.run();
    let mut rng = thread_rng();

    let count = Arc::new(AtomicUsize::new(0));

	let mut timer_refs = vec![];
	let mut timer_map = HashMap::new();
	// let mut pop = Arc::new(Mutex::new(Vec::new()));
	// let mut run_success_map = HashMap::new();
	// let mut run_success_vec = Vec::new();
    for i in 1..100001 {
		// let map1 = &mut run_success_map;
		// let vec1 = &mut run_success_vec;
		let count = count.clone();
		let t = rng.gen_range(10, 5000);
        let timer_ref  =TIMER.set_timeout(FuncRuner::new(Box::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
			// println!("timer task {:?}", i);
			// map1.insert(i, true);
			// vec1.push(i);
			// pop.lock.push(i);
        })), t);

		// if let Some(r) = timer_map.get(&timer_ref) {
		// 	panic!("error:{}", timer_ref);
		// }
		timer_refs.push(timer_ref);
		timer_map.insert(timer_ref, true);
	}
	

    // println!("timer refs = {:?}, len:{}", timer_refs,timer_refs.len());


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

    thread::sleep(Duration::from_millis(7000));

	println!("cancel success: {:?}, cancel fail: {:?}", cancel_success.len(), cancel_fail.len());
	
	// for cs in cancel_success.iter() {
	// 	match run_success_map.get(cs) {
	// 		Some(_) => {
	// 			panic!("xxxxxxxxxxxxxxxxxxxxxx");
	// 		}
	// 	}
	// }

	// for cs in cancel_fail.iter() {
	// 	match run_success_map.get(cs) {
	// 		None => {panic!("xxxxxxxxxxxxxxxxxxxxxx");}
	// 	}
	// }

	

	println!("count = {:?}, {:?}", count, TIMER_RUN_COUNT.get());
	// for i of cancel {}
}


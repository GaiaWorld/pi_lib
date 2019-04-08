#![feature(integer_atomics)]
#![feature(fnbox)]

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
use std::boxed::FnBox;
use std::mem::{transmute};
use std::marker::Send;

use atom::Atom;
use apm::counter::{GLOBAL_PREF_COLLECT, PrefCounter, PrefTimer};
use wheel::slab_wheel::Wheel;
use wheel::wheel::Item;
use time::{now_millis};

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

    pub fn run(&self){
        let s = self.0.clone();
		thread::Builder::new()
            .name("Timer".to_string())
            .spawn(move ||{
                let mut sleep_time = {
                    let mut lock = s.lock().unwrap();
                    lock.wheel.set_time(now_millis());
                    lock.clock_ms
                };
                loop {
                    thread::sleep(Duration::from_millis(sleep_time));
                    let mut now = now_millis();
                    now = run_zero(&s);//运行0毫秒任务
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
                        now = run_zero(&s);//运行0毫秒任务
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
    pub fn new(f: Box<FnBox()>) -> Self {
        unsafe { transmute(f) }
    }
}

impl Runer for FuncRuner {
    fn run(self, _index: usize){
        let func: Box<FnBox()> = unsafe { transmute((self.0, self.1)) };
        func();
    }
}


lazy_static! {
	pub static ref TIMER: Timer<FuncRuner> = Timer::new(10);
}

pub struct Timer<T: 'static + Send + Runer>(Arc<Mutex<TimerImpl<T>>>);

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


fn run_zero<T: Send + Runer>(timer: &Arc<Mutex<TimerImpl<T>>>) -> u64{
    let mut now = 0;
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
    for _ in 0..r.len(){
        j -= 1;
        let e = r.remove(j);
        e.0.elem.run(e.1);
    }

    TIMER_RUN_COUNT.sum(1);
    TIMER_RUN_TIME.timing(start);

    now_millis()
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


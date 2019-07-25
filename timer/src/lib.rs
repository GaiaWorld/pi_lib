#![feature(integer_atomics)]

extern crate atom;
extern crate apm;
extern crate wheel;
extern crate ver_index;
extern crate time;

#[macro_use]
extern crate lazy_static;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration};
use std::sync::atomic::{AtomicUsize, AtomicU64};
use std::mem::{transmute};
use std::marker::Send;
use std::marker::PhantomData;

use atom::Atom;
use apm::counter::{GLOBAL_PREF_COLLECT, PrefCounter, PrefTimer};
use wheel::slab_wheel::SlabWheel;
use ver_index::VerIndex;
use ver_index::bit::BitIndex;
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

pub trait Runer<ID>{
    fn run(self, id: ID);
}

impl<T: 'static + Send + Runer<I::ID>, I:VerIndex+Default+Send+Sync+'static> Timer<T, I>{
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
                        now = run_task::<T, I>(&mut r);
                        now = run_zero(&s, now);//运行0毫秒任务
                    }
                }
		}).unwrap();
	}

    pub fn set_timeout(&self, elem: T, ms: u32) -> I::ID{
        TIMER_CREATE_COUNT.sum(1);

        let mut lock = self.0.lock().unwrap();
        let time =  lock.wheel.get_time();
		lock.wheel.insert(Item{elem: elem, time_point: time + (ms as u64)})
	}

    pub fn cancel(&self, index: I::ID) -> Option<T>{
        let mut lock = self.0.lock().unwrap();
		match lock.wheel.remove(index) {
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

pub struct TimerImpl<T: Send + Runer<I::ID>, I:VerIndex>{
	wheel: SlabWheel<T, I>,
	statistics: Statistics,
	clock_ms: u64,
}

impl<T: Send + Runer<I::ID>, I:VerIndex+Default> TimerImpl<T, I>{
	pub fn new(mut clock_ms: u64) -> Self{
        if clock_ms < 10{
            clock_ms = 10;
        }
		TimerImpl{
			wheel: SlabWheel::default(),
			statistics: Statistics::new(),
			clock_ms: clock_ms,
		}
	}

    pub fn clear(&mut self){
        self.wheel.clear();
	}
}

pub struct FuncRuner<T>(usize, usize, PhantomData<T>);
impl<T> Default for FuncRuner<T> {
    fn default() -> Self {
        FuncRuner(0, 0, PhantomData)
    }
}
impl<T> FuncRuner<T>{
    pub fn new(f: Box<dyn Fn()>) -> Self {
        unsafe { transmute(f) }
    }
}

impl<ID> Runer<ID> for FuncRuner<ID> {
    fn run(self, _id: ID){
        let func: Box<dyn Fn()> = unsafe { transmute((self.0, self.1)) };
        func();
    }
}


lazy_static! {
	pub static ref TIMER: Timer<FuncRuner<usize>, BitIndex> = Timer::new(10);
}

pub struct Timer<T: 'static + Send + Runer<I::ID>, I: VerIndex>(Arc<Mutex<TimerImpl<T, I>>>);

impl<T: 'static + Send + Runer<I::ID>, I: VerIndex> Clone for Timer<T, I> {
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


fn run_zero<T: Send + Runer<I::ID>, I:VerIndex>(timer: &Arc<Mutex<TimerImpl<T, I>>>, mut now: u64) -> u64{
    loop {
        let mut r:Vec<(Item<T>, I::ID)> = {
            let mut s = timer.lock().unwrap();
            let temp = s.wheel.replace_zero_cache(Vec::new());
            match s.wheel.zero_size() > 0{
                true => s.wheel.get_zero(temp),
                false => {
                    break;
                }
            }
        };
        now = run_task::<T, I>(&mut r);
        r.clear();
        timer.lock().unwrap().wheel.replace_zero_cache(r);
    }
    now
}

//执行任务，返回任务执行完的时间
fn run_task<T: Send + Runer<I::ID>, I:VerIndex>(r: &mut Vec<(Item<T>, I::ID)>) -> u64{
    let start = TIMER_RUN_TIME.start();
    loop {
        match r.pop() {
            Some(e) => e.0.elem.run(e.1),
            _ => break
        }
    }
    TIMER_RUN_COUNT.sum(1);
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


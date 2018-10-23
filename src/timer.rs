use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicU64};
use std::boxed::FnBox;
use std::mem::{transmute};
use std::marker::Send;

use wheel::{Wheel, Item};
use time::{now_millis};

pub trait Runer{
    fn run(self, index: &Arc<AtomicUsize>);
}

impl<T: 'static + Send + Runer> Timer<T>{
    pub fn new(clock_ms: u64) -> Self {
        Timer(Arc::new(Mutex::new(TimerImpl::new(clock_ms))))
    }

    pub fn run(&self){
        let s = self.0.clone();
		thread::spawn(move ||{
            let mut sleep_time = {
                let mut lock = s.lock().unwrap();
                lock.wheel.set_time(now_millis());
                lock.clock_ms
            };
			loop {
                thread::sleep(Duration::from_millis(sleep_time));
                let mut now = now_millis();
                now = run_zero(&s, now);//运行0毫秒任务
                loop {
                    let mut r = {
                        let mut s = s.lock().unwrap();
                        match now >= s.clock_ms + s.wheel.time{
                            true => s.wheel.roll(),
                            false => {
                                sleep_time = s.clock_ms + s.wheel.time- now;
                                break;
                            }
                        }
                    };
                    now = run_task(&s, &mut r, now);
                    now = run_zero(&s, now);//运行0毫秒任务
                }
			}
		});
	}

    pub fn set_timeout(&self, elem: T, ms: u32) -> Arc<AtomicUsize>{
        let mut lock = self.0.lock().unwrap();
		lock.statistics.all_count.fetch_add(1, Ordering::Relaxed);
        let time =  lock.wheel.time;
		lock.wheel.insert(Item{elem: elem, time_point: time + (ms as u64)})
	}

    pub fn cancel(&self, index: &Arc<AtomicUsize>) -> Option<T>{
        let mut lock = self.0.lock().unwrap();
		match lock.wheel.try_remove(index) {
			Some(v) => {
                lock.statistics.cancel_count.fetch_add(1, Ordering::Relaxed);
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
    fn run(self, _index: &Arc<AtomicUsize>){
        let func: Box<FnBox()> = unsafe { transmute((self.0, self.1)) };
        func();
    }
}


lazy_static! {
	pub static ref TIMER: Timer<FuncRuner> = Timer::new(10);
}

pub struct Timer<T: 'static + Send + Runer>(Arc<Mutex<TimerImpl<T>>>);

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
        now = run_task(timer, &mut r, now);
        r.clear();
        timer.lock().unwrap().wheel.set_zero_cache(r);
    }
    now
}

//执行任务，返回任务执行完的时间
fn run_task<T: Send + Runer>(timer: &Arc<Mutex<TimerImpl<T>>>, r: &mut Vec<(Item<T>, Arc<AtomicUsize>)>, old: u64) -> u64{
    let mut j = r.len();
    {
        let lock = timer.lock().unwrap();
        lock.statistics.run_count.fetch_add(r.len(), Ordering::Relaxed);//统计运行任务个数
    }
    for _ in 0..r.len(){
        j -= 1;
        let e = r.remove(j);
        e.0.elem.run(&e.1);
    }
    let now = now_millis();
    timer.lock().unwrap().statistics.run_time.fetch_add(now - old, Ordering::Relaxed); //统计运行时长
    now
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


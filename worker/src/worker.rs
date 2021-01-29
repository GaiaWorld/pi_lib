use std::thread::park_timeout;
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use std::fmt::{Display, Formatter, Result};
use std::sync::atomic::{Ordering, AtomicUsize};

use threadpool::ThreadPool;

use atom::Atom;
use apm::counter::{GLOBAL_PREF_COLLECT, PrefCounter, PrefTimer};

use task::Task;
use task_pool::TaskPool;
use task_pool::enums::Task as BaseTask;

lazy_static! {
    //虚拟机动态同步任务弹出数量
    static ref JS_DYNAMIC_SYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_dynamic_sync_task_pop_count"), 0).unwrap();
    //虚拟机静态异步任务弹出数量
    static ref JS_STATIC_ASYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_static_async_task_pop_count"), 0).unwrap();
    //虚拟机静态同步任务弹出数量
    static ref JS_STATIC_SYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_static_sync_task_pop_count"), 0).unwrap();
    //存储动态同步任务弹出数量
    static ref STORE_DYNAMIC_SYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_dynamic_sync_task_pop_count"), 0).unwrap();
    //存储静态异步任务弹出数量
    static ref STORE_STATIC_ASYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_static_async_task_pop_count"), 0).unwrap();
    //存储静态同步任务弹出数量
    static ref STORE_STATIC_SYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_static_sync_task_pop_count"), 0).unwrap();
    //网络动态同步任务弹出数量
    static ref NET_DYNAMIC_SYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_dynamic_sync_task_pop_count"), 0).unwrap();
    //网络静态异步任务弹出数量
    static ref NET_STATIC_ASYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_static_async_task_pop_count"), 0).unwrap();
    //网络静态同步任务弹出数量
    static ref NET_STATIC_SYNC_TASK_POP_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_static_sync_task_pop_count"), 0).unwrap();
    //虚拟机慢任务数量
    static ref JS_SLOW_TASK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_slow_task_count"), 0).unwrap();
    //虚拟机慢任务总时长
    static ref JS_SLOW_TASK_TIME: PrefTimer = GLOBAL_PREF_COLLECT.new_static_timer(Atom::from("js_slow_task_time"), 0).unwrap();
    //虚拟机异常任务数量
    static ref JS_PANIC_TASK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("js_panic_task_count"), 0).unwrap();
    //存储慢任务数量
    static ref STORE_SLOW_TASK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_slow_task_count"), 0).unwrap();
    //存储慢任务总时长
    static ref STORE_SLOW_TASK_TIME: PrefTimer = GLOBAL_PREF_COLLECT.new_static_timer(Atom::from("store_slow_task_time"), 0).unwrap();
    //存储异常任务数量
    static ref STORE_PANIC_TASK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("store_panic_task_count"), 0).unwrap();
    //网络慢任务数量
    static ref NET_SLOW_TASK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_slow_task_count"), 0).unwrap();
    //网络慢任务总时长
    static ref NET_SLOW_TASK_TIME: PrefTimer = GLOBAL_PREF_COLLECT.new_static_timer(Atom::from("net_slow_task_time"), 0).unwrap();
    //网络异常任务数量
    static ref NET_PANIC_TASK_COUNT: PrefCounter = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("net_panic_task_count"), 0).unwrap();
}

/*
* 工作者状态
*/
#[derive(Clone)]
pub enum WorkerStatus {
    Stop = 0,   //停止
    Wait,       //等待
    Running,    //运行中
}

/*
* 工作者类型
*/
#[derive(Debug, Clone)]
pub enum WorkerType {
    Normal = 0, //通用
    Js,         //虚拟机
    Store,      //存储
    Net,        //网络
}

impl ToString for WorkerType {
    fn to_string(&self) -> String {
        match self {
            &WorkerType::Normal => String::from("Normal Task"),
            &WorkerType::Js => String::from("JS Task"),
            &WorkerType::Store => String::from("Store Task"),
            &WorkerType::Net => String::from("Net Task"),
        }
    }
}

/*
* 工作者
*/
#[derive(Debug)]
pub struct Worker {
    uid:            u32,            //工作者编号
    slow:           Duration,       //工作者慢任务时长，单位us
    status:         AtomicUsize,    //工作者状态
    worker_type:    WorkerType,     //工作者类型
    static_async:   PrefCounter,    //工作者静态异步任务计数器
    static_sync:    PrefCounter,    //工作者静态同步任务计数器
    dynamic_sync:   PrefCounter,    //工作者动态同步任务计数器
    slow_counter:   PrefCounter,    //工作者慢任务计数器
    slow_timer:     PrefTimer,      //工作者慢任务计时器
    panic_counter:  PrefCounter,    //工作者异常任务计数器
}

unsafe impl Sync for Worker {} //声明保证多线程安全性

impl Display for Worker {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Worker[uid = {}, slow = {:?}, status = {}, static_async = {}, static_sync = {}, dynamic_sync = {}]",
            self.uid, self.slow, self.status.load(Ordering::Relaxed), self.static_async.get(), self.static_sync.get(), self.dynamic_sync.get())
	}
}

impl Worker {
    //创建一个工作者
    pub fn new(worker_type: WorkerType, uid: u32, slow: u32) -> Self {
        let (static_async, static_sync, dynamic_sync, slow_counter, slow_timer, panic_counter) = match worker_type {
            WorkerType::Normal => {
                (JS_STATIC_ASYNC_TASK_POP_COUNT.clone(),
                 JS_STATIC_SYNC_TASK_POP_COUNT.clone(),
                 JS_DYNAMIC_SYNC_TASK_POP_COUNT.clone(),
                 JS_SLOW_TASK_COUNT.clone(),
                 JS_SLOW_TASK_TIME.clone(),
                 JS_PANIC_TASK_COUNT.clone())
            },
            WorkerType::Js => {
                (JS_STATIC_ASYNC_TASK_POP_COUNT.clone(),
                 JS_STATIC_SYNC_TASK_POP_COUNT.clone(),
                 JS_DYNAMIC_SYNC_TASK_POP_COUNT.clone(),
                 JS_SLOW_TASK_COUNT.clone(),
                 JS_SLOW_TASK_TIME.clone(),
                 JS_PANIC_TASK_COUNT.clone())
            },
            WorkerType::Store => {
                (STORE_STATIC_ASYNC_TASK_POP_COUNT.clone(),
                 STORE_STATIC_SYNC_TASK_POP_COUNT.clone(),
                 STORE_DYNAMIC_SYNC_TASK_POP_COUNT.clone(),
                 STORE_SLOW_TASK_COUNT.clone(),
                 STORE_SLOW_TASK_TIME.clone(),
                 STORE_PANIC_TASK_COUNT.clone())
            },
            WorkerType::Net => {
                (NET_STATIC_ASYNC_TASK_POP_COUNT.clone(),
                 NET_STATIC_SYNC_TASK_POP_COUNT.clone(),
                 NET_DYNAMIC_SYNC_TASK_POP_COUNT.clone(),
                 NET_SLOW_TASK_COUNT.clone(),
                 NET_SLOW_TASK_TIME.clone(),
                 NET_PANIC_TASK_COUNT.clone())
            },
        };

        Worker {
            uid:        uid,
            slow:       Duration::from_micros(slow as u64),
            status:     AtomicUsize::new(WorkerStatus::Wait as usize),
            worker_type,
            static_async,
            static_sync,
            dynamic_sync,
            slow_counter,
            slow_timer,
            panic_counter,
        }
    }

    //启动
    pub fn startup(pool: &ThreadPool, walker: Arc<(Mutex<bool>, Condvar)>, worker: Arc<Worker>, tasks: Arc<TaskPool<Task>>) -> bool {
        pool.execute(move|| {
            let mut task = Task::new();
            Worker::work_loop(walker, worker, tasks, &mut task);
        });
        true
    }

    //工作循环
    fn work_loop(walker: Arc<(Mutex<bool>, Condvar)>, worker: Arc<Worker>, tasks: Arc<TaskPool<Task>>, task: &mut Task) {
        let mut status: usize;
        loop {
            status = worker.get_status();
            //处理控制状态
            if status == WorkerStatus::Stop as usize {
                //退出当前循环
                break;
            } else if status == WorkerStatus::Wait as usize {
                //继续等待控制状态
                park_timeout(Duration::from_millis(1000));
                continue;
            } else if status == WorkerStatus::Running as usize {
                //继续工作
                worker.work(&walker, &tasks, task);
            }
        }
    }

    //获取工作者当前状态
    #[inline]
    pub fn get_status(&self) -> usize {
        self.status.load(Ordering::Relaxed)
    }

    //设置工作者当前状态
    pub fn set_status(&self, current: WorkerStatus, new: WorkerStatus) -> bool {
        match self.status.compare_exchange(current as usize, new as usize, Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => true,
            _ => false,
        }
    }

    //获取工作者的工作计数
    pub fn count(&self) -> usize {
        self.static_async.get() + self.static_sync.get() + self.dynamic_sync.get()
    }

    //关闭工作者
    pub fn stop(&self) -> bool {
        if self.get_status() == WorkerStatus::Stop as usize {
            return true;
        }
        match self.status.compare_exchange(WorkerStatus::Running as usize, WorkerStatus::Stop as usize,
            Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => true,
            _ => {
                match self.status.compare_exchange(WorkerStatus::Wait as usize, WorkerStatus::Stop as usize,
                    Ordering::Acquire, Ordering::Relaxed) {
                    Ok(_) => true,
                    _ => false,
                }
            },
        }
    }

    //工作
    fn work(&self, walker: &Arc<(Mutex<bool>, Condvar)>, tasks: &Arc<TaskPool<Task>>, task: &mut Task) {
        let base_task: BaseTask<Task>;
        //同步块
        {
            let &(ref lock, ref cvar) = &**walker;
            let mut wake = lock.lock().unwrap();
            while !*wake {
                //等待任务唤醒
                let (mut w, wait) = cvar.wait_timeout(wake, Duration::from_millis(100)).unwrap();
                if wait.timed_out() {
                    //等待超时，则继续工作
                    *w = true;
                    wake = w;
                } else {
                    wake = w;
                }
            }

            if let Some(t) = tasks.pop() {
                //有任务
                base_task = t;
            } else {
                //没有任务，则重置唤醒状态，立即解锁，并处理控制状态
                *wake = false;
                return;
            }
        }
        check_slow_task(self, task, base_task); //执行任务
    }
}

fn check_slow_task(_worker: &Worker, _task: &mut Task, _base_task: BaseTask<Task>) {
    // let mut lock = None;
    // match base_task {
    //     BaseTask::Async(t) => {
    //         //填充异步任务
    //         worker.static_async.sum(1);

    //         t.copy_to(task);
    //     },
    //     BaseTask::Sync(t, q) => {
    //         //填充同步任务
    //         if q < 0 {
    //             worker.static_sync.sum(1);
    //         } else {
    //             worker.dynamic_sync.sum(1);
    //         }

    //         t.copy_to(task);
    //         lock = Some(q);
    //     }
    // }
    
    // let time = worker.slow_timer.start();
    // if let Err(e) = panic::catch_unwind(|| { task.run(lock); }) {
    //     //执行任务异常
    //     worker.panic_counter.sum(1);

    //     let reason = match e.downcast_ref::<&str>() {
    //         Some(str) => Some(str.to_string()),
    //         None => {
    //             match e.downcast_ref::<String>() {
    //                 Some(string) => Some(string.to_string()),
    //                 None => None,
    //             }
    //         },
    //     };
    //     warn!("!!!> {} Run Error, time: {:?}, thread: {:?}, task: {}, e: {:?}", worker.worker_type.to_string(), Instant::now() - time, thread::current(), task, reason);
    // } else {
    //     //执行任务成功
    //     let elapsed = time.elapsed();
    //     if time.elapsed() >= worker.slow {
    //         //记录慢任务
    //         worker.slow_counter.sum(1);
    //         worker.slow_timer.timing(time);

    //         info!("===> Slow {}, time: {:?}, thread: {:?}, task: {}", worker.worker_type.to_string(), Instant::now() - time, thread::current(), task);
    //     }
    // }
}
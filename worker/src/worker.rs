use std::thread::park_timeout;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, Condvar};
use std::fmt::{Display, Formatter, Result};
use std::sync::atomic::{Ordering, AtomicUsize};

use threadpool::ThreadPool;

use task_pool::TaskPool;
use task::Task;

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
* 工作者
*/
#[derive(Debug)]
pub struct Worker {
    uid:        u32,            //工作者编号
    slow:       Duration,       //工作者慢任务时长，单位us
    status:     AtomicUsize,    //工作者状态
    counter:    AtomicUsize,    //工作者计数器
}

unsafe impl Sync for Worker {} //声明保证多线程安全性

impl Display for Worker {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Worker[uid = {}, slow = {:?}, status = {}, counter = {}]", 
            self.uid, self.slow, self.status.load(Ordering::Relaxed), self.counter.load(Ordering::Relaxed))
	}
}

impl Worker {
    //创建一个工作者
    pub fn new(uid: u32, slow: u32) -> Self {
        Worker {
            uid:        uid,
            slow:       Duration::from_micros(slow as u64),
            status:     AtomicUsize::new(WorkerStatus::Wait as usize),
            counter:    AtomicUsize::new(0),
        }
    }

    //启动
    pub fn startup(pool: &ThreadPool, worker: Arc<Worker>, sync: Arc<(Mutex<TaskPool>, Condvar)>) -> bool {
        pool.execute(move|| {
            let mut task = Task::new();
            Worker::work_loop(worker, sync, &mut task);
        });
        true
    }

    //工作循环
    fn work_loop(worker: Arc<Worker>, sync: Arc<(Mutex<TaskPool>, Condvar)>, task: &mut Task) {
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
                worker.work(&sync, task);
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
        self.counter.load(Ordering::Relaxed)
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
    fn work(&self, sync: &Arc<(Mutex<TaskPool>, Condvar)>, task: &mut Task) {
        //同步块
        {
            let &(ref lock, ref cvar) = &**sync;
            let mut task_pool = lock.lock().unwrap();
            while (*task_pool).size() == 0 {
                //等待任务
                let (pool, wait) = cvar.wait_timeout(task_pool, Duration::from_micros(1000)).unwrap();
                if wait.timed_out() {
                    return //等待超时，则立即解锁，并处理控制状态
                }
                task_pool = pool;
            }
            (*task_pool).pop(task); //获取任务
        }
        check_slow_task(self, task); //执行任务
        self.counter.fetch_add(1, Ordering::Acquire); //增加工作计数
    }
}

#[inline]
fn check_slow_task(worker: &Worker, task: &mut Task) {
    use std::panic;
    let time = Instant::now();
    if let Err(e) = panic::catch_unwind(|| { task.run(); }) {
        //执行任务失败
        let elapsed = time.elapsed();
        println!("!!!> Task Run Error, time: {}, task: {}, e: {:?}", elapsed.as_secs() * 1000000 + (elapsed.subsec_micros() as u64), task, e);
    } else {
        //执行任务成功
        let elapsed = time.elapsed();
        if time.elapsed() >= worker.slow {
            //记录慢任务
            println!("===> Slow Task, time: {}, task: {}", elapsed.as_secs() * 1000000 + (elapsed.subsec_micros() as u64), task);
        }
    }
}
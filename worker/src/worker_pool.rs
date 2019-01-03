use fnv::FnvHashMap;
use std::sync::{Arc, Mutex, Condvar};
use std::fmt::{Display, Formatter, Result as FmtResult}; //避免和标准Result冲突，改名为FmtResult

use threadpool::{ThreadPool, Builder as ThreadPoolBuilder};

use task_pool::TaskPool;

use task::Task;
use worker::{WorkerStatus, Worker};

/*
* 工作者池
*/
pub struct WorkerPool {
    counter:        u32,                            //工作者编号计数器
    map:            FnvHashMap<u32, Arc<Worker>>,   //工作者缓存
    thread_pool:    ThreadPool,                     //线程池
    walker:         Arc<(Mutex<bool>, Condvar)>,    //唤醒者
}

impl Display for WorkerPool {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let pool = self.thread_pool.clone();
		write!(f, "WorkerPool[counter = {}, worker_size = {}, wait_size = {}, active_size = {}, panic_size = {}]", 
        self.counter, self.size(), pool.queued_count(), pool.active_count(), pool.panic_count())
	}
}

impl WorkerPool {
    //构建指定数量工作者的工作者池
    pub fn new(len: usize, stack_size: usize, slow: u32, walker: Arc<(Mutex<bool>, Condvar)>) -> Self {
        let mut counter: u32 = 0;
        let mut map = FnvHashMap::default();
        for _ in 0..len {
            counter += 1;
            map.insert(counter, Arc::new(Worker::new(counter, slow)));
        }
        WorkerPool {
            counter:        counter,
            map:            map,
            thread_pool:    ThreadPoolBuilder::new().
                                                num_threads(len).
                                                thread_stack_size(stack_size).
                                                build(),
            walker:         walker,
        }
    }

    //获取工作者数量
    pub fn size(&self) -> u32 {
        self.map.len() as u32
    }

    //获取指定状态的工作者编号数组
    pub fn workers(&self, status: usize) -> Vec<u32> {
        let mut vec = Vec::<u32>::new();
        for (uid, worker) in self.map.iter() {
            if worker.get_status() == status {
                vec.push(*uid);
            }
        }
        vec
    }

    //休眠指定工作者
    pub fn sleep(&self, uid: u32) -> bool {
        match self.map.get(&uid) {
            Some(worker) => {
                worker.set_status(WorkerStatus::Running, WorkerStatus::Wait)
            },
            None => false,
        }
    }

    //唤醒指定工作者
    pub fn wakeup(&self, uid: u32) -> bool {
        match self.map.get(&uid) {
            Some(worker) => {
                worker.set_status(WorkerStatus::Wait, WorkerStatus::Running)
            },
            None => false,
        }
    }

    //停止指定工作者
    pub fn stop(&self, uid: u32) -> bool {
        match self.map.get(&uid) {
            Some(worker) => {
                worker.stop()
            },
            None => false,
        }
    }

    //启动工作者，启动时需要指定任务池的同步对象
    pub fn start(&self, pool: Arc<TaskPool<Task>>, uid: u32) -> bool {
        match self.map.get(&uid) {
            Some(worker) => {
                if worker.set_status(WorkerStatus::Stop, WorkerStatus::Running) {
                    Worker::startup(&self.thread_pool, self.walker.clone(), worker.clone(), pool.clone())
                } else {
                    false
                }
            },
            None => false,
        }
    }

    //在指定任务池中，运行工作池，需要指定任务池的同步对象
    pub fn run(&self, pool: Arc<TaskPool<Task>>) {
        pool.set_count(self.thread_pool.max_count()); //设置任务池线程数
        for (_, worker) in self.map.iter() {
            if worker.set_status(WorkerStatus::Wait, WorkerStatus::Running) {
                Worker::startup(&self.thread_pool, self.walker.clone(), worker.clone(), pool.clone());
            }
        }
    }

    //增加工作者
    pub fn increase(&mut self, pool: Arc<TaskPool<Task>>, len: usize, slow: u32) {
        if len == 0 {
            return;
        }

        let start = self.counter + 1;
        let mut worker: Arc<Worker>;
        for _ in 0..len {
            self.counter += 1;
            worker = Arc::new(Worker::new(self.counter, slow));
            worker.stop();
            self.map.insert(self.counter, worker.clone());
        }
        let end = self.counter + 1;
        self.thread_pool.set_num_threads(self.counter as usize);
        pool.set_count(self.thread_pool.max_count()); //重置任务池线程数
        for uid in start..end {
            self.start(pool.clone(), uid); //启动新创建的工作者
        }
    }

    //减少工作者
    pub fn decrease(&mut self, pool: Arc<TaskPool<Task>>, len: usize) {
        if len == 0 || len > self.counter as usize {
            return;
        }

        self.counter -= len as u32;
        let min = self.counter;
        //从工作池中移除已关闭的工作者
        self.map.retain(|&uid, worker| {
            //从尾部开始关闭工作者
            if uid > min {
                !worker.stop()
            } else {
                true
            }
        });
        self.thread_pool.set_num_threads(self.counter as usize);
        pool.set_count(self.thread_pool.max_count()); //重置任务池线程数
    }
}
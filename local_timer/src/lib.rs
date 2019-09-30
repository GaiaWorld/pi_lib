#![feature(integer_atomics)]

use std::mem::transmute;

use time::run_millis;
use wheel::{wheel::Item, slab_wheel::Wheel};

/*
* 本地定时器，由本地线程驱动
*/
pub struct LocalTimer<T: Send + 'static> {
    wheel:      Wheel<T>,       //定时轮
    tick_time:  usize,          //定时器最小定时间隔
}

impl<T: Send + 'static> LocalTimer<T>{
    //构建一个指定间隔时长的本地定时器，单位毫秒
    pub fn new() -> Self {
        LocalTimer::with_tick(10)
    }

    //构建一个指定间隔时长的本地定时器，单位毫秒
    pub fn with_tick(mut tick_time: usize) -> Self {
        if tick_time < 10{
            tick_time = 10;
        }

        LocalTimer{
            wheel: Wheel::new(),
            tick_time,
        }
    }

    //设置定时任务，返回任务句柄
    pub fn set_timeout(&mut self, task: T, timeout: usize) -> usize {
	    let item = Item {
            elem: task,
            time_point: self.wheel.get_time() + (timeout as u64)
        };
        self.wheel.insert(item)
    }

    //驱动定时器运行，已到时间的任务，会从定时器中移除，并返回
    pub fn poll(&mut self) -> Vec<T> {
        let mut tasks = Vec::new();
        let mut tick_time = self.tick_time;
        self.wheel.set_time(run_millis());

        poll_zero(self, &mut tasks);
        let mut r = self.wheel.roll();
        poll_task(self, &mut r, &mut tasks);

        while run_millis() >= self.tick_time as u64 + self.wheel.get_time() {
            r = self.wheel.roll();
            poll_task(self, &mut r, &mut tasks);
            poll_zero(self, &mut tasks);
        }

        tasks
    }

    //取消指定任务句柄的定时任务
    pub fn cancel(&mut self, index: usize) -> Option<T> {
        match self.wheel.try_remove(index) {
            None => None,
            Some(e) => {
                Some(e.elem)
            },
        }
    }

    //清空所有定时任务
    pub fn clear(&mut self) {
        self.wheel.clear();
    }
}

//驱动定时器运行超时时长为0的任务，返回已到时间的任务
fn poll_zero<T: Send + 'static>(timer: &mut LocalTimer<T>, tasks: &mut Vec<T>) {
    while timer.wheel.zero_size() > 0 {
        //如果有超时时长为0的任务
        let mut r = timer.wheel.get_zero();
        poll_task(timer, &mut r, tasks);
        r.clear();
        timer.wheel.set_zero_cache(r);
    }
}

//驱动定时器运行定时任务，返回已到时间的任务
fn poll_task<T: Send + 'static>(timer: &mut LocalTimer<T>, r: &mut Vec<(Item<T>, usize)>, tasks: &mut Vec<T>) {
    let mut j = r.len();
    for _ in 0..r.len(){
        j -= 1;
        let e = r.remove(j);
        tasks.push(e.0.elem);
    }
}



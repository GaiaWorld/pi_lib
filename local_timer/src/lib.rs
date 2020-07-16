#![feature(integer_atomics)]

use std::mem::transmute;

use time::run_millis;
use wheel::{wheel::Item, slab_wheel::Wheel};

/*
* 本地定时器，由本地线程推动
*/
pub struct LocalTimer<T: Send + 'static> {
    wheel:      Wheel<T>,       //定时轮
    tick_time:  usize,          //定时轮最小定时间隔
}

impl<T: Send + 'static> LocalTimer<T>{
    //构建一个指定间隔时长的本地定时器，单位毫秒
    pub fn new() -> Self {
        LocalTimer::with_tick(10)
    }

    //构建一个指定间隔时长的本地定时器，单位毫秒
    pub fn with_tick(mut tick_time: usize) -> Self {
        if tick_time < 10 {
            tick_time = 10;
        }

        let mut wheel = Wheel::new();
        wheel.set_time(run_millis()); //初始化定时轮上次推动的时间
        LocalTimer{
            wheel,
            tick_time,
        }
    }

    //获取当前未到期的定时任务
    pub fn len(&self) -> usize {
        self.wheel.len()
    }

    //设置定时任务，返回任务句柄
    pub fn set_timeout(&mut self, task: T, timeout: usize) -> usize {
	    let item = Item {
            elem: task,
            time_point: self.wheel.get_time() + (timeout as u64)
        };
        self.wheel.insert(item)
    }

    //推动定时器运行，已到时间的任务，会从定时器中移除，并返回
    pub fn poll(&mut self) -> Vec<T> {
        let mut tasks = Vec::new();

        poll_zero(self, &mut tasks); //运行0毫秒任务
        let mut r = self.wheel.roll();
        poll_task(&mut r, &mut tasks);

        while run_millis() > self.tick_time as u64 + self.wheel.get_time() {
            r = self.wheel.roll();
            poll_task(&mut r, &mut tasks);
            poll_zero(self, &mut tasks); //运行0毫秒任务
        }

        tasks
    }

    //尝试推动定时器运行，返回定时器内部时间与当前时间的差值
    pub fn try_poll(&mut self) -> u64 {
        let now = run_millis();
        let last = self.wheel.get_time();
        let diff = now - last;

        if now > self.tick_time as u64 + last {
            //当前时间没有任何到期的任务，且超过定时轮最小定时间隔，则立即推动一次定时轮
            self.wheel.roll_once();
        }

        diff
    }

    //尝试获取一个已到时间的任务，返回的任务会从定时器中移除
    pub fn try_pop(&mut self) -> Option<T> {
        if let Some(item) = self.wheel.get_one_zero() {
            //有0毫秒任务
            return Some(item.elem);
        } else {
            //没有0毫秒任务，则弹出其它任务
            if let Some(item) = self.wheel.pop() {
                return Some(item.elem);
            }
        }

        None
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

//推动定时器运行超时时长为0的任务，返回已到时间的任务
fn poll_zero<T: Send + 'static>(timer: &mut LocalTimer<T>, tasks: &mut Vec<T>) {
    while timer.wheel.zero_size() > 0 {
        //如果有超时时长为0的任务
        let mut r = timer.wheel.get_zero();
        poll_task(&mut r, tasks);
        r.clear();
        timer.wheel.set_zero_cache(r);
    }
}

//推动定时器运行定时任务，返回已到时间的任务
fn poll_task<T: Send + 'static>(r: &mut Vec<(Item<T>, usize)>, tasks: &mut Vec<T>) {
    let mut j = r.len();
    for _ in 0..r.len(){
        j -= 1;
        let e = r.remove(j);
        tasks.push(e.0.elem);
    }
}
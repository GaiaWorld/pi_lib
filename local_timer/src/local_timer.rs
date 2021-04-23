
use std::{collections::VecDeque, fmt::{Debug, Formatter, Result as FResult}};

use dyn_uint::{UintFactory, ClassFactory, SlabFactory};
use time::run_millis;
use crate::frame_wheel::{FrameWheel};
use crate::item::{TimeoutItem};

/// 任务轮实现的延时任务管理
/// * `T` 延时任务的数据类型
/// * `N1` 一级粒度轮的槽位数 <任务轮分4级粒度>
/// * `N2` 二级粒度轮的槽位数 <任务轮分4级粒度>
/// * `N3` 三级粒度轮的槽位数 <任务轮分4级粒度>
/// * `N4` 四级粒度轮的槽位数 <任务轮分4级粒度>
/// * N1 到 N4 依次减少, 总数不应超过 usize 范围
/// ## Example ##
/// ```rust
/// let mut timer = LocalTimer::<i32, 100, 60, 60, 24>::new(10, run_millis());
/// // 插入一个任务 - 数据为 55, 延时为 1000ms
/// let timeout_handler = timer.insert(Item::new(55 as i32, 1000 as u64));
/// // 尝试获取一个超时任务
/// let one_task_option = timer.pop();
/// // 当获取到的任务为空，可以检查可休眠时间
/// let sleep_time = timer.check_sleep(now);
/// // 可移除一个任务
/// let item_option = timer.try_remove(timeout_handler);
/// ```
pub struct LocalTimer<T, const N1: usize, const N2: usize, const N3: usize, const N4: usize> {
    /// 最小粒度世界线的帧间隔
    pub frame_time: u64,
    /// 启动时间
    pub start_time: u64,
    /// 滚动的累积时间
    pub roll_time: u64,
    /// Index 工厂
    pub index_factory: SlabFactory<usize,()>,
    /// 帧进度的任务轮
    pub frame_wheel: FrameWheel<T, N1, N2, N3, N4>
}

impl<T, const N1: usize, const N2: usize, const N3: usize, const N4: usize> LocalTimer<T, N1, N2, N3, N4>{

    /// Create a wheel to support four rounds.
    /// * `frame_time` 最小时间间隔 - 单位`毫秒`
    /// * `now` 当前绝对时间
    /// * `tip` frame_time 与 N1..N4 之积 的乘积不应超过 uszie 范围
    /// ### Error Example ###
    /// ```rust
    /// // 当 usize 最大 2**32, 则下面的创建不保证运行正确，且可能崩溃
    /// let mut timer = WheelTimer::<i32, 100, 60, 60, 24>::new(1000, run_millis()); // 100*60*60*24*1000 = 8640000000 > 2**32
    /// ```
    pub fn new(frame_time: u64, now: u64) -> Self{
        LocalTimer {
            frame_time,
            start_time: now,
            roll_time: 0,
            index_factory: SlabFactory::new(),
            frame_wheel: FrameWheel::new()
        }
    }

    #[inline]
    /// 检查可休眠时间
    /// * now 当前线程时间 <与创建时设置的 now 属于同一时间进度>
    pub fn check_sleep(&self, now: u64) -> u64 {
        let curr = self.start_time + self.roll_time;
        if curr > now {
            return curr - now;
        }
        else {
            return 0;
        }
    }

    /// 总任务数量
    pub fn len(&self) -> usize {
        self.frame_wheel.len()
    }

    #[inline]
    /// 当前运行时长
    pub fn get_time(&mut self) -> u64{
        self.roll_time
    }

    /// 插入元素
    /// * `data` 目标数据
    /// * `timeout` 延时时间 - 单位`毫秒`
    pub fn insert(&mut self, mut data: T, timeout: u64) -> usize{
        let index = self.index_factory.create(0, 0, ());

        // 相对毫秒延时 转换为 绝对帧位置
        let frame_point = timeout / (self.frame_time as u64) + self.frame_wheel.frame;

        let elem = TimeoutItem::new(data, frame_point);

        self.frame_wheel.insert(elem, index, &mut self.index_factory);

        index
    }

    /// clear all elem
    pub fn clear(&mut self){
        self.frame_wheel.clear();
        self.index_factory.clear();
    }

    #[inline]
    /// 弹出一个超时任务 - 当没有任务弹出，经过检查需要做一次滚动则进行一次滚动 - 当没有任务弹出，外部可以检查睡眠时间
    /// * `now` 当前时间
    pub fn pop(&mut self, now: u64) -> Option<(TimeoutItem<T>, usize)> {
        if let Some(task) = self.frame_wheel.pop() {
            self.index_factory.destroy(task.1);
            return Some(task);
        }
        else {
            // 已经没有超时任务，检查时间是否滚动
            if self.check(now) {
                self.roll_once();
            }
        }
        None
    }

    /// 移除一个任务
    /// * `index` 任务插入时返回的 index
    pub fn try_remove(&mut self, index: usize) -> Option<TimeoutItem<T>>{
        match self.index_factory.try_load(index) {
            Some(i) => {
                if let Some((elem, _)) = self.frame_wheel.delete(self.index_factory.get_class(index).clone(), i, &mut self.index_factory) {
                    self.index_factory.destroy(index);
                    return Some(elem);
                }

                None
            },
            None => None,
        }
    }

    /// Panics if index is out of bounds. 移除一个任务
    /// * `index` 任务插入时返回的 index
    pub fn remove(&mut self, index: usize) -> Option<TimeoutItem<T>> {
        if let Some((elem, _)) = self.frame_wheel.delete(self.index_factory.get_class(index).clone(), self.index_factory.load(index), &mut self.index_factory) {
            self.index_factory.destroy(index);
            return Some(elem);
        }
        None
    }

    #[inline]
    /// 检查是否应该做一次滚动 - 内部使用 
    /// * `now` 当前线程时间 <与创建时设置的 now 属于同一时间进度>
    pub fn check(&self, now: u64) -> bool {
        return self.check_sleep(now) == 0;
    }

    #[inline]
    /// 滚动一次 - 使用者不调用
    pub fn roll_once(&mut self) {
        
        self.roll_time += self.frame_time as u64;
        self.frame_wheel.roll_once(&mut self.index_factory);
    }

    pub fn get_item_timeout(&self, item: &TimeoutItem<T>) -> u64 {
        item.get_frame_point() * self.frame_time
    }
}

impl<T: Debug, const N1: usize, const N2: usize, const N3: usize, const N4: usize> Debug for LocalTimer<T, N1, N2, N3, N4> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
r##"Wheel( 
    index_factory: {:?},
    wheel: {:?},
)"##,
               self.index_factory,
               self.frame_wheel
        )
    }
}
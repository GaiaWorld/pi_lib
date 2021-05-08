//! # 线程不安全的帧轮结构
//! 
//!
//!
use std::{mem::MaybeUninit, usize};

use std::{cmp::{Ord, Ordering}, u64};
use std::mem::{replace, swap};
use std::fmt::{Debug, Formatter, Result as FResult};
// use std::collections::Vec;

use heap::heap::Heap;
use dyn_uint::{UintFactory, ClassFactory};
use arrayvec::ArrayVec;

use crate::item::TimeoutItem;

/// 粒度级别枚举
pub enum LevelEnum {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    LevelN = 5,
}

struct Wheel <T, const N: usize> {
    /// 数据存放数组
    pub arr: ArrayVec<Vec<(TimeoutItem<T>, usize)>, N>,
    /// 跨度
    pub interval: usize,
    /// 粒度
    pub percesion: usize,
    /// 当前槽位置
    index: usize,
    /// 进入的轮的最大帧延迟数目限制
    pub max_limit: usize,
    /// 轮中当前绝对帧进度
    local_frame: usize,
    /// 元素 Index 起点
    pub index_class_start: usize,
    /// 层级累积精度 - 第X层累积精度为 前X层精度之和
    pub accumulat_percesion: usize
}

impl<T, const N: usize> Wheel<T, N> {
    /// 创建帧粒度为 percesion 的轮
    /// 
    pub fn new(percesion: usize, index_class_start: usize, accumulat_percesion: usize) -> Self {
        let mut arr:  ArrayVec<Vec<(TimeoutItem<T>, usize)>, N> = ArrayVec::<Vec<(TimeoutItem<T>, usize)>, N>::new();

        for _ in 0..N {
            arr.push(Vec::default());
        }

        let interval = percesion * N;

        Wheel {
            arr,
            percesion,
            interval,
            index: 0,
            max_limit: 0,
            local_frame: 0,
            index_class_start,
            accumulat_percesion
        }
    }

    /// 队尾弹出
    pub fn pop(&mut self) -> Option<(TimeoutItem<T>, usize)> {
        self.arr[self.index].pop()
    }

    /// 保存一个元素
    pub fn save<F: UintFactory + ClassFactory<usize>>(&mut self, save_index: usize, mut item: TimeoutItem<T>, index: usize, index_class: usize, index_factory: &mut F) {

        // println!("adjust_items - N {:?} - save {:?} - time {:?} - Local {:?}", N, save_index, item.get_frame_point(), self.local_frame);

        index_factory.store(index, self.arr[save_index].len());
        index_factory.set_class(index, index_class);
        self.arr[save_index].push((item, index));
    }

    /// 获取当前活动槽队列
    pub fn get_curr(&mut self) -> &mut Vec<(TimeoutItem<T>, usize)> {
        &mut self.arr[self.index]
    }

    /// 计算目标帧数在当前轮中的跨度
    pub fn compute_span(&self, diff_frame: u64) -> u64 {
        (diff_frame / self.percesion as u64)
    }

    /// 转移目标队列中的元素到当前轮
    pub fn adjust_items<F: UintFactory + ClassFactory<usize>>(&mut self, deque: &mut Vec<(TimeoutItem<T>, usize)>, index_factory: &mut F, curr_frame: u64) {
        loop {
            match deque.pop() {
                Some(temp) => {
                    // 转移时的计算 在转移的目标层级中的跨度计算，需要减去累积粒度值后进行判断 而不是 1
                    let diff_frame = sub1(temp.0.get_frame_point(), curr_frame, self.accumulat_percesion as u64);
                    let span = self.compute_span(diff_frame);

                    let save_index = next_tail(self.index, span as usize, self.arr.len());
                    let index_class = save_index + self.index_class_start;
                    
                    // println!("Curr {:?} diff_frame {:?} span {:?} save_index {:?}", curr_frame, diff_frame, span, save_index);

                    self.save(save_index, temp.0, temp.1, index_class, index_factory);
                },
                None => break,
            }
        }
    }

    /// 清空数据
    pub fn clear(&mut self) {
        for v in self.arr.iter_mut() {
            v.clear();
        }
    }

    /// 调整轮当前的活动槽位序号
    pub fn change_index(&mut self, index: usize) {
        self.index = index;
        
        self.local_frame = self.index * self.percesion;
    }

    /// 获取当前活动槽位序号
    pub fn get_index(&self) -> usize {
        self.index
    }
    
    /// 获取轮中当前活动槽对应帧位置
    pub fn get_local_frame(&self) -> usize {
        self.local_frame
    }
}

pub struct FrameWheel<T, const N1: usize, const N2: usize, const N3: usize, const N4: usize> {
    wheel_level1: Wheel<T, N1>,
    wheel_level2: Wheel<T, N2>,
    wheel_level3: Wheel<T, N3>,
    wheel_level4: Wheel<T, N4>,

    /// levelN 的元素 Index 起点
    pub index_class_start_n: usize,

    /// 堆
    pub heap: Heap<TimeoutItem<T>>,

    /// 总任务计数
    pub len: usize,

    /// 当前帧计数
    pub frame: u64,
}

impl<T, const N1: usize, const N2: usize, const N3: usize, const N4: usize> FrameWheel<T, N1, N2, N3, N4>{

    /// Create a wheel to support four rounds.
    /// 创建 默认 Wheel
    ///
    pub fn new() -> Self{

        let mut percesion = 1;
        let mut accumulat_percesion = 0 + 1;
        let index_class_start_1: usize = 0;
        let mut wheel_level1 = Wheel::<T, N1>::new(percesion, index_class_start_1, accumulat_percesion);

        percesion = wheel_level1.interval;
        accumulat_percesion += percesion;
        let index_class_start_2: usize = index_class_start_1 + wheel_level1.arr.len();
        let mut wheel_level2 = Wheel::<T, N2>::new(percesion, index_class_start_2, accumulat_percesion);

        percesion = wheel_level2.interval;
        accumulat_percesion += percesion;
        let index_class_start_3: usize = index_class_start_2 + wheel_level2.arr.len();
        let mut wheel_level3 = Wheel::<T, N3>::new(percesion, index_class_start_3, accumulat_percesion);

        percesion = wheel_level3.interval;
        accumulat_percesion += percesion;
        let index_class_start_4: usize = index_class_start_3 + wheel_level3.arr.len();
        let mut wheel_level4 = Wheel::<T, N4>::new(percesion, index_class_start_4, accumulat_percesion);

        wheel_level1.max_limit = wheel_level1.interval;
        wheel_level2.max_limit = wheel_level1.interval + wheel_level2.interval;
        wheel_level3.max_limit = wheel_level1.interval + wheel_level2.interval + wheel_level3.interval;
        wheel_level4.max_limit = wheel_level1.interval + wheel_level2.interval + wheel_level3.interval + wheel_level4.interval;

        let index_class_start_n: usize = index_class_start_4 + wheel_level4.arr.len();

        FrameWheel {
            heap: Heap::new(Ordering::Less),
            len: 0,

            frame: 0,

            wheel_level1,
            wheel_level2,
            wheel_level3,
            wheel_level4,

            index_class_start_n,
        }
    }

    ///
    /// 获取当前元素数量 
    ///
    pub fn len(&self) -> usize {
        self.len
    }

    /// 向轮中 插入元素
    /// * @param `item` 目标元素
    /// * @param `index` 目标元素 index
    /// * @param `index_factory` index工厂
    /// * @param `time_info` 时间计算
    pub fn insert< F: UintFactory + ClassFactory<usize>>(&mut self, mut item: TimeoutItem<T>, index: usize, index_factory: &mut F) {
        self.len += 1;

        // 计算时间差
        let mut diff = sub(item.get_frame_point(), self.frame);
        
        if diff < self.wheel_level1.percesion as u64 {
            let span = 0;

            let save_index = self.wheel_level1.index as usize;
            let index_class = save_index + 0;
            
            self.wheel_level1.save(save_index, item, index, index_class, index_factory);
            return;
        }
        
        if diff >= self.wheel_level4.max_limit as u64 {
            let index_class = self.wheel_level1.arr.len() + self.wheel_level2.arr.len() + self.wheel_level3.arr.len() + self.wheel_level4.arr.len();
            index_factory.set_class(index, index_class);
            self.heap.push(item, index, index_factory);
            return;
        }

        diff = diff - 1;

        // println!("Curr {:?}", self.frame);

        if diff < self.wheel_level1.max_limit as u64 {

            save::<T, F, N1>(diff, 0, self.wheel_level1.index_class_start, &mut self.wheel_level1, item, index, index_factory);
        }
        else if diff < self.wheel_level2.max_limit as u64 {
            diff += self.wheel_level1.get_local_frame() as u64;

            save::<T, F, N2>(diff, 1, self.wheel_level2.index_class_start, &mut self.wheel_level2, item, index, index_factory);
        }
        else if diff < self.wheel_level3.max_limit as u64 {
            diff += self.wheel_level1.get_local_frame() as u64;
            diff += self.wheel_level2.get_local_frame() as u64;
            
            save::<T, F, N3>(diff, 1, self.wheel_level3.index_class_start, &mut self.wheel_level3, item, index, index_factory);
        } 
        else if diff < self.wheel_level4.max_limit as u64 {
            diff += self.wheel_level1.get_local_frame() as u64;
            diff += self.wheel_level2.get_local_frame() as u64;
            diff += self.wheel_level3.get_local_frame() as u64;
            
            save::<T, F, N4>(diff, 1, self.wheel_level4.index_class_start, &mut self.wheel_level4, item, index, index_factory);
        }
    }

    /// 轮滚动 - 向后滚动一个最小粒度
    /// 
    /// * @param `index_factory` index工厂
    /// * @param `time_info` 时间计算
    pub fn roll_once< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F){
        self.frame = self.frame + 1;
        let index = next_tail(self.wheel_level1.index, 1, self.wheel_level1.arr.len());
        self.wheel_level1.change_index(index);

        if index == 0 {
            self.adjust_wheel(LevelEnum::Level2, index_factory);

            // println!("Change Index {:?} - Len {:?}", index, self.wheel_level1.arr[0].len());
        }

    }

    /// 调用 roll_once 后， 可调用本方法取出超时任务
    /// * @tip 弹出 None 时，外部可以检查时间决定 roll_once
    /// * @return `Option<(Item<T>, usize)>` 弹出的缓存元素
    pub fn pop(&mut self) -> Option<(TimeoutItem<T>, usize)>{
        let r = self.wheel_level1.pop();

        if r.is_some() {
            self.len -= 1;
        }
        r
    }

    /// Panics if index is out of bounds.
    /// * @param `index_class` 目标元素在Index工厂中的类型ID
    /// * @param `index` 目标元素index
    /// * @param `index_factory` index工厂
    /// * @return `Option<(Item<T>, usize)>` 弹出的元素
    pub fn delete< F: UintFactory + ClassFactory<usize>>(&mut self, index_class: usize, index: usize, index_factory: &mut F) -> Option<(TimeoutItem<T>, usize)> {
        let r = 
            if index_class < self.wheel_level2.index_class_start {
                delete_from_qeque(&mut self.wheel_level1.arr[index_class], index, index_factory)
            }
            else if index_class < self.wheel_level3.index_class_start {
                delete_from_qeque(&mut self.wheel_level2.arr[index_class - self.wheel_level2.index_class_start], index, index_factory)
            }
            else if index_class < self.wheel_level4.index_class_start {
                delete_from_qeque(&mut self.wheel_level3.arr[index_class - self.wheel_level3.index_class_start], index, index_factory)
            }
            else if index_class < self.index_class_start_n {
                delete_from_qeque(&mut self.wheel_level4.arr[index_class - self.wheel_level4.index_class_start], index, index_factory)
            }
            else {
                unsafe { Some(self.heap.delete(index, index_factory)) }
            };

        if r.is_some() {
            self.len -= 1;
        }

        r
    }

    /// clear all elem
    pub fn clear(&mut self){
        self.heap.clear();

        self.wheel_level1.clear();
        self.wheel_level2.clear();
        self.wheel_level3.clear();
        self.wheel_level4.clear();

        self.len = 0;
    }

    fn delete_from_qeque< F: UintFactory + ClassFactory<usize>>(qeque: &mut Vec<(TimeoutItem<T>, usize)>, index: usize, index_factory: &mut F) -> Option<(TimeoutItem<T>, usize)> {
        if let Some(mut r) = qeque.pop() {
            if index < qeque.len(){
                index_factory.store(r.1, index);
                swap(&mut r, &mut qeque[index]);
            }
            return Some(r);
        }
        None
    }

    /// 检查可下降一个粒度级别的元素
    /// * @param `layer` 粒度级别
    /// * @param `index_factory` index工厂
    pub fn adjust_wheel<F: UintFactory + ClassFactory<usize>>(&mut self, level: LevelEnum, index_factory: &mut F) {
    
        match level {
            LevelEnum::Level2 => {
                let index = adjust_wheel::<F, T, N2, N1>(self.frame,&mut self.wheel_level2, &mut self.wheel_level1, index_factory);
                if index == 0 {
                    self.adjust_wheel(LevelEnum::Level3, index_factory);
                }

            },
            LevelEnum::Level3 => {
                let index = adjust_wheel::<F, T, N3, N2>(self.frame,&mut self.wheel_level3, &mut self.wheel_level2, index_factory);
                if index == 0 {
                    self.adjust_wheel(LevelEnum::Level4, index_factory);
                }
                
            },
            LevelEnum::Level4 => {
                let index = adjust_wheel::<F, T, N4, N3>(self.frame, &mut self.wheel_level4, &mut self.wheel_level3, index_factory);
                // 最大尺度的轮每次调整都检查堆的移动 - 而不是等到最大尺度轮移动到 0 才检查
                self.adjust_wheel(LevelEnum::LevelN, index_factory);
            },
            LevelEnum::LevelN => {
                self.adjust_heap_items(index_factory);
                return;
            }
            _ => {
                return;
            }
        };
    }
    
    /// 检查堆中可下降一个粒度级别的元素
    /// * @param `index_factory` index工厂
    fn adjust_heap_items< F: UintFactory + ClassFactory<usize>>(&mut self, index_factory: &mut F){
        // let mut deque = Vec::<(TimeoutItem<T>, usize)>::default();
        while self.heap.len() > 0 {
            let v = unsafe {self.heap.get_unchecked(0)};

            if sub(v.get_frame_point(), self.frame) >= self.wheel_level4.max_limit as u64 {
                break;
            }
            let value = unsafe {
                self.heap.delete(0, index_factory)
            };

            let diff_frame = sub1(value.0.get_frame_point(), self.frame, self.wheel_level4.accumulat_percesion as u64);
            let span = self.wheel_level4.compute_span(diff_frame);

            let save_index = next_tail(self.wheel_level4.index, span as usize, self.wheel_level4.arr.len());
            let index_class = save_index + self.wheel_level4.index_class_start;
            
            self.wheel_level4.save(save_index, value.0, value.1, index_class, index_factory);
        }
    }
}

impl<T: Debug, const N1: usize, const N2: usize, const N3: usize, const N4: usize> Debug for FrameWheel<T, N1, N2, N3, N4> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        let mut arr_str = "[".to_string();
        let mut i = 1;
        
        for v in self.wheel_level1.arr.iter() {
            arr_str += "(";
            arr_str += i.to_string().as_str();
            arr_str += ",";
            arr_str += &format!("{:?}", v);
            arr_str += ")";
            arr_str += ",";
            i += 1;
        }
        
        for v in self.wheel_level2.arr.iter(){
            arr_str += "(";
            arr_str += i.to_string().as_str();
            arr_str += ",";
            arr_str += &format!("{:?}", v);
            arr_str += ")";
            arr_str += ",";
            i += 1;
        }
        
        for v in self.wheel_level3.arr.iter(){
            arr_str += "(";
            arr_str += i.to_string().as_str();
            arr_str += ",";
            arr_str += &format!("{:?}", v);
            arr_str += ")";
            arr_str += ",";
            i += 1;
        }
        
        for v in self.wheel_level4.arr.iter(){
            arr_str += "(";
            arr_str += i.to_string().as_str();
            arr_str += ",";
            arr_str += &format!("{:?}", v);
            arr_str += ")";
            arr_str += ",";
            i += 1;
        }
        arr_str += "]";

        write!(fmt,
r##"Wheel( 
    arr: {:?},
    heap:{:?},
    point1:{:?},
    point2:{:?},
    point3:{:?},
    point4:{:?},
    len:{},
)"##,
               arr_str,
               self.heap,
               self.wheel_level1.index,
               self.wheel_level2.index,
               self.wheel_level3.index,
               self.wheel_level4.index,
               self.len,
        )
    }
}


fn adjust_wheel<F: UintFactory + ClassFactory<usize>, T, const NA: usize, const NB: usize>(frame: u64, curr_wheel: &mut Wheel<T, NA>, smaller_wheel: &mut Wheel<T, NB>, index_factory: &mut F) -> usize {
    let deque = curr_wheel.get_curr();

    smaller_wheel.adjust_items(deque, index_factory, frame);

    let index = next_tail(curr_wheel.index, 1, curr_wheel.arr.len());
    curr_wheel.change_index(index);

    index
}

fn save<T, F: UintFactory + ClassFactory<usize>, const N: usize>(diff: u64, span_diff: usize, index_start: usize, wheel: &mut Wheel<T, N>, mut item: TimeoutItem<T>, index: usize, index_factory: &mut F) {

    let span = wheel.compute_span(diff); // diff / self.wheel_level4.percesion;
    let save_index = next_tail(wheel.index, (span) as usize - span_diff, wheel.arr.len());
    let index_class = save_index + index_start;

    wheel.save(save_index, item, index, index_class, index_factory);
}

fn delete_from_qeque<T, F: UintFactory + ClassFactory<usize>>(qeque: &mut Vec<(TimeoutItem<T>, usize)>, index: usize, index_factory: &mut F) -> Option<(TimeoutItem<T>, usize)> {
    if let Some(mut r) = qeque.pop() {
        if index < qeque.len(){
            index_factory.store(r.1, index);
            swap(&mut r, &mut qeque[index]);
        }
        return Some(r);
    }
    None
}

#[inline]
/// 某个粒度层级的层级指针 在指定的粒度跨度 后的位置
/// * `cur_local` 当前位置
/// * `span` 跨度
/// * `capacity` 当前层级的总跨度
pub fn next_tail(cur_local: usize, span: usize, capacity: usize) -> usize{
    (cur_local + span) % capacity
}

#[inline]
pub fn sub(x: u64, y: u64) -> u64{
    match x > y{
        true => x - y,
        false => 0
    }
}

#[inline]
fn sub1(x: u64, y: u64, percesion: u64) -> u64{
    match x > y + percesion {
        true => x - y - percesion,
        false => 0,
    }
}

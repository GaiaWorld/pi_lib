//! 全部采用常量泛型的多层定时轮
//! 常量泛型依次为 首层轮的槽数量, 后面层的轮内槽的数量, 轮的层数

use slot_deque::{Deque, Slot};
use slotmap::{Key, new_key_type};
use std::marker::PhantomData;
use std::{cmp::Ordering, fmt};

// 定义队列键类型
new_key_type! {
    pub struct TimerKey;
}

/// 定时轮放入方法的结果
pub enum Result<T> {
    Ok(TimerKey),
    Overflow(usize, T),
}
/// 定时轮
pub struct Wheel<T, const N0: usize, const N: usize, const L: usize> {
    /// 首层轮
    layer0: [Deque<TimerKey>; N0],
    /// 多层定时轮
    layers: [[Deque<TimerKey>; N]; L],
    /// 首层轮的当前滚动到的位置
    index: usize,
    /// 每层的当前滚动到的位置
    indexs: [usize; L],
    mark: PhantomData<T>,
}
impl<T, const N0: usize, const N: usize, const L: usize> Default for Wheel<T, N0, N, L> {
    fn default() -> Self {
        Wheel {
            layer0: [Default::default(); N0],
            layers: [[Default::default(); N]; L],
            index: 0,
            indexs: [0; L],
            mark: PhantomData,
        }
    }
}

impl<T: fmt::Debug, const N0: usize, const N: usize, const L: usize> fmt::Debug
    for Wheel<T, N0, N, L>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wheel")
            .field("layer0", &self.layer0)
            .field("layers", &self.layers)
            .field("index", &self.index)
            .field("indexs", &self.indexs)
            .finish()
    }
}

impl<T, const N0: usize, const N: usize, const L: usize> Wheel<T, N0, N, L> {
    /// 获得滚动次数
    pub fn roll_count(&self) -> usize {
        let mut c = self.index;
        for i in 0..L {
            c += self.indexs[i] * (N0 * N.pow(i as u32));
        }
        c
    }
    /// 判断当前槽位是否还有定时任务
    pub fn is_cur_over(&self) -> bool {
        self.layer0[self.index].head().is_null()
    }
    /// 放入一个定时任务，定时时间不能超过定时轮的最大定时时间
    pub fn push(
        &mut self,
        mut timeout: usize,
        el: T,
        slot: &mut Slot<TimerKey, TimeoutItem<T>>,
    ) -> Result<T> {
        if timeout < N0 {
            let j = (self.index + timeout) % N0;
            return Result::Ok(self.layer0[j].push_back(TimeoutItem::new(timeout, el, j), slot));
        }
        let mut fix = self.index;
        for i in 0..L {
            let t = N0 * N.pow(i as u32);
            if timeout < t * N {
                timeout = (timeout + fix + self.indexs[i] * t) % (t * N);
                let j = timeout / t;
                return Result::Ok(
                    self.layers[i][j]
                        .push_back(TimeoutItem::new(timeout, el, N0 + i * N + j), slot),
                );
            }
            fix += self.indexs[i] * t;
        }
        Result::Overflow(timeout + fix, el)
    }
    /// 将指定key的定时任务重新放入轮中， 放入时前面的轮的index应为0
    pub fn push_key<A>(
        &mut self,
        key: TimerKey,
        slot: &mut Slot<TimerKey, TimeoutItem<T>>,
        arg: &mut A,
        func: fn(&mut A, &mut TimeoutItem<T>),
    ) -> TimerKey {
        let node = unsafe { slot.get_unchecked_mut(key) };
        let next = node.next();
        func(arg, &mut node.el);
        if node.el.timeout < N0 {
            node.el.index = node.el.timeout;
            self.layer0[node.el.timeout].push_key_back(key, slot);
            return next;
        }
        for i in 0..L {
            let t = N0 * N.pow(i as u32);
            if node.el.timeout < t * N {
                let j = node.el.timeout / t;
                node.el.index = N0 + i * N + j;
                self.layers[i][j].push_key_back(key, slot);
                return next;
            }
        }
        panic!("timeout overflow")
    }
    /// 获取定时轮能容纳的最大定时时间
    pub fn max_time(&self) -> usize {
        N0 * N.pow(L as u32)
    }
    /// 弹出最小精度的一个定时任务
    /// * @tip 弹出 None 时，外部可以检查时间决定是否roll
    /// * @return `Option<Item<T>>` 弹出的定时元素
    pub fn pop(&mut self, slot: &mut Slot<TimerKey, TimeoutItem<T>>) -> Option<TimeoutItem<T>> {
        self.layer0[self.index].pop_front(slot)
    }
    /// 轮滚动 - 向后滚动一个最小粒度, 可能会造成轮的逐层滚动。返回是否滚动到底了
    pub fn roll(&mut self, slot: &mut Slot<TimerKey, TimeoutItem<T>>) -> bool {
        // 如果首层的轮没有滚到底，则简单+1返回
        if self.index < N0 - 1 {
            self.index += 1;
            return false;
        }
        self.index = 0;
        // 将后一层的轮上滚动一次，
        self.indexs[0] = (self.indexs[0] + 1) % N;
        // 将槽位的所有任务插入到首层轮中
        let mut head = self.layers[0][self.indexs[0]].head();
        if !head.is_null() {
            self.layers[0][self.indexs[0]] = Default::default();
            loop {
                let node = unsafe { slot.get_unchecked_mut(head) };
                let next = node.next();
                // 减去当前位置对应的时间
                node.el.timeout -= N0 * self.indexs[0];
                node.el.index = node.el.timeout;
                self.layer0[node.el.timeout].push_key_back(head, slot);
                if next.is_null() {
                    break;
                }
                head = next;
            }
        }
        if self.indexs[0] > 0 {
            return false;
        }
        // 依次处理每个轮
        for i in 1..L {
            // 将本层的轮上滚动一次，
            self.indexs[i] = (self.indexs[i] + 1) % N;
            // 将槽位的所有任务重新插入轮中
            let mut head = self.layers[i][self.indexs[i]].head();
            if !head.is_null() {
                self.layers[i][self.indexs[i]] = Default::default();
                let mut t = N0 * N.pow(i as u32) * self.indexs[i];
                loop {
                    head = self.push_key(head, slot, &mut t, reduce);
                    if head.is_null() {
                        break;
                    }
                }
            }
            if self.indexs[i] > 0 {
                // 没有滚到底，则返回false
                return false;
            }
        }
        // 最后一个轮滚到底被重置时, 返回true
        true
    }
    /// 获得定时轮中指定层和指定槽位的双端队列
    pub fn get_slot_mut(&mut self, mut index: usize) -> &mut Deque<TimerKey> {
        if index < N0 {
            &mut self.layer0[index]
        } else {
            index -= N0;
            &mut self.layers[index / N][index % N]
        }
    }
}
fn reduce<T>(time: &mut usize, it: &mut TimeoutItem<T>) {
    it.timeout -= *time;
}
/// 定时条目
pub struct TimeoutItem<T> {
    /// 延时 - 外部不可用
    pub timeout: usize,
    /// 数据
    pub el: T,
    /// 所在的轮及槽位
    pub index: usize,
}
impl<T> TimeoutItem<T> {
    pub fn new(timeout: usize, el: T, index: usize) -> Self {
        TimeoutItem { timeout, el, index }
    }
    pub fn timeout(&self) -> usize {
        self.timeout
    }
}
impl<T: fmt::Debug> fmt::Debug for TimeoutItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeoutItem")
            .field("timeout", &self.timeout)
            .field("el", &self.el)
            .field("index", &self.index)
            .finish()
    }
}

impl<T> Eq for TimeoutItem<T> {}

impl<T> Ord for TimeoutItem<T> {
    fn cmp(&self, other: &TimeoutItem<T>) -> Ordering {
        self.timeout.cmp(&other.timeout)
    }
}

impl<T> PartialOrd for TimeoutItem<T> {
    fn partial_cmp(&self, other: &TimeoutItem<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for TimeoutItem<T> {
    fn eq(&self, other: &TimeoutItem<T>) -> bool {
        self.timeout == other.timeout
    }
}

#[test]
fn test() {
    use crate::*;
    let mut slot: Slot<TimerKey, TimeoutItem<usize>> = Default::default();
    let vec = vec![
        0, 1, 10, 6, 4, 4, 4, 5, 3, 7, 2, 15, 18, 21, 26, 31, 39, 41, 8, 99, 90,
    ]; //
       //let vec = vec![1,10,6];
    let mut wheel: Wheel<usize, 10, 4, 2> = Wheel::default();
    println!("max_time:{:?}", wheel.max_time());
    for i in vec.clone() {
        wheel.push(i, i, &mut slot);
    }
    println!("{:?}", wheel);
    let mut sorted = vec.clone();
    sorted.sort();
    sorted.reverse();
    let mut c = 0;
    while sorted.len() > 0 {
        if let Some(r) = wheel.pop(&mut slot) {
            println!("r:{:?}, c:{}", r, c);
            assert_eq!(r.el, sorted.pop().unwrap());
            assert_eq!(r.el, c);
            if r.el == 18 {
                sorted.push(32);
                sorted.push(47);
                sorted.sort();
                sorted.reverse();
                wheel.push(14, 32, &mut slot);
                wheel.push(29, 47, &mut slot);
                println!("---{:?}", wheel);
            }
        } else {
            let r = wheel.roll(&mut slot);
            c += 1;
            if c % 10 == 0 {
                if c < 40 {
                    sorted.push(c * 2 + 2);
                    sorted.sort();
                    sorted.reverse();
                    wheel.push(c + 2, c * 2 + 2, &mut slot);
                }
                assert_eq!(c, wheel.roll_count());
                println!("{:?} roll_over:{}", wheel, r);
            }
        }
    }
}

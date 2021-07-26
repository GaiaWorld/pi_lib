//! 全部采用常量泛型的多层定时轮
//! 常量泛型依次为 首层轮的槽数量, 后面层的轮内槽的数量, 轮的层数

#![feature(maybe_uninit_ref)]

use std::{cmp::Ordering, fmt, mem::MaybeUninit};

pub struct Wheel<T, const N0: usize, const N: usize, const L: usize> {
    /// 首层轮
    layer0: MaybeUninit<[Vec<TimeoutItem<T>>; N0]>,
    /// 多层定时轮
    layers: MaybeUninit<[[Vec<TimeoutItem<T>>; N]; L]>,
    /// 首层轮的当前滚动到的位置
    index: usize,
    /// 每层的当前滚动到的位置
    indexs: [usize; L],
}
impl<T, const N0: usize, const N: usize, const L: usize> Default for Wheel<T, N0, N, L> {
    fn default() -> Self {
        Wheel {
            layer0: unsafe { MaybeUninit::zeroed().assume_init() },
            layers: unsafe { MaybeUninit::zeroed().assume_init() },
            index: 0,
            indexs: [0; L],
        }
    }
}
impl<T, const N0: usize, const N: usize, const L: usize> Drop for Wheel<T, N0, N, L> {
    fn drop(&mut self) {
        // MaybeUninit inhibits vec's drop
        let layer0 = unsafe { self.layer0.assume_init_mut() };
        for i in 0..N0 {
            layer0[i].clear()
        }
        let layers = unsafe { self.layers.assume_init_mut() };
        for i in 0..L {
            for j in 0..N {
                layers[i][j].clear()
            }
        }
    }
}
impl<T: fmt::Debug, const N0: usize, const N: usize, const L: usize> fmt::Debug
    for Wheel<T, N0, N, L>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wheel")
            .field("layer0", unsafe { self.layer0.assume_init_ref() })
            .field("layers", unsafe { self.layers.assume_init_ref() })
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
        unsafe { self.layer0.assume_init_ref()[self.index].is_empty() }
    }
    /// 放入一个定时任务，定时时间如果超过定时轮的最大定时时间，则返回修正时间后的该定时任务
    pub fn push(&mut self, mut it: TimeoutItem<T>) -> Option<TimeoutItem<T>> {
        if it.timeout < N0 {
            let layer0 = unsafe { self.layer0.assume_init_mut() };
            layer0[(it.timeout + self.index) % N0].push(it);
            return None;
        }
        let mut fix = self.index;
        let layers = unsafe { self.layers.assume_init_mut() };
        for i in 0..L {
            let t = N0 * N.pow(i as u32);
            if it.timeout < t * N {
                it.timeout = (it.timeout + fix + self.indexs[i] * t) % (t * N);
                layers[i][it.timeout / t].push(it);
                return None;
            }
            fix += self.indexs[i] * t;
        }
        it.timeout += fix;
        Some(it)
    }
    /// 获取定时轮能容纳的最大定时时间
    pub fn max_time(&self) -> usize {
        N0 * N.pow(L as u32)
    }
    /// 弹出最小精度的一个定时任务
    /// * @tip 弹出 None 时，外部可以检查时间决定是否roll
    /// * @return `Option<Item<T>>` 弹出的定时元素
    pub fn pop(&mut self) -> Option<TimeoutItem<T>> {
        unsafe { self.layer0.assume_init_mut()[self.index].pop() }
    }
    /// 轮滚动 - 向后滚动一个最小粒度, 可能会造成轮的逐层滚动。返回是否滚动到底了
    pub fn roll(&mut self) -> bool {
        // 如果首层的轮没有滚到底，则简单+1返回
        if self.index < N0 - 1 {
            self.index += 1;
            return false;
        }
        self.index = 0;
        // 将后一层的轮上滚动一次，
        self.indexs[0] = (self.indexs[0] + 1) % N;
        // 将槽位的所有任务插入到首层轮中
        while let Some(mut it) = unsafe { self.layers.assume_init_mut() }[0][self.indexs[0]].pop() {
            // 减去当前位置对应的时间
            it.timeout -= N0 * self.indexs[0];
            unsafe { self.layer0.assume_init_mut()[it.timeout].push(it) };
        }
        if self.indexs[0] > 0 {
            return false;
        }
        // 依次处理每个轮
        for i in 1..L {
            // 将本层的轮上滚动一次，
            self.indexs[i] = (self.indexs[i] + 1) % N;
            // 将槽位的所有任务重新插入轮中
            while let Some(mut it) =
                unsafe { self.layers.assume_init_mut() }[i][self.indexs[i]].pop()
            {
                // 减去当前位置对应的时间
                it.timeout -= N0 * N.pow(i as u32) * self.indexs[i];
                self.push(it);
            }
            if self.indexs[i] > 0 {
                // 没有滚到底，则返回false
                return false;
            }
        }
        // 最后一个轮滚到底被重置时, 返回true
        true
    }
}

/// 定时条目
pub struct TimeoutItem<T> {
    /// 延时 - 外部不可用
    pub timeout: usize,
    /// 数据
    pub el: T,
}
impl<T> TimeoutItem<T> {
    pub fn new(timeout: usize, el: T) -> Self {
        TimeoutItem { timeout, el }
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
    let vec = vec![
        0, 1, 10, 6, 4, 4, 4, 5, 3, 7, 2, 15, 18, 21, 26, 31, 39, 41, 8, 79, 89,
    ]; //
       //let vec = vec![1,10,6];
    let mut wheel: Wheel<usize, 10, 3, 2> = Wheel::default();
    println!("max_time:{:?}", wheel.max_time());
    for i in vec.clone() {
        wheel.push(TimeoutItem::new(i, i));
    }
    println!("{:?}", wheel);
    let mut sorted = vec.clone();
    sorted.sort();
    sorted.reverse();
    let mut c = 0;
    while sorted.len() > 0 {
        if let Some(r) = wheel.pop() {
            println!("r:{:?}, c:{}", r, c);
            assert_eq!(r.el, sorted.pop().unwrap());
            assert_eq!(r.el, c);
            if r.el == 18 {
                sorted.push(32);
                sorted.push(47);
                sorted.sort();
                sorted.reverse();
                wheel.push(TimeoutItem::new(14, 32));
                wheel.push(TimeoutItem::new(29, 47));
                println!("---{:?}", wheel);
            }
        } else {
            let r = wheel.roll();
            c += 1;
            if c % 10 == 0 {
                if c < 60 {
                    sorted.push(c * 2 + 2);
                    sorted.sort();
                    sorted.reverse();
                    wheel.push(TimeoutItem::new(c + 2, c * 2 + 2));
                }
                //assert_eq!(c, wheel.roll_count());
            }
            println!("roll:: {:?} {:?}", r, wheel.roll_count());
        }
    }
}

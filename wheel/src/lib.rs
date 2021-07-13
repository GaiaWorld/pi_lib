//! 全部采用常量泛型的多层定时轮
//! 常量泛型依次为 精度, 轮内槽的数量, 轮的层数

#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_array_assume_init)]

use std::{cmp::Ordering, fmt, mem::MaybeUninit};

pub struct Wheel<T, const P: usize, const N: usize, const L: usize> {
    /// 多层定时轮
    arr: MaybeUninit<[[Vec<TimeoutItem<T>>; N]; L]>,
    /// 每层的当前滚动到的位置
    indexs: [usize; L],
}
impl<T: Sized, const P: usize, const N: usize, const L: usize> Default for Wheel<T, P, N, L> {
    fn default() -> Self {
        Wheel {
            arr: unsafe { MaybeUninit::zeroed().assume_init() },
            indexs: [0; L],
        }
    }
}
impl<T: Sized, const P: usize, const N: usize, const L: usize> Drop for Wheel<T, P, N, L> {
    fn drop(&mut self) {
        let arr = unsafe { self.arr.assume_init_mut() };
        // MaybeUninit inhibits vec's drop
        for i in 0..L {
            for j in 0..N {
                arr[i][j].clear()
            }
        }
    }
}
impl<T: fmt::Debug, const P: usize, const N: usize, const L: usize> fmt::Debug
    for Wheel<T, P, N, L>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wheel")
            .field("arr", unsafe { self.arr.assume_init_ref() })
            .field("indexs", &self.indexs)
            .finish()
    }
}

impl<T, const P: usize, const N: usize, const L: usize> Wheel<T, P, N, L> {
    /// 放入一个定时任务，返回所在轮的层和槽位, 及槽位内向量中定时任务所在的位置，
    /// 定时时间不能超过定时轮的最大定时时间
    pub fn push(&mut self, timeout: usize, el: T) -> (usize, usize, usize) {
        let arr = unsafe { self.arr.assume_init_mut() };
        if timeout < P {
            arr[0][self.indexs[0]].push(TimeoutItem::new(timeout, el));
            return (0, self.indexs[0], arr[0][self.indexs[0]].len() - 1);
        }
        for i in 0..L {
            if timeout < P * (N + 1).pow(i as u32 + 1) {
                let j = (self.indexs[i] + timeout / (P * (N + 1).pow(i as u32)) - 1) % (N + 1);
                arr[i][j].push(TimeoutItem::new(timeout, el));
                return (i, j, arr[i][j].len() - 1);
            }
        }
        panic!("timeout overflow")
    }
    /// 获取定时轮能容纳的最大定时时间
    pub fn max_time(&self) -> usize {
        P * (N + 1).pow(L as u32)
    }
    /// 弹出最小精度的一个定时任务
    /// * @tip 弹出 None 时，外部可以检查时间决定是否roll
    /// * @return `Option<Item<T>>` 弹出的定时元素
    pub fn pop(&mut self) -> Option<TimeoutItem<T>> {
        unsafe { self.arr.assume_init_mut()[0][self.indexs[0]].pop() }
    }
    /// 轮滚动 - 向后滚动一个最小粒度, 可能会造成轮的逐层滚动。返回是否滚动到底了
    pub fn roll<A>(
        &mut self,
        arg: &mut A,
        func: fn(&mut A, &mut [TimeoutItem<T>], usize, usize, usize),
    ) -> bool {
        // 依次处理每个轮
        for i in 0..L - 1 {
            // 如果本层的轮没有滚到底，则简单+1返回
            if self.indexs[i] < N - 1 {
                self.indexs[i] += 1;
                return false;
            }
            self.indexs[i] = 0;
            let i1 = i + 1;
            // 将后一层的轮上的当前槽位的所有任务取余后插入到前面的轮中，然后继续处理后续的轮
            let index = self.indexs[i1];
            while let Some(mut it) = unsafe { self.arr.assume_init_mut() }[i1][index].pop() {
                it.timeout %= P * (N + 1).pow(i1 as u32);
                let r = self.push(it.timeout, it.el);
                func(
                    arg,
                    unsafe { self.arr.assume_init_mut() }[r.0][r.1].as_mut_slice(),
                    r.0,
                    r.1,
                    r.2,
                )
            }
        }
        // 处理最后一个轮
        if self.indexs[L - 1] < N - 1 {
            self.indexs[L - 1] += 1;
            false
        } else {
            self.indexs[L - 1] = 0;
            true
        }
    }
    /// 获得定时轮中指定层和指定槽位的向量数组
    pub fn get_slot_mut(&mut self, layer: usize, slot: usize) -> &mut [TimeoutItem<T>] {
        unsafe { self.arr.assume_init_mut()[layer][slot].as_mut_slice() }
    }
    /// 移除定时轮中指定层和指定槽位的向量数组中指定位置的定时任务
    pub fn remove(&mut self, layer: usize, slot: usize, index: usize) -> TimeoutItem<T> {
        let arr = unsafe { self.arr.assume_init_mut() };
        arr[layer][slot].swap_remove(index)
    }
}

/// 定时条目
pub struct TimeoutItem<T> {
    /// 延时 - 外部不可用
    timeout: usize,
    /// 数据
    pub el: T,
}
impl<T> TimeoutItem<T> {
    pub fn new(timeout: usize, el: T) -> Self {
        TimeoutItem {
            timeout,
            el,
        }
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

pub fn empty<T, A>(
    _arg: &mut A,
    _arr: &mut [TimeoutItem<T>],
    _layer: usize,
    _slot: usize,
    _index: usize,
) {
}

#[test]
fn test() {
    use crate::*;
    let vec = vec![1, 10, 6, 5, 9, 4, 4, 4, 3, 7, 99, 90, 2, 15, 8];
    //let vec = vec![1,10,6];
    let mut wheel: Wheel<usize, 1, 9, 2> = Wheel::default();
    for i in vec.clone() {
        wheel.push(i, i);
    }
    println!("{:?}", wheel);
    let mut sorted = vec.clone();
    sorted.sort();
    sorted.reverse();

    while sorted.len() > 0 {
        if let Some(r) = wheel.pop() {
            println!("r:{:?}", r);
            assert_eq!(r.el, sorted.pop().unwrap());
        } else {
            let r = wheel.roll(&mut (), empty);
            println!("{:?}", wheel);
        }
    }
}

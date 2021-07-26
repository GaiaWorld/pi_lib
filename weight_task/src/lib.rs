//! 任务池
//! 可以向任务池中放入不同权重的任务，任务池提供弹出功能，任务池大概率会弹出权重高的任务。
//! 任务池支持的任务可以大致分为三类：
//!     1. 串行任务：插入串行任务需要先创建队列，放入到同一个队列的任务，会按顺序弹出。
//!         创建队列会返回队列key，可以通过该key获取队列引用。
//!         可以获得队列状态-是否被锁定，可以从队列头或尾放入任务，也可弹出任务。
//!         可设置队列权重。调整队列权重立刻在下一次弹出队列时生效。
//!         队列可以设置为弹出任务后自动锁定，直到外部将队列解开锁定，锁定的队列不会在弹出任务。
//!     2. 并行任务：在任务池中，如果不是队列任务，那一定是一个并行任务。
//!         并行任务与串行任务的区别是，并行任务不需要排队，并行任务的权重越高，弹出的概率越大。
//!     3. 定时任务，该任务先被存在定时器中，超时后，才能有机会被弹出。
//!         定时任务分为可撤销和不可撤销两类，可撤销的定时任务放入时会返回唯一key，通过该key可撤销该任务。
//!

use std::{collections::VecDeque, fmt, num::NonZeroU32};

use ext_heap::empty;
use rand_core::{RngCore, SeedableRng};
use slotmap::{new_key_type, Key, SlotMap};
use weight::{WeightHeap, WeightItem};
use wy_rng::WyRng;

// 定义队列键类型
new_key_type! {
    pub struct DequeKey;
}

/// 队列权重类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeightType {
    // 标准权重
    Normal(NonZeroU32),
    // 单位权重，总权重为队列长度*单位权重
    Unit(NonZeroU32),
}
/// 任务队列
pub struct Deque<T, D> {
    /// 队列
    pub deque: VecDeque<T>,
    /// 队列权重类型
    weight_type: WeightType,
    /// 所在的权重堆的位置
    weight_index: usize,
    /// 锁定状态, None表示不自动锁定， true表示锁定，false表示无锁定
    lock_state: Option<bool>,
    /// 旧的长度
    old_deque_len: usize,
    /// 关联的数据
    pub data: D,
}
impl<T, D> Deque<T, D> {
    pub fn new(weight_type: WeightType, data: D) -> Self {
        Deque {
            deque: Default::default(),
            weight_type,
            weight_index: usize::MAX,
            lock_state: None,
            old_deque_len: 0,
            data,
        }
    }
    /// 获得权重类型
    pub fn weight_type(&self) -> WeightType {
        self.weight_type
    }
    /// 获得队列锁定状态
    pub fn lock_state(&self) -> Option<bool> {
        self.lock_state
    }
    /// 获得队列状态
    pub fn state(&self) -> DequeState {
        DequeState {
            new_deque_len: self.deque.len(),
            weight_type: self.weight_type,
            weight_index: self.weight_index,
            lock_state: self.lock_state,
            old_deque_len: self.old_deque_len,
        }
    }
}
impl<T: fmt::Debug, D: fmt::Debug> fmt::Debug for Deque<T, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Deque")
            .field("deque", &self.deque)
            .field("weight_type", &self.weight_type)
            .field("weight_index", &self.weight_index)
            .field("lock_state", &self.lock_state)
            .field("data", &self.data)
            .finish()
    }
}
/// 队列状态，修复队列在权重堆上时需要
pub struct DequeState {
    /// 新的队列长度
    new_deque_len: usize,
    /// 队列权重类型
    weight_type: WeightType,
    /// 所在的权重堆的位置
    weight_index: usize,
    /// 锁定状态, None表示不自动锁定， true表示锁定，false表示无锁定
    lock_state: Option<bool>,
    /// 旧的队列长度
    old_deque_len: usize,
}

/// 任务池
pub struct TaskPool<T, D, const N0: usize, const N: usize, const L: usize> {
    slot: SlotMap<DequeKey, Deque<T, D>>,
    // 串行任务队列池
    sync_pool: WeightHeap<DequeKey>,
    // 并行任务池
    async_pool: WeightHeap<T>,
    // 不可撤销定时器
    timer: timer::Timer<T, N0, N, L>,
    // 可撤销定时器
    cancel_timer: cancel_timer::Timer<T, N0, N, L>,
    // 两个定时器的权重
    timer_weight: usize,
    // 随机数
    rng: WyRng,
    // 串行任务的添加数量
    sync_add_count: usize,
    // 串行任务的移除数量
    sync_remove_count: usize,
    // 并行任务的添加数量
    async_add_count: usize,
    // 并行任务的移除数量
    async_remove_count: usize,
}

impl<T: fmt::Debug, D, const N0: usize, const N: usize, const L: usize> fmt::Debug
    for TaskPool<T, D, N0, N, L>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TaskPool")
            .field("sync_pool", &self.sync_pool)
            .field("async_pool", &self.async_pool)
            .field("timer", &self.timer)
            .field("cancel_timer", &self.cancel_timer)
            .field("timer_weight", &self.timer_weight)
            .field("rng", &self.rng)
            .field("sync_add_count", &self.sync_add_count)
            .field("sync_remove_count", &self.sync_remove_count)
            .field("async_add_count", &self.async_add_count)
            .field("async_remove_count", &self.async_remove_count)
            .finish()
    }
}

impl<T, D, const N0: usize, const N: usize, const L: usize> Default for TaskPool<T, D, N0, N, L> {
    fn default() -> Self {
        TaskPool {
            slot: Default::default(),
            sync_pool: Default::default(),
            async_pool: Default::default(),
            timer: Default::default(),
            cancel_timer: Default::default(),
            timer_weight: 65535,
            rng: Default::default(),
            sync_add_count: 0,
            sync_remove_count: 0,
            async_add_count: 0,
            async_remove_count: 0,
        }
    }
}
impl<T, D, const N0: usize, const N: usize, const L: usize> TaskPool<T, D, N0, N, L> {
    /// 获得所有任务的添加数量
    pub fn add_count(&self) -> usize {
        self.sync_add_count
            + self.async_add_count
            + self.timer.add_count()
            + self.cancel_timer.add_count()
    }
    /// 获得所有任务的移除数量
    pub fn remove_count(&self) -> usize {
        self.sync_remove_count
            + self.async_remove_count
            + self.timer.remove_count()
            + self.cancel_timer.remove_count()
    }
    /// 获得串行任务的添加数量
    pub fn sync_add_count(&self) -> usize {
        self.sync_add_count
    }
    /// 获得串行任务的移除数量
    pub fn sync_remove_count(&self) -> usize {
        self.sync_remove_count
    }
    /// 获得并行任务的添加数量
    pub fn async_add_count(&self) -> usize {
        self.async_add_count
    }
    /// 获得并行任务的移除数量
    pub fn async_remove_count(&self) -> usize {
        self.async_remove_count
    }
    /// 重置随机种子
    pub fn reset_rng(&mut self, seed: u64) {
        self.rng = WyRng::seed_from_u64(seed);
    }
    /// 获得串行任务队列的数量
    pub fn deque_len(&self) -> usize {
        self.slot.len()
    }
    /// 将指定的串行任务队列加入任务池，返回队列key
    pub fn push_deque(&mut self, deque: Deque<T, D>) -> DequeKey {
        let w = match deque.weight_type {
            WeightType::Normal(w) => w.get() as usize,
            WeightType::Unit(w) => (w.get() as usize) * deque.deque.len(),
        };
        let lock_state = deque.lock_state;
        let key = self.slot.insert(deque);
        if Some(true) != lock_state && w > 0 {
            // 如果队列没有锁定， 并且有任务， 则放入权重池中
            self.sync_pool
                .push_weight(w, key, &mut self.slot, set_index);
        }
        key
    }
    /// 重设置队列的权重
    pub fn reset_deque_weight(&mut self, key: DequeKey, weight_type: WeightType) -> bool {
        if let Some(it) = self.slot.get_mut(key) {
            if it.weight_type == weight_type {
                return true;
            }
            it.weight_type = weight_type;
            // 当前队列没有任务
            if it.deque.len() == 0 {
                return true;
            }
            // 当前队列被锁定
            if Some(true) == it.lock_state {
                return true;
            }
            let w = match it.weight_type {
                WeightType::Normal(w) => w.get() as usize,
                WeightType::Unit(w) => (w.get() as usize) * it.deque.len(),
            };
            // 修正权重堆中的权重
            self.sync_pool
                .modify_weight(it.weight_index, w, &mut self.slot, set_index);
            return true;
        }
        false
    }
    /// 获得任务队列的只读引用
    pub fn get_deque(&self, key: DequeKey) -> Option<&Deque<T, D>> {
        self.slot.get(key)
    }
    /// 获得任务队列的可写引用
    pub fn get_deque_mut(&mut self, key: DequeKey) -> Option<&mut Deque<T, D>> {
        let mut r = self.slot.get_mut(key);
        if let Some(it) = &mut r {
            it.old_deque_len = it.deque.len();
        }
        r
    }
    /// 增删任务队列的任务后，应修复队列状态
    pub fn repair_deque_state(&mut self, key: DequeKey, state: DequeState) {
        // 当前队列被锁定
        if Some(true) == state.lock_state {
            return;
        }
        if state.new_deque_len > 0 {
            if state.old_deque_len > 0 {
                if state.new_deque_len == state.old_deque_len {
                    return;
                }
                match state.weight_type {
                    WeightType::Unit(w) => {
                        // 单位权重情况下， 任务数量变化要修正权重堆中的权重
                        let w = (w.get() as usize) * state.new_deque_len;
                        self.sync_pool.modify_weight(
                            state.weight_index,
                            w,
                            &mut self.slot,
                            set_index,
                        );
                    }
                    _ => (),
                };
                if state.new_deque_len > state.old_deque_len {
                    self.sync_add_count += state.new_deque_len - state.old_deque_len;
                } else {
                    self.sync_remove_count += state.old_deque_len - state.new_deque_len;
                }
            } else {
                let w = match state.weight_type {
                    WeightType::Normal(w) => w.get() as usize,
                    WeightType::Unit(w) => (w.get() as usize) * state.new_deque_len,
                };
                // 插入到权重堆
                self.sync_pool
                    .push_weight(w, key, &mut self.slot, set_index);
                self.sync_add_count += state.new_deque_len;
            }
        } else if state.old_deque_len > 0 {
            // 移除出权重堆
            self.sync_pool
                .remove_index(state.weight_index, &mut self.slot, set_index);
            self.sync_remove_count += state.old_deque_len;
        }
    }
    /// 释放队列的锁，成功释放，则返回true， 否则返回false
    pub fn deque_unlock(&mut self, key: DequeKey) -> bool {
        if let Some(it) = self.slot.get_mut(key) {
            if Some(true) == it.lock_state {
                // 解锁
                it.lock_state = Some(false);
                if it.deque.len() > 0 {
                    // 队列中有任务，则将队列放入到权重堆
                    let w = match it.weight_type {
                        WeightType::Normal(w) => w.get() as usize,
                        WeightType::Unit(w) => (w.get() as usize) * it.deque.len(),
                    };
                    self.sync_pool
                        .push_weight(w, key, &mut self.slot, set_index);
                }
            }
            true
        } else {
            false
        }
    }

    /// 删除一个任务队列，如果删除成功，返回true， 否则返回false
    pub fn remove_deque(&mut self, key: DequeKey) -> bool {
        if let Some(it) = self.slot.remove(key) {
            // 当前队列没有任务
            if it.deque.len() == 0 {
                return true;
            }
            self.sync_remove_count += it.deque.len();
            // 当前队列被锁定
            if Some(true) == it.lock_state {
                return true;
            }
            self.sync_pool
                .remove_index(it.weight_index, &mut self.slot, set_index);
            true
        } else {
            false
        }
    }
    /// 插入一个指定任务权重的并行任务
    pub fn push_async(&mut self, task: T, weight: u32) {
        self.async_pool
            .push_weight(weight as usize, task, &mut (), empty);
        self.async_add_count += 1;
    }
    /// 获得不可删除的定时器
    pub fn get_timer(&self) -> &timer::Timer<T, N0, N, L> {
        &self.timer
    }
    /// 获得可删除的定时器
    pub fn get_cancel_timer(&self) -> &cancel_timer::Timer<T, N0, N, L> {
        &self.cancel_timer
    }
    /// 获得不可删除的定时器
    pub fn get_timer_mut(&mut self) -> &mut timer::Timer<T, N0, N, L> {
        &mut self.timer
    }
    /// 获得可删除的定时器
    pub fn get_cancel_timer_mut(&mut self) -> &mut cancel_timer::Timer<T, N0, N, L> {
        &mut self.cancel_timer
    }
    /// 获得定时器的权重
    pub fn get_timer_weight(&self) -> usize {
        self.timer_weight
    }
    /// 设置定时器的权重
    pub fn set_timer_weight(&mut self, weight: usize) {
        self.timer_weight = weight;
    }
    /// 弹出一个任务，如果任务存在，返回任务及所在队列, 否则返回None
    /// 如果该任务是一个串行队列任务，并且为自动加锁状态，则会对该任务所在的队列加锁，此后，该队列的任务无法弹出，
    /// 直到外部调用free_deque方法解锁该队列，该队列的任务在后续的弹出过程中才有机会被弹出
    pub fn pop(&mut self, now: u64) -> (Option<T>, DequeKey) {
        let sync_w = if let Some(r) = self.sync_pool.peek() {
            r.amount() as u64
        } else {
            0
        };
        let async_w = if let Some(r) = self.async_pool.peek() {
            r.amount() as u64
        } else {
            0
        };
        let timer_w = if self.timer.is_ok(now) {
            self.timer_weight as u64
        } else {
            0
        };
        let cancel_timer_w = if self.cancel_timer.is_ok(now) {
            self.timer_weight as u64
        } else {
            0
        };
        let amount = sync_w + async_w + timer_w + cancel_timer_w;
        if amount == 0 {
            return (None, DequeKey::null());
        }
        let mut w = self.rng.next_u64() % amount;
        if w < cancel_timer_w {
            return (self.cancel_timer.pop(now), DequeKey::null());
        } else {
            w -= cancel_timer_w;
        }
        if w < timer_w {
            return (self.timer.pop(now), DequeKey::null());
        } else {
            w -= timer_w;
        }
        if w < async_w {
            if let Some(r) = self.async_pool.pop(&mut (), empty) {
                self.async_remove_count += 1;
                return (Some(r.el), DequeKey::null());
            }
        } else {
            w -= async_w;
        }
        // 从串行任务队列的权重堆中根据权重查找队列
        let index = self.sync_pool.find_weight(w as usize);
        let key = self.sync_pool.as_slice()[index].el;
        let it = &mut self.slot[key];
        // 弹出任务
        let r = it.deque.pop_front();
        self.sync_remove_count += 1;
        if it.lock_state.is_some() {
            // 如果队列为自动锁定状态，则改为锁定， 并移出权重堆
            it.lock_state = Some(true);
            self.sync_pool
                .remove_index(index, &mut self.slot, set_index);
        } else if it.deque.is_empty() {
            // 如果队列为空，则移出权重堆
            self.sync_pool
                .remove_index(index, &mut self.slot, set_index);
        } else if let WeightType::Unit(uw) = it.weight_type {
            // 如果队列权重类型为单位权重，则调整队列在权重堆上的权重
            let ww = (uw.get() as usize) * it.deque.len();
            self.sync_pool
                .modify_weight(index, ww, &mut self.slot, set_index);
        }
        (r, key)
    }
    /// 判断当前时间内是否还有可以弹出的任务
    pub fn is_ok(&mut self, now: u64) -> bool {
        self.sync_pool.len() > 0
            || self.async_pool.len() > 0
            || self.timer.is_ok(now)
            || self.cancel_timer.is_ok(now)
    }
}

fn set_index<T, D>(
    slot: &mut SlotMap<DequeKey, Deque<T, D>>,
    arr: &mut [WeightItem<DequeKey>],
    loc: usize,
) {
    let i = &arr[loc];
    unsafe {
        slot.get_unchecked_mut(i.el).weight_index = loc;
    }
}

// 测试定时器得延时情况
#[cfg(test)]
mod test_mod {
    //extern crate rand_core;

    use std::{
        thread,
        time::{Duration, Instant},
    };

    //use self::rand_core::{RngCore, SeedableRng};
    use crate::*;

    #[test]
    fn test() {
        let mut pool: TaskPool<(u64, u64), u64, 128, 16, 1> = Default::default();
        let arr = [
            pool.push_deque(Deque::new(
                WeightType::Unit(unsafe { NonZeroU32::new_unchecked(1000) }),
                1,
            )),
            pool.push_deque(Deque::new(
                WeightType::Unit(unsafe { NonZeroU32::new_unchecked(200) }),
                2,
            )),
            pool.push_deque(Deque::new(
                WeightType::Normal(unsafe { NonZeroU32::new_unchecked(20000) }),
                3,
            )),
        ];
        let mut rng = WyRng::seed_from_u64(22222);
        let start = Instant::now();
        for i in 1..100000 {
            let t = (rng.next_u32() % 16100) as u64;
            let now = Instant::now();
            let tt = now.duration_since(start).as_millis() as u64;
            if i < 100 {
                if t % 4 == 0 {
                    println!("push: timeout:{} r:{:?}", t, (i, t));
                    pool.get_timer_mut().push(t as usize, (i, t));
                } else if t % 4 == 1 {
                    println!("push: cancel:{} r:{:?}", t, (i, t));
                    pool.get_cancel_timer_mut().push(t as usize, (i, t));
                } else if t % 4 == 2 {
                    println!("push: async:{} r:{:?}", t, (i, t));
                    pool.push_async((i, t), t as u32);
                    continue;
                } else if t % 4 == 3 {
                    println!("push: sync:{} r:{:?}", t, (i, t));
                    let k = arr[(t % 3) as usize];
                    let d = pool.get_deque_mut(k).unwrap();
                    d.deque.push_back((i, t));
                    let state = d.state();
                    pool.repair_deque_state(k, state);
                    continue;
                }
            }
            while let (Some(it), dk) = pool.pop(tt) {
                println!("ppp:{:?}, now:{}, dk:{:?}", it, tt, dk);
            }
            if i > 100 && pool.add_count() == pool.remove_count() {
                //println!("vec:{:?}", vec);
                println!("return: add_count:{:?}", pool.add_count());
                return;
            }
            thread::sleep(Duration::from_millis(1 as u64));
        }
    }
}

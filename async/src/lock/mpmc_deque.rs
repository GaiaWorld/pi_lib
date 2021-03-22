//! # 多生产者多消费者的双端队列
//!

use std::sync::Arc;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU8, Ordering};

use super::spin;

/*
* 锁状态
*/
const UNLOCK_EMPTY: u8 = 0;     //无锁无任务
const UNLOCK_NON_EMPTY: u8 = 1; //无锁有任务
const LOCKED: u8 = 2;           //有锁

/*
* 内部自旋锁双端队列
*/
struct InnerDeque<T: 'static> {
    tail_status:    AtomicU8,                         //队列尾锁状态
    tail:           UnsafeCell<Option<Vec<T>>>,       //队列尾
    head_status:    AtomicU8,                         //队列头锁状态
    head:           UnsafeCell<Option<VecDeque<T>>>,  //队列头
}

unsafe impl<T: 'static> Send for InnerDeque<T> {}
unsafe impl<T: 'static> Sync for InnerDeque<T> {}

///
/// MPMC双端队列
///
pub struct MpmcDeque<T: 'static> {
    inner:  Arc<InnerDeque<T>>, //内部自旋锁双端队列
}

unsafe impl<T: 'static> Send for MpmcDeque<T> {}
unsafe impl<T: 'static> Sync for MpmcDeque<T> {}

impl<T: 'static> Clone for MpmcDeque<T> {
    fn clone(&self) -> Self {
        MpmcDeque {
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> MpmcDeque<T> {
    /// 构建自旋锁双端队列
    pub fn new() -> Self {
        let inner = Arc::new(InnerDeque {
            tail_status: AtomicU8::new(UNLOCK_EMPTY),
            tail: UnsafeCell::new(Some(Vec::new())),
            head_status: AtomicU8::new(UNLOCK_EMPTY),
            head: UnsafeCell::new(Some(VecDeque::new())),
        });

        MpmcDeque {
            inner,
        }
    }

    /// 构建指定初始容量的自旋锁双端队列
    pub fn with_capacity(capacity: usize) -> Self {
        let inner = Arc::new(InnerDeque {
            tail_status: AtomicU8::new(UNLOCK_EMPTY),
            tail: UnsafeCell::new(Some(Vec::with_capacity(capacity))),
            head_status: AtomicU8::new(UNLOCK_EMPTY),
            head: UnsafeCell::new(Some(VecDeque::new())),
        });

        MpmcDeque {
            inner,
        }
    }

    /// 检查队列尾是否为空
    pub fn is_empty_tail(&self) -> bool {
        self.inner.tail_status.load(Ordering::SeqCst) == UNLOCK_EMPTY
    }

    /// 检查队列头是否为空
    pub fn is_empty_head(&self) -> bool {
        self.inner.head_status.load(Ordering::SeqCst) == UNLOCK_EMPTY
    }

    /// 获取队列尾长度
    pub fn tail_len(&self) -> usize {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.tail_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则获取队列尾长度
                    unsafe {
                        let len = (&*self.inner.tail.get()).as_ref().unwrap().len();
                        if len > 0 {
                            self.inner.tail_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.tail_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        }
                        return len;
                    }
                }
            }
        }
    }

    /// 获取队列头长度
    pub fn head_len(&self) -> usize {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.head_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则获取队列尾长度
                    unsafe {
                        let len = (&*self.inner.head.get()).as_ref().unwrap().len();
                        if len > 0 {
                            self.inner.head_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.head_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        }
                        return len;
                    }
                }
            }
        }
    }

    /// 从队列头弹出值
    pub fn pop(&self) -> Option<T> {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.head_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则弹出队列头
                    unsafe {
                        let head = (&mut *self.inner.head.get()).as_mut().unwrap();
                        if let Some(value) = head.pop_front() {
                            if head.is_empty() {
                                self.inner.head_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                            } else {
                                self.inner.head_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                            }

                            return Some(value);
                        } else {
                            //队列头没有值，则尝试从队列尾中弹出值
                            status = UNLOCK_NON_EMPTY;
                            loop {
                                match self.inner.tail_status.compare_exchange_weak(status,
                                                                                   LOCKED,
                                                                                   Ordering::Acquire,
                                                                                   Ordering::Relaxed) {
                                    Err(current) if current == LOCKED => {
                                        //已锁，则自旋后继续尝试锁
                                        spin_len = spin(spin_len);
                                        continue;
                                    },
                                    Err(current) => {
                                        //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                                        status = current;
                                        continue;
                                    },
                                    Ok(_) => {
                                        //锁成功，则交换队列尾和队列头，并弹出队列头
                                        if swap(&self.inner.tail, &self.inner.head) {
                                            self.inner.tail_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                            self.inner.head_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);

                                            return (&mut *self.inner.head.get()).as_mut().unwrap().pop_front();
                                        } else {
                                            if (&*self.inner.tail.get()).as_ref().unwrap().is_empty() {
                                                self.inner.tail_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                            } else {
                                                self.inner.tail_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                                            }
                                            self.inner.head_status.store(UNLOCK_EMPTY, Ordering::SeqCst);

                                            return None;
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 从队列头加入值
    pub fn push_front(&self, value: T) {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.head_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则加入队列头
                    unsafe {
                        let head = (&mut *self.inner.head.get()).as_mut().unwrap();
                        head.push_front(value);

                        //解锁，并返回
                        if head.is_empty() {
                            self.inner.head_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.head_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        }
                        return;
                    }
                }
            }
        }
    }

    /// 从队列尾加入值
    pub fn push_back(&self, value: T) {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.tail_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则加入队列尾
                    unsafe {
                        let tail = (&mut *self.inner.tail.get()).as_mut().unwrap();
                        tail.push(value);

                        //解锁，并返回
                        if tail.is_empty() {
                            self.inner.tail_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.tail_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        }
                        return;
                    }
                }
            }
        }
    }

    /// 从队列头取所有值
    pub fn take_heads(&self) -> VecDeque<T> {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.head_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则取出所有值
                    let r = replace_head(&self.inner.head);

                    //解锁，并返回
                    self.inner.head_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                    return r;
                }
            }
        }
    }

    /// 从队列尾取所有值
    pub fn take_tails(&self) -> Vec<T> {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.tail_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则取出所有值
                    let r = replace_tail(&self.inner.tail);

                    //解锁，并返回
                    self.inner.tail_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                    return r;
                }
            }
        }
    }

    /// 连接到队列头
    pub fn join(&self, head: Vec<T>) {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.head_status.compare_exchange_weak(status,
                                                               LOCKED,
                                                               Ordering::Acquire,
                                                               Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则
                    join_head(&self.inner.head, head);

                    //解锁，并返回
                    self.inner.head_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                    return;
                }
            }
        }
    }
}

//交换队列头和队列尾
#[inline]
fn swap<T: 'static>(tail: &UnsafeCell<Option<Vec<T>>>, head: &UnsafeCell<Option<VecDeque<T>>>) -> bool {
    unsafe {
        let tail = tail.get();
        let head = head.get();
        if (&*tail).as_ref().unwrap().len() > 0 && (&*head).as_ref().unwrap().len() == 0 {
            //队列尾非空，且队列头为空，则交换，并返回成功
            let vec = (&mut *tail).take().unwrap();
            let vec_deque = (&mut *head).take().unwrap();
            *tail = Some(vec_deque.into());
            *head = Some(vec.into());
            true
        } else {
            //队列尾为空，或队列头非空，则返回失败
            false
        }
    }
}

//替换队列头
#[inline]
fn replace_head<T: 'static>(head: &UnsafeCell<Option<VecDeque<T>>>) -> VecDeque<T> {
    unsafe {
        let head = head.get();
        let vec_deque = (&mut *head).take().unwrap();
        *head = Some(VecDeque::new());
        vec_deque
    }
}

//替换队列尾
#[inline]
fn replace_tail<T: 'static>(tail: &UnsafeCell<Option<Vec<T>>>) -> Vec<T> {
    unsafe {
        let tail = tail.get();
        let vec = (&mut *tail).take().unwrap();
        *tail = Some(Vec::new());
        vec
    }
}

//连接到队列头
fn join_head<T: 'static>(current: &UnsafeCell<Option<VecDeque<T>>>, mut new: Vec<T>) {
    unsafe {
        let head = (&mut *current.get()).as_mut().unwrap();
        while let Some(value) = new.pop() {
            head.push_front(value);
        }
    }
}

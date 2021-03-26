//! # 多生产者单消费者的单端队列
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

///
/// 构建MPSC的双端队列，并返回发送者和接收者
///
pub fn mpsc_deque<T: 'static>() -> (Sender<T>, Receiver<T>) {
    let send_buf = Arc::new(SendBuf {
        buf_status: AtomicU8::new(UNLOCK_EMPTY),
        buf: UnsafeCell::new(Some(Vec::new())),
    });
    let sender = Sender {
        inner: send_buf,
    };

    let recv_buf = RecvBuf {
        sender: sender.clone(),
        buf: UnsafeCell::new(Some(VecDeque::new())),
    };

    (sender,
     Receiver {
        inner: recv_buf,
    })
}

/*
* 发送缓冲区
*/
struct SendBuf<T: 'static> {
    buf_status:    AtomicU8,                         //缓冲区锁状态
    buf:           UnsafeCell<Option<Vec<T>>>,       //缓冲区
}

///
/// 双端队列的发送者
///
pub struct Sender<T: 'static> {
    inner:  Arc<SendBuf<T>>, //缓冲区
}

unsafe impl<T: 'static> Send for Sender<T> {}
unsafe impl<T: 'static> Sync for Sender<T> {}

impl<T: 'static> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> Sender<T> {
    /// 尝试检查发送缓冲区是否为空，不允许用于精确判断
    pub fn try_is_empty(&self) -> bool {
        self.inner.buf_status.load(Ordering::SeqCst) == UNLOCK_EMPTY
    }

    /// 获取发送缓冲区长度，可用于精确判断
    pub fn len(&self) -> usize {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.buf_status.compare_exchange_weak(status,
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
                    //锁成功，则获取发送缓冲区长度
                    unsafe {
                        let len = (&*self.inner.buf.get()).as_ref().unwrap().len();
                        if len > 0 {
                            self.inner.buf_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        }
                        return len;
                    }
                }
            }
        }
    }

    /// 发送指定的值
    pub fn send(&self, value: T) {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.buf_status.compare_exchange_weak(status,
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
                    //锁成功，则加入发送缓冲区
                    unsafe {
                        let tail = (&mut *self.inner.buf.get()).as_mut().unwrap();
                        tail.push(value);

                        //解锁，并返回
                        if tail.is_empty() {
                            self.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.buf_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        }
                        return;
                    }
                }
            }
        }
    }

    /// 获取发送缓冲区
    pub fn take(&self) -> Vec<T> {
        let mut spin_len = 1;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            match self.inner.buf_status.compare_exchange_weak(status,
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
                    let r = replace_send_buf(&self.inner.buf);

                    //解锁，并返回
                    self.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                    return r;
                }
            }
        }
    }
}

/*
* 接收缓冲区
*/
struct RecvBuf<T: 'static> {
    sender: Sender<T>,                          //发送者
    buf:    UnsafeCell<Option<VecDeque<T>>>,    //缓冲区
}

///
/// 双端队列的接收者
///
pub struct Receiver<T: 'static> {
    inner:  RecvBuf<T>,  //缓冲区
}

unsafe impl<T: 'static> Send for Receiver<T> {}

impl<T: 'static> Receiver<T> {
    /// 尝试检查队列是否为空，不允许用于精确判断
    pub fn try_is_empty(&self) -> bool {
        unsafe {
            self.inner.sender.try_is_empty() && (&*self.inner.buf.get()).as_ref().unwrap().is_empty()
        }
    }

    /// 获取队列长度，可用于精确判断
    pub fn len(&self) -> usize {
        unsafe {
            self.inner.sender.len() + (&*self.inner.buf.get()).as_ref().unwrap().len()
        }
    }

    /// 将指定值推入接收缓冲区头
    pub fn push_front(&mut self, value: T) {
        unsafe {
            (&mut *self.inner.buf.get()).as_mut().unwrap().push_front(value);
        }
    }

    /// 非阻塞接收值
    pub fn try_recv(&mut self) -> Option<T> {
        unsafe {
            if let Some(value) = (&mut *self.inner.buf.get()).as_mut().unwrap().pop_front() {
                //接收缓冲区有值，则立即返回
                Some(value)
            } else {
                //接收缓冲区没有值
                if self.inner.sender.try_is_empty() {
                    //发送缓冲区没有值，则立即返回空
                    None
                } else {
                    //发送缓冲区有值，则交换发送缓冲区和接收缓冲区，并从接收缓冲区弹出
                    let mut spin_len = 1;
                    let mut status = UNLOCK_NON_EMPTY;
                    loop {
                        match self.inner.sender.inner.buf_status.compare_exchange_weak(status,
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
                                //锁成功，则交换发送缓冲区和接收缓冲区，并从接收缓冲区弹出
                                if swap(self.inner.sender.inner.buf.get(), self.inner.buf.get()) {
                                    //交换成功，则从交换后的接收缓冲区弹出值
                                    self.inner.sender.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                    return (&mut *self.inner.buf.get()).as_mut().unwrap().pop_front();
                                } else {
                                    //交换失败，则立即返回空
                                    self.inner.sender.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                    return None;
                                }
                            },
                        }
                    }
                }
            }
        }
    }

    /// 非阻塞接收当前所有值
    pub fn try_recv_all(&mut self) -> Vec<T> {
        let mut truncated = false;
        let mut vec = Vec::new();
        unsafe {
            loop {
                if let Some(value) = (&mut *self.inner.buf.get()).as_mut().unwrap().pop_front() {
                    //接收缓冲区有值，则缓存，并继续弹出接收缓冲区的值
                    vec.push(value);
                } else {
                    //接收缓冲区没有值
                    if truncated || self.inner.sender.try_is_empty() {
                        //本次获取已截短或发送缓冲区没有值，则立即返回
                        return vec;
                    } else {
                        //发送缓冲区有值，则交换发送缓冲区和接收缓冲区，并从接收缓冲区弹出
                        let mut spin_len = 1;
                        let mut status = UNLOCK_NON_EMPTY;
                        loop {
                            match self.inner.sender.inner.buf_status.compare_exchange_weak(status,
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
                                    //锁成功，则交换发送缓冲区和接收缓冲区，并从接收缓冲区弹出
                                    if swap(self.inner.sender.inner.buf.get(), self.inner.buf.get()) {
                                        //交换成功，则从交换后的接收缓冲区弹出值
                                        self.inner.sender.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                        truncated = true; //已截短
                                        break;
                                    } else {
                                        //交换失败，则立即返回
                                        self.inner.sender.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                        return vec;
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

//交换发送缓冲区和接收缓冲区
#[inline]
fn swap<T: 'static>(send_buf: *mut Option<Vec<T>>, recv_buf: *mut Option<VecDeque<T>>) -> bool {
    unsafe {
        if (&*send_buf).as_ref().unwrap().len() > 0 && (&*recv_buf).as_ref().unwrap().len() == 0 {
            //发送缓冲区非空，且接收缓冲区为空，则交换
            let vec = (&mut *send_buf).take().unwrap();
            let deque = (&mut *recv_buf).take().unwrap();
            *send_buf = Some(deque.into());
            *recv_buf = Some(vec.into());
            true
        } else {
            //发送缓冲区为空，或接收缓冲区非空
            false
        }
    }
}

//替换发送缓冲区
#[inline]
fn replace_send_buf<T: 'static>(buf: &UnsafeCell<Option<Vec<T>>>) -> Vec<T> {
    unsafe {
        let send_buf = buf.get();
        let vec = (&mut *send_buf).take().unwrap();
        *send_buf = Some(Vec::new());
        vec
    }
}

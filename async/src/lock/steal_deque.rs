//! # 多生产者多消费者的双端队列，支持任务窃取
//!

use std::sync::Arc;
use std::ptr::null_mut;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicPtr, AtomicUsize, Ordering};

use parking_lot::{Mutex, Condvar};

use super::spin;

/*
* 锁状态
*/
const UNLOCK_EMPTY: u8 = 0;     //无锁无任务
const UNLOCK_NON_EMPTY: u8 = 1; //无锁有任务
const LOCKED: u8 = 2;           //有锁

///
/// 构建支持窃取值的MPSC双端队列，并返回发送者和接收者
///
pub fn steal_deque<T: 'static>() -> (Sender<T>, Receiver<T>) {
    let send_buf = Arc::new(SendBuf {
        buf_status: AtomicU8::new(UNLOCK_EMPTY),
        buf: UnsafeCell::new(Some(Vec::new())),
    });
    let sender = Sender {
        inner: send_buf,
    };

    let recv_buf = Arc::new(RecvBuf {
        sender: sender.clone(),
        deque: AtomicPtr::new(Box::into_raw(Box::new(VecDeque::new()))),
        buf: UnsafeCell::new(None),
    });

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

    /// 尝试指定次数，发送指定的值，发送失败则返回原值
    pub fn try_send(&self, limit: usize, value: T) -> Option<T> {
        let mut spin_len = 1;
        let mut try_count = 0;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            if try_count > limit {
                //尝试次数达到上限，则立即返回值
                return Some(value);
            }

            match self.inner.buf_status.compare_exchange_weak(status,
                                                              LOCKED,
                                                              Ordering::Acquire,
                                                              Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    try_count += 1;
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    try_count += 1;
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
                        return None;
                    }
                }
            }
        }
    }

    /// 发送指定的值，返回当前发送缓冲区长度
    pub fn send(&self, value: T) -> usize {
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
                        let len = tail.len();
                        if len == 0 {
                            self.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.buf_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        }
                        return len;
                    }
                }
            }
        }
    }

    /// 将指定的缓冲区追加到当前缓冲区尾部，返回当前发送缓冲区长度
    pub fn append(&self, buf: &mut Vec<T>) -> usize {
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
                        tail.append(buf);

                        //解锁，并返回
                        let len = tail.len();
                        if len == 0 {
                            self.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                        } else {
                            self.inner.buf_status.store(UNLOCK_NON_EMPTY, Ordering::SeqCst);
                        }
                        return len;
                    }
                }
            }
        }
    }

    /// 尝试指定次数，获取发送缓冲区
    pub fn try_take(&self, limit: usize) -> Option<Vec<T>> {
        let mut spin_len = 1;
        let mut try_count = 0;
        let mut status = UNLOCK_NON_EMPTY;
        loop {
            if try_count > limit {
                //尝试次数已达限制，则立即返回空
                return None;
            }

            match self.inner.buf_status.compare_exchange_weak(status,
                                                              LOCKED,
                                                              Ordering::Acquire,
                                                              Ordering::Relaxed) {
                Err(current) if current == LOCKED => {
                    //已锁，则自旋后继续尝试锁
                    spin_len = spin(spin_len);
                    try_count += 1;
                    continue;
                },
                Err(current) => {
                    //锁状态不匹配，则更新当前锁状态，并立即尝试锁
                    status = current;
                    try_count += 1;
                    continue;
                },
                Ok(_) => {
                    //锁成功，则取出所有值
                    let r = replace_send_buf(&self.inner.buf);

                    //解锁，并返回
                    self.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                    return Some(r);
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
    sender: Sender<T>,              //发送者
    deque:  AtomicPtr<VecDeque<T>>, //队列
    buf:    UnsafeCell<Option<T>>,  //缓冲区
}

///
/// 双端队列的接收者
///
pub struct Receiver<T: 'static> {
    inner:  Arc<RecvBuf<T>>,  //缓冲区
}

unsafe impl<T: 'static> Send for Receiver<T> {}
unsafe impl<T: 'static> Sync for Receiver<T> {}

impl<T: 'static> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> Receiver<T> {
    /// 检查接收队列是否为空
    pub fn is_empty_recv(&self) -> bool {
        unsafe {
            let deque = self.inner.deque.load(Ordering::SeqCst);
            if deque.is_null() {
                //正在非阻塞的接收值
                true
            } else {
                (&*deque).is_empty()
            }
        }
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        unsafe {
            let deque = self.inner.deque.load(Ordering::SeqCst);
            if deque.is_null() {
                //正在非阻塞的接收值
                self.inner.sender.try_is_empty() && (&*self.inner.buf.get()).is_none()
            } else {
                self.inner.sender.try_is_empty()
                    && (&*deque).is_empty()
                    && (&*self.inner.buf.get()).is_none()
            }
        }
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        let deque = self.inner.deque.load(Ordering::SeqCst);
        if deque.is_null() {
            //正在非阻塞的接收值
            0
        } else {
            unsafe {
                if (&*self.inner.buf.get()).is_none() {
                    self.inner.sender.len() + (&*deque).len()
                } else {
                    self.inner.sender.len() + (&*deque).len() + 1
                }
            }
        }
    }

    /// 非阻塞接收值，并记录当前接收队列的长度
    pub fn try_recv(&self, counter: &AtomicUsize) -> Option<T> {
        if let Some(value) = unsafe { (&mut *self.inner.buf.get()).take() } {
            Some(value)
        } else {
            //接收缓冲区没有值，则从接收队列弹出两个值，并返回首个值
            if let Some(r) = self.pop(&counter) {
                if let Some(value) = self.pop(&counter) {
                    //填充缓冲区
                    unsafe { *self.inner.buf.get() = Some(value); }
                }

                Some(r)
            } else {
                None
            }
        }
    }

    /// 非阻塞弹出值，并记录当前接收队列的长度
    fn pop(&self, counter: &AtomicUsize) -> Option<T> {
        if let Some(mut deque) = swap_recv_deque(&self.inner.deque, null_mut()) {
            if let Some(value) = deque.pop_front() {
                //归还接收队列，忽略交换返回，并返回
                counter.fetch_sub(1, Ordering::Relaxed); //减少接收队列的任务计数
                swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(deque)));
                Some(value)
            } else {
                //接收队列没有值
                if self.inner.sender.try_is_empty() {
                    //发送缓冲区没有值，则归还接收队列，忽略交换返回，并立即返回空
                    swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(deque)));
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
                                //锁成功
                                match swap(&self.inner.sender.inner.buf, deque) {
                                    Err(deque) => {
                                        //交换失败，归还当前接收队列，忽略交换返回，并立即返回空
                                        self.inner.sender.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);
                                        swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(deque)));
                                        return None;
                                    },
                                    Ok(mut new_deque) => {
                                        //交换成功，归还交换后的接收队列，忽略交换返回，并从交换后的接收队列弹出值
                                        self.inner.sender.inner.buf_status.store(UNLOCK_EMPTY, Ordering::SeqCst);

                                        let r = new_deque.pop_front();
                                        let new_deque_len = new_deque.len();
                                        if new_deque_len > 0 {
                                            //增加接收队列的任务计数
                                            counter.fetch_add(new_deque.len(), Ordering::Relaxed);
                                        }
                                        swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(new_deque)));
                                        return r;
                                    },
                                }
                            },
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    /// 获取接收队列
    pub fn take(&self) -> Option<VecDeque<T>> {
        //交换接收队列，返回当前接收队列
        swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(VecDeque::new())))
    }

    /// 向接收缓冲区头部增加值
    pub fn push_front(&self, value: T, counter: &AtomicUsize)  {
        unsafe {
            if let Some(last_value) = (&mut *self.inner.buf.get()).take() {
                //缓冲区有值，则将缓冲区的值放入接收缓冲区头
                if let Some(mut deque) = swap_recv_deque(&self.inner.deque, null_mut()) {
                    deque.push_front(last_value);
                    swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(deque)));
                } else {
                    panic!("!!!!!!Push to recv deque front failed, reason: recv dqeueu not exist");
                }
            }

            //将值放入缓冲区，并增加接收队列的任务计数
            *self.inner.buf.get() = Some(value);
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 向接收缓冲区尾部增加值
    pub fn append(&self, value: T, counter: &AtomicUsize)  {
        if let Some(mut deque) = swap_recv_deque(&self.inner.deque, null_mut()) {
            deque.push_back(value);
            counter.fetch_add(1, Ordering::Relaxed); //增加接收队列的任务计数
            swap_recv_deque(&self.inner.deque, Box::into_raw(Box::new(deque)));
        } else {
            panic!("!!!!!!Append to recv deque back failed, reason: recv dqeueu not exist");
        }
    }
}

//交换发送缓冲区和接收队列，成功返回交换后的接收队列，失败返回当前接收队列
#[inline]
fn swap<T: 'static>(send_buf: &UnsafeCell<Option<Vec<T>>>, recv_deque: VecDeque<T>) -> Result<VecDeque<T>, VecDeque<T>> {
    unsafe {
        let send_buf = send_buf.get();
        if (&*send_buf).as_ref().unwrap().len() > 0 && recv_deque.len() == 0 {
            //发送缓冲区非空，且接收队列为空，则交换，并返回被交换后的接收队列
            let vec = (&mut *send_buf).take().unwrap();
            *send_buf = Some(recv_deque.into());
            Ok(vec.into())
        } else {
            //发送缓冲区为空，或接收队列非空，则返回当前接收队列
            Err(recv_deque)
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

//交换接收队列，并返回上一个接收队列
#[inline]
fn swap_recv_deque<T: 'static>(handle: &AtomicPtr<VecDeque<T>>, recv_deque: *mut VecDeque<T>) -> Option<VecDeque<T>> {
    let last_recv_deque = handle.swap(recv_deque, Ordering::SeqCst);
    if last_recv_deque.is_null() {
        //上一个接收队列为空，则忽略
        None
    } else {
        //上一个接收队列不为空，则解引用接收队列，防止接收队列泄漏
        unsafe {
            Some(*Box::from_raw(last_recv_deque))
        }
    }
}
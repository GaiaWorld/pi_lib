//! # 通用异步读写锁，不支持重入
//!

use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::task::{Waker, Context, Poll};

use super::spin_lock::SpinLock;

/*
* 读写锁状态
*/
const EXCLUSIVE: isize = -1;        //独占
const UNLOCKED: isize = 0;          //未锁
const SHARED_ONCE: isize = 1;       //唯一共享

///
/// 异步读锁守护者
///
pub struct RwLockReadGuard<T> {
    guarder:  Arc<InnerRwLock<T>>,  //内部锁
}

impl<T> Deref for RwLockReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.guarder.inner.get()
        }
    }
}

impl<T> Drop for RwLockReadGuard<T> {
    fn drop(&mut self) {
        if let Some(waker) = {
            let mut status = self.guarder.status.lock();
            //释放异步共享锁，并返回是否准备解锁共享锁
            if status.0 <= UNLOCKED {
                //当前锁状态错误，则立即抛出异常，并释放当前共享锁
                panic!("Free shared lock failed, current: {:?}, reason: invalid current status", status.0);
            } else if status.0 > SHARED_ONCE {
                //当前不是释放唯一共享锁，则减去共享锁计数，立即返回，并释放当前共享锁
                (&mut status).0 -= 1;
                return;
            } else {
                //当前锁状态满足条件，返回需要唤醒的独占锁任务，并解锁当前共享锁
                (&mut status).0 = UNLOCKED;
                status.2.pop_front()
            }
        } {
            //有异步独占锁任务等待当前异步共享锁释放，则唤醒此独占锁任务
            waker.wake();
        }
    }
}

/*
* 异步写锁守护者
*/
pub struct RwLockWriteGuard<T> {
    guarder:  Arc<InnerRwLock<T>>,  //内部锁
}

impl<T> Deref for RwLockWriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.guarder.inner.get()
        }
    }
}

impl<T> DerefMut for RwLockWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.guarder.inner.get()
        }
    }
}

impl<T> Drop for RwLockWriteGuard<T> {
    fn drop(&mut self) {
        if let wakers = {
            let mut status = self.guarder.status.lock();
            (&mut status).0 = UNLOCKED;

            let mut wakers = Vec::new();
            if status.1.len() > 0 {
                //有异步共享锁任务等待异步独占锁释放，则唤醒所有共享锁任务，并释放当前独占锁
                while let Some(waker) = status.1.pop_front() {
                    wakers.push(waker);
                }
            } else if status.2.len() > 0 {
                //有异步独占锁任务等待当前异步独占锁释放，则唤醒此独占锁任务，并释放当前独占锁
                if let Some(waker) = status.2.pop_front() {
                    wakers.push(waker);
                }
            } else {
                //没有异步共享锁任务等待异步独占锁释放，也没有异步独占锁任务等待当前异步独占锁释放，则立即返回，并释放当前独占锁
                return;
            }

            wakers
        } {
            //有异步读写锁任务等待当前异步读写锁释放，则唤醒此读写锁任务
            for waker in wakers {
                waker.wake();
            }
        }
    }
}

///
/// 异步读写锁，支持临界区内执行异步任务等待，不支持重入
///
pub struct RwLock<T> {
    inner:  Arc<InnerRwLock<T>>,  //内部锁
}

unsafe impl<T> Send for RwLock<T> {}
unsafe impl<T> Sync for RwLock<T> {}

/*
* 异步读写锁同步方法
*/
impl<T> RwLock<T> {
    /// 构建异步读写锁
    pub fn new(v: T) -> Self {
        let inner = Arc::new(InnerRwLock {
            status: SpinLock::new((UNLOCKED, VecDeque::new(), VecDeque::new())),
            inner: UnsafeCell::new(v),
        });

        RwLock {
            inner,
        }
    }
}

/*
* 异步读写锁异步方法，支持临界区内执行异步任务等待，不支持重入
*/
impl<T> RwLock<T> {
    /// 获取异步读锁
    pub async fn read(&self) -> RwLockReadGuard<T> {
        FutureShared {
            inner: self.inner.clone(),
        }.await
    }

    /// 获取异步写锁
    pub async fn write(&self) -> RwLockWriteGuard<T> {
        FutureExclusive {
            inner: self.inner.clone(),
        }.await
    }
}

/*
* 内部异步读写锁
*/
struct InnerRwLock<T> {
    status: SpinLock<(isize, VecDeque<Waker>, VecDeque<Waker>)>,    //异步读写锁状态
    inner:  UnsafeCell<T>,                                          //异步读写锁内容
}

unsafe impl<T> Send for InnerRwLock<T> {}
unsafe impl<T> Sync for InnerRwLock<T> {}

/*
* 共享锁异步任务
*/
struct FutureShared<T> {
    inner:  Arc<InnerRwLock<T>>,  //内部锁
}

impl<T> Future for FutureShared<T> {
    type Output = RwLockReadGuard<T>;

    //抢占式的获取共享锁
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //尝试获取异步共享锁
        let mut status = self.inner.status.lock();
        if status.0 >= UNLOCKED {
            //获取异步共享锁成功，则返回异步读锁守护者
            (&mut status).0 += 1; //增加共享计数
            return Poll::Ready(RwLockReadGuard {
                guarder: (&self).inner.clone()
            });
        }

        //尝试获取异步共享锁失败，则加入共享等待队列
        status.1.push_back(cx.waker().clone());
        Poll::Pending
    }
}

/*
* 独占锁异步任务
*/
struct FutureExclusive<T> {
    inner:  Arc<InnerRwLock<T>>,  //内部锁
}

impl<T> Future for FutureExclusive<T> {
    type Output = RwLockWriteGuard<T>;

    //抢占式的获取独占锁
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //尝试获取异步独占锁
        let mut status = self.inner.status.lock();
        if status.0 == UNLOCKED {
            //获取异步独占锁成功，则返回异步写锁守护者
            (&mut status).0 = EXCLUSIVE;
            return Poll::Ready(RwLockWriteGuard {
                guarder: (&self).inner.clone()
            });
        }

        //尝试获取异步独占锁失败，则加入独占等待队列
        status.2.push_back(cx.waker().clone());
        Poll::Pending
    }
}
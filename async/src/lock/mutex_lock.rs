//! # 通用异步互斥锁，不支持重入
//!

use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::task::{Waker, Context, Poll};

use super::spin_lock::SpinLock;

///
/// 异步互斥锁守护者
///
pub struct MutexGuard<T> {
    guarder:  Arc<InnerMutex<T>>,  //内部锁
}

impl<T> Deref for MutexGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.guarder.inner.get()
        }
    }
}

impl<T> DerefMut for MutexGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.guarder.inner.get()
        }
    }
}

impl<T> Drop for MutexGuard<T> {
    fn drop(&mut self) {
        if let Some(waker) = {
            //解锁当前互斥锁
            let mut status = self.guarder.status.lock();
            (&mut status).0 = true;
            status.1.pop_front()
        } {
            //有异步互斥锁任务等待当前异步互斥锁释放，则唤醒此互斥锁任务
            waker.wake();
        }
    }
}

///
/// 异步互斥锁，支持临界区内执行异步任务等待，不支持重入
///
pub struct Mutex<T> {
    inner:  Arc<InnerMutex<T>>,  //内部锁
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

/*
* 异步互斥锁同步方法
*/
impl<T> Mutex<T> {
    /// 构建异步互斥锁
    pub fn new(v: T) -> Self {
        let inner = Arc::new(InnerMutex {
            status: SpinLock::new((true, VecDeque::new())),
            inner: UnsafeCell::new(v),
        });

        Mutex {
            inner,
        }
    }
}

/*
* 异步互斥锁异步方法
*/
impl<T> Mutex<T> {
    /// 获取异步互斥锁
    pub async fn lock(&self) -> MutexGuard<T> {
        FutureMutex {
            inner: self.inner.clone(),
        }.await
    }
}

/*
* 内部异步互斥锁
*/
struct InnerMutex<T> {
    status:         SpinLock<(bool, VecDeque<Waker>)>,  //异步互斥锁状态
    inner:          UnsafeCell<T>,                      //异步互斥锁内容
}

unsafe impl<T> Send for InnerMutex<T> {}
unsafe impl<T> Sync for InnerMutex<T> {}

/*
* 互斥锁异步任务
*/
struct FutureMutex<T> {
    inner:  Arc<InnerMutex<T>>,  //内部锁
}

impl<T> Future for FutureMutex<T> {
    type Output = MutexGuard<T>;

    //抢占式的获取互斥锁
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut status = self.inner.status.lock();
        if status.0 {
            //获取异步互斥锁成功，则返回异步互斥锁守护者
            (&mut status).0 = false;
            return Poll::Ready(MutexGuard {
                guarder: (&self).inner.clone()
            });
        }

        //尝试获取异步互斥锁失败，则加入锁等待队列
        status.1.push_back(cx.waker().clone());
        Poll::Pending
    }
}

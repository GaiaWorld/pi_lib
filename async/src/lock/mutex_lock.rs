use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::task::{Waker, Context, Poll};
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_queue::SegQueue;

use super::spin;

/*
* 异步互斥锁守护者
*/
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
        self.guarder.unlock();
        self.guarder.lock_waits();
        if let Ok(waker) = self.guarder.waits.pop() {
            waker.wake();
        }
        self.guarder.unlock_waits();
    }
}

/*
* 异步互斥锁
*/
pub struct Mutex<T> {
    inner:  Arc<InnerMutex<T>>,  //内部锁
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

/*
* 异步互斥锁同步方法
*/
impl<T> Mutex<T> {
    //构建异步互斥锁
    pub fn new(v: T) -> Self {
        let inner = Arc::new(InnerMutex {
            waits_status: AtomicBool::new(false),
            waits: SegQueue::new(),
            lock_status: AtomicBool::new(false),
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
    //获取异步互斥锁
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
    waits_status: AtomicBool,   //锁等待队列状态
    waits:  SegQueue<Waker>,    //锁等待队列
    lock_status: AtomicBool,    //异步互斥锁状态
    inner:  UnsafeCell<T>,      //异步互斥锁内容
}

unsafe impl<T> Send for InnerMutex<T> {}
unsafe impl<T> Sync for InnerMutex<T> {}

impl<T> InnerMutex<T> {
    //加入等待队列
    #[inline]
    pub fn push(&self, waker: Waker) {
        self.waits.push(waker);
    }

    //锁住等待队列
    pub fn lock_waits(&self) {
        let mut spin_len = 1;
        loop {
            match self.waits_status.compare_exchange(false,
                                                     true,
                                                     Ordering::Relaxed,
                                                     Ordering::Relaxed) {
                Ok(_) => return, //锁成功
                Err(_) => {
                    //锁失败，则自旋后，继续锁
                    spin_len = spin(spin_len);
                    continue;
                },
            }
        }
    }

    //解锁等待队列
    pub fn unlock_waits(&self) {
        self.waits_status.store(false, Ordering::Relaxed);
    }

    //尝试获取异步互斥锁，返回是否成功
    pub fn try_lock(&self) -> bool {
        match self.lock_status.compare_exchange(false,
                                                true,
                                                Ordering::Acquire,
                                                Ordering::Relaxed) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    //解锁异步互斥锁，并唤醒锁等待队列头
    pub fn unlock(&self) {
        self.lock_status.store(false, Ordering::Relaxed);
    }
}

/*
* 互斥锁异步任务
*/
struct FutureMutex<T> {
    inner:  Arc<InnerMutex<T>>,  //内部锁
}

impl<T> Future for FutureMutex<T> {
    type Output = MutexGuard<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //阻塞获取等待队列的锁
        (&self).inner.lock_waits();

        //尝试获取异步互斥锁
        for spin_len in 1..5 {
            if (&self).inner.try_lock() {
                //获取锁成功，则立即解锁等待队列，并返回异步互斥锁守护者
                (&self).inner.unlock_waits();
                return Poll::Ready(MutexGuard {
                    guarder: (&self).inner.clone()
                });
            } else {
                //获取锁失败，则自旋后，再次尝试
                spin(spin_len);
                continue;
            }
        }

        //尝试获取异步互斥锁失败，则加入锁等待队列，并立即解锁等待队列
        (&self).inner.push(cx.waker().clone());
        (&self).inner.unlock_waits();
        Poll::Pending
    }
}


use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::task::{Waker, Context, Poll};
use std::sync::atomic::{AtomicBool, Ordering};

use super::{spin, mpsc_deque::{Sender, Receiver, mpsc_deque}};

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
        unsafe {
            let consumer = &mut *self.guarder.consumer;
            //因为互斥锁保证了，同一时间只有一个线程可以获取到锁，所以可以使用不精确的检查接收队列是否为空的检查
            if consumer.try_is_empty() {
                self.guarder.unlock();
            } else {
                if let Some(waker) = consumer.try_recv() {
                    //有异步任务等待异步互斥锁释放，则解锁并唤醒此任务
                    self.guarder.unlock();
                    waker.wake();
                } else {
                    self.guarder.unlock();
                }
            }
        }
    }
}

/*
* 异步互斥锁，支持临界区内执行异步任务等待，不支持重入
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
        let (producor, consumer) = mpsc_deque();
        let inner = Arc::new(InnerMutex {
            producor,
            consumer: Box::into_raw(Box::new(consumer)),
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
    producor:       Sender<Waker>,          //锁等待队列生产者
    consumer:       *mut Receiver<Waker>,   //锁等待队列消费者
    lock_status:    AtomicBool,             //异步互斥锁状态
    inner:          UnsafeCell<T>,          //异步互斥锁内容
}

unsafe impl<T> Send for InnerMutex<T> {}
unsafe impl<T> Sync for InnerMutex<T> {}

impl<T> InnerMutex<T> {
    //加入等待队列
    #[inline(always)]
    pub fn push(&self, waker: Waker) {
        self.producor.send(waker);
    }

    //尝试获取异步互斥锁，返回是否成功
    #[inline(always)]
    pub fn try_lock(&self) -> bool {
        self.lock_status.compare_exchange_weak(false,
                                               true,
                                               Ordering::Acquire,
                                               Ordering::Relaxed)
            .is_ok()
    }

    //解锁异步互斥锁
    #[inline(always)]
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

    //抢占式的获取互斥锁
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //尝试获取异步互斥锁
        for spin_len in 1..10 {
            if (&self).inner.try_lock() {
                //获取异步互斥锁成功，则返回异步互斥锁守护者
                return Poll::Ready(MutexGuard {
                    guarder: (&self).inner.clone()
                });
            } else {
                //获取异步互斥锁失败，则自旋后，再次尝试
                spin(spin_len);
                continue;
            }
        }

        //尝试获取异步互斥锁失败，则加入锁等待队列
        (&self).inner.push(cx.waker().clone());
        Poll::Pending
    }
}

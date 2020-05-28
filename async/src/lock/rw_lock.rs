use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::task::{Waker, Context, Poll};
use std::sync::atomic::{AtomicIsize, Ordering};

use super::{spin, mpsc_deque::{Sender, Receiver, mpsc_deque}};

/*
* 读写锁状态
*/
const UNLOCK_SHARED: isize = -10;   //解锁共享
const EXCLUSIVE: isize = -1;        //独占
const UNLOCKED: isize = 0;          //未锁
const SHARED_ONCE: isize = 1;       //唯一共享

/*
* 异步读锁守护者
*/
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
        if self.guarder.free_shared() {
            //准备解锁共享锁
            unsafe {
                let exclusive_consumer = &mut *self.guarder.exclusive_consumer;
                //因为准备解锁共享锁状态保证了，同一时间只有一个线程可以获取到锁，所以可以使用不精确的检查接收队列是否为空的检查
                if !exclusive_consumer.try_is_empty() {
                    if let Some(waker) = exclusive_consumer.try_recv() {
                        //有异步独占锁任务等待异步共享锁释放，则唤醒此独占锁任务
                        self.guarder.unlock_shared();
                        waker.wake();
                        return;
                    }
                }
                self.guarder.unlock_shared();
            }
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
        unsafe {
            let shared_producor = &self.guarder.shared_producor;
            let exclusive_consumer = &mut *self.guarder.exclusive_consumer;
            //因为独占锁保证了，同一时间只有一个线程可以获取到锁，所以可以使用不精确的检查接收队列是否为空的检查
            if !shared_producor.try_is_empty() {
                //有异步共享锁任务等待异步独占锁释放，则唤醒所有共享锁任务
                let wakers = shared_producor.take();
                self.guarder.unlock_exclusive();
                for waker in wakers {
                    waker.wake();
                }
                return;
            } else if !exclusive_consumer.try_is_empty() {
                //有异步独占锁任务等待当前异步独占锁释放，则唤醒此独占锁任务
                if let Some(waker) = exclusive_consumer.try_recv() {
                    //有异步独占锁任务等待异步共享锁释放，则唤醒此独占锁任务
                    self.guarder.unlock_exclusive();
                    waker.wake();
                    return;
                }
            }
            self.guarder.unlock_exclusive();
        }
    }
}

/*
* 异步读写锁，支持临界区内执行异步任务等待，不支持重入
*/
pub struct RwLock<T> {
    inner:  Arc<InnerRwLock<T>>,  //内部锁
}

unsafe impl<T> Send for RwLock<T> {}
unsafe impl<T> Sync for RwLock<T> {}

/*
* 异步读写锁同步方法
*/
impl<T> RwLock<T> {
    //构建异步读写锁
    pub fn new(v: T) -> Self {
        let (shared_producor, shared_consumer) = mpsc_deque();
        let (exclusive_producor, exclusive_consumer) = mpsc_deque();
        let inner = Arc::new(InnerRwLock {
            shared_producor,
            shared_consumer: Box::into_raw(Box::new(shared_consumer)),
            exclusive_producor,
            exclusive_consumer: Box::into_raw(Box::new(exclusive_consumer)),
            lock_status: AtomicIsize::new(UNLOCKED),
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
    //获取异步读锁
    pub async fn read(&self) -> RwLockReadGuard<T> {
        FutureShared {
            inner: self.inner.clone(),
        }.await
    }

    //获取异步写锁
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
    shared_producor:    Sender<Waker>,          //共享等待队列生产者
    shared_consumer:    *mut Receiver<Waker>,   //共享等待队列消费者
    exclusive_producor: Sender<Waker>,          //独占等待队列生产者
    exclusive_consumer: *mut Receiver<Waker>,   //独占等待队列消费者
    lock_status:        AtomicIsize,            //异步读写锁状态
    inner:              UnsafeCell<T>,          //异步读写锁内容
}

unsafe impl<T> Send for InnerRwLock<T> {}
unsafe impl<T> Sync for InnerRwLock<T> {}

impl<T> InnerRwLock<T> {
    //加入共享等待队列
    #[inline(always)]
    pub fn push_shared(&self, waker: Waker) {
        self.shared_producor.send(waker);
    }

    //加入独占等待队列
    #[inline(always)]
    pub fn push_exclusive(&self, waker: Waker) {
        self.exclusive_producor.send(waker);
    }

    //获取当前异步读写锁状态
    #[inline(always)]
    pub fn get_lock_status(&self) -> isize {
        self.lock_status.load(Ordering::Relaxed)
    }

    //尝试获取异步共享锁，返回是否成功
    pub fn try_lock_shared(&self) -> bool {
        let mut status = self.get_lock_status();
        if status == EXCLUSIVE {
            //锁当前被独占，则立即返回失败
            return false;
        } else if status == UNLOCK_SHARED {
            //准备解锁共享锁，则将状态更新为未锁，并继续尝试获取共享锁
            status = UNLOCKED;
        }

        loop {
            if let Some(new_status) = status.checked_add(1) {
                match self.lock_status.compare_exchange_weak(status,
                                                             new_status,
                                                             Ordering::Acquire,
                                                             Ordering::Relaxed) {
                    Err(current) if current == EXCLUSIVE => {
                        //锁当前被独占，则立即返回失败
                        return false;
                    },
                    Err(current) if current == UNLOCK_SHARED => {
                        //准备解锁共享锁，则将状态更新为未锁，并立即尝试获取共享锁
                        status = UNLOCKED;
                        continue;
                    },
                    Err(current) => {
                        //共享锁状态不匹配，则更新当前共享锁状态，并立即尝试获取共享锁
                        status = current;
                        continue;
                    },
                    Ok(_) => {
                        //获取共享锁成功，则增加共享锁计数
                        return true;
                    },
                }
            } else {
                //共享锁数量已达限制
                panic!("Shared lock limit");
            }
        }
    }

    //释放异步共享锁，并返回是否准备解锁共享锁
    #[inline(always)]
    pub fn free_shared(&self) -> bool {
        match self.lock_status.compare_exchange_weak(SHARED_ONCE,
                                                     UNLOCK_SHARED,
                                                     Ordering::Acquire,
                                                     Ordering::Relaxed) {
            Err(current) if current <= UNLOCKED => {
                //当前锁状态错误，则立即抛出异常
                panic!("Free shared lock failed, current: {:?}, reason: invalid current status", current);
            },
            Err(_) => {
                //当前不是释放唯一共享锁，则减去共享锁计数，并返回未准备解锁共享锁
                self.lock_status.fetch_sub(1, Ordering::Relaxed);
                false
            },
            Ok(_) => {
                //当前锁满足条件，则返回准备解锁共享锁
                true
            }
        }
    }

    //解锁异步共享锁，返回是否成功
    #[inline(always)]
    pub fn unlock_shared(&self) {
        self.lock_status.store(UNLOCKED, Ordering::Relaxed);
    }

    //尝试获取异步独占锁，返回是否成功
    #[inline(always)]
    pub fn try_lock_exclusive(&self) -> bool {
        self.lock_status.compare_exchange_weak(UNLOCKED,
                                               EXCLUSIVE,
                                               Ordering::Acquire,
                                               Ordering::Relaxed)
            .is_ok()
    }

    //解锁异步独占锁，返回是否成功
    #[inline(always)]
    pub fn unlock_exclusive(&self) {
        self.lock_status.store(UNLOCKED, Ordering::Relaxed);
    }
}

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
        for spin_len in 1..10 {
            if (&self).inner.try_lock_shared() {
                //获取异步共享锁成功，则返回异步读锁守护者
                return Poll::Ready(RwLockReadGuard {
                    guarder: (&self).inner.clone()
                });
            } else {
                //获取异步共享锁失败，则自旋后，再次尝试
                spin(spin_len);
                continue;
            }
        }

        //尝试获取异步共享锁失败，则加入共享等待队列，并立即解锁等待队列的共享锁
        (&self).inner.push_shared(cx.waker().clone());
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
        for spin_len in 1..5 {
            if (&self).inner.try_lock_exclusive() {
                //获取异步独占锁成功，则返回异步写锁守护者
                return Poll::Ready(RwLockWriteGuard {
                    guarder: (&self).inner.clone()
                });
            } else {
                //获取异步独占锁失败，则自旋后，再次尝试
                spin(spin_len);
                continue;
            }
        }

        //尝试获取异步独占锁失败，则加入独占等待队列
        (&self).inner.push_exclusive(cx.waker().clone());
        Poll::Pending
    }
}
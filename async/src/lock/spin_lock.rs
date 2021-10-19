//! # 同步自旋锁，不支持重入
//!

use std::sync::Arc;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

use super::spin;

///
/// 同步自旋锁守护者
///
pub struct SpinLockGuard<T> {
    guarder:  Arc<InnerSpinLock<T>>,  //内部锁
}

impl<T> Deref for SpinLockGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.guarder.inner.get()
        }
    }
}

impl<T> DerefMut for SpinLockGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.guarder.inner.get()
        }
    }
}

impl<T> Drop for SpinLockGuard<T> {
    fn drop(&mut self) {
        self.guarder.status.store(false, Ordering::Relaxed);
    }
}

///
/// 同步自旋锁，不支持临界区内执行异步任务等待，不支持重入
///
pub struct SpinLock<T> {
    inner:  Arc<InnerSpinLock<T>>,  //内部锁
}

unsafe impl<T> Send for SpinLock<T> {}
unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// 构建同步自旋锁
    pub fn new(v: T) -> Self {
        let inner = Arc::new(InnerSpinLock {
            status: AtomicBool::new(false),
            inner: UnsafeCell::new(v),
        });

        SpinLock {
            inner,
        }
    }

    /// 获取同步自旋锁
    #[cfg(not(target_arch = "aarch64"))]
    pub fn lock(&self) -> SpinLockGuard<T> {
        let mut spin_len = 1;
        loop {
            match self.inner.status.compare_exchange_weak(false,
                                                          true,
                                                          Ordering::Acquire,
                                                          Ordering::Relaxed) {
                Err(_) => {
                    //锁失败，则自旋后，继续锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Ok(_) => {
                    return SpinLockGuard {
                        guarder: self.inner.clone(),
                    };
                },
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    pub fn lock(&self) -> SpinLockGuard<T> {
        let mut spin_len = 1;
        loop {
            match self.inner.status.compare_exchange(false,
                                                     true,
                                                     Ordering::Acquire,
                                                     Ordering::Relaxed) {
                Err(_) => {
                    //锁失败，则自旋后，继续锁
                    spin_len = spin(spin_len);
                    continue;
                },
                Ok(_) => {
                    return SpinLockGuard {
                        guarder: self.inner.clone(),
                    };
                },
            }
        }
    }
}

/*
* 内部同步自旋锁
*/
struct InnerSpinLock<T> {
    status: AtomicBool,     //同步自旋锁状态
    inner:  UnsafeCell<T>,  //同步自旋锁内容
}

unsafe impl<T> Send for InnerSpinLock<T> {}
unsafe impl<T> Sync for InnerSpinLock<T> {}
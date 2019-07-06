use std::thread;
use std::boxed::Box;
use std::time::Duration;
use std::marker::PhantomData;
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};

#[cfg(all(feature="unstable", any(target_arch = "x86", target_arch = "x86_64")))]
#[inline(always)]
pub fn pause() {
    unsafe { asm!("PAUSE") };
}

#[cfg(all(not(feature="unstable"), any(target_arch = "x86", target_arch = "x86_64")))]
#[inline(always)]
pub fn pause() {
    thread::sleep(Duration::from_millis(1));
}

#[cfg(all(not(target_arch = "x86"), not(target_arch = "x86_64")))]
#[inline(always)]
pub fn pause() {
    thread::sleep(Duration::from_millis(1));
}

/*
* 无锁栈帧，包括值和下一个栈帧的指针
*/
struct LFStackFrame<T: 'static>(T, usize);

/*
* 无锁栈
*/
pub struct LFStack<T: 'static> {
    lock:       AtomicBool,     //原子锁
    top:        AtomicUsize,    //栈顶
    phantom:    PhantomData<T>,
}

impl<T> Drop for LFStack<T> {
    fn drop(&mut self) {
        //释放所有栈帧
        let mut top: usize = self.top.load(Ordering::Relaxed);
        let mut frame;
        loop {
            unsafe {
                if top == 0 {
                    //释放所有栈帧完成
                    break;
                }

                frame = Box::from_raw(top as *mut LFStackFrame<T>);
                top = (*frame).1;
            }
        }
    }
}

impl<T: 'static> LFStack<T> {
    //构建一个指定容量的无锁栈
    pub fn new() -> Self {
        LFStack {
            lock: AtomicBool::new(false),
            top: AtomicUsize::new(0),
            phantom: PhantomData,
        }
    }

    //获取当前栈高度
    pub fn size(&self) -> usize {
        let mut size = 0;
        loop {
            match self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
                true => {
                    pause();
                },
                false => {
                    let mut sp = self.top.load(Ordering::Relaxed);
                    loop {
                        if sp == 0 {
                            //获取完成，则释放原子锁
                            self.lock.store(false, Ordering::SeqCst);
                            return size;
                        } else {
                            unsafe {
                                let frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                size += 1;
                                sp = (*frame).1; //获取后续栈指针
                                Box::into_raw(frame); //将当前栈帧转化为栈指针，防止被回收
                            }
                        }
                    }
                }
            }
        }
    }

    //将值弹出栈顶
    pub fn pop(&self) -> Option<T> {
        loop {
            match self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
                true => {
                    pause();
                },
                false => {
                    let top = self.top.load(Ordering::Relaxed);
                    if top == 0 {
                        //空栈
                        self.lock.store(false, Ordering::SeqCst); //释放原子锁
                        return None;
                    } else {
                        unsafe {
                            let top_frame = Box::from_raw(top as *mut LFStackFrame<T>);
                            self.top.store(top_frame.1, Ordering::Relaxed);
                            self.lock.store(false, Ordering::SeqCst); //释放原子锁
                            return Some(top_frame.0);
                        }
                    }
                }
            }
        }
    }

    //将值推入栈顶
    pub fn push(&self, val: T) {
        loop {
            match self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
                true => {
                    pause();
                },
                false => {
                    let last_top = self.top.load(Ordering::Relaxed);
                    let top_frame = Box::into_raw(Box::new(LFStackFrame(val, last_top)));
                    self.top.store(top_frame as usize, Ordering::Relaxed);
                    self.lock.store(false, Ordering::SeqCst); //释放原子锁
                    return;
                }
            }
        }
    }

    //阻塞整理栈
    pub fn collect(&self, handler: Arc<Fn(&mut T) -> bool>) {
        loop {
            match self.lock.compare_and_swap(false, true, Ordering::SeqCst) {
                true => {
                    pause();
                },
                false => {
                    let mut sp = self.top.load(Ordering::Relaxed);
                    let mut last_sp = 0;

                    loop {
                        if sp == 0 {
                            //整理结束，释放原子锁
                            self.lock.store(false, Ordering::SeqCst);
                            return;
                        } else {
                            unsafe {
                                let mut frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                if handler(&mut (*frame).0) {
                                    //需要移除当前栈帧
                                    if last_sp == 0 {
                                        //当前栈帧为首帧
                                        sp = (*frame).1; //记录当前栈帧的后续栈指针
                                        self.top.store(sp, Ordering::Relaxed); //将当前栈帧的后续栈指针设置为栈顶指针
                                    } else {
                                        //当前栈帧为后续帧
                                        let mut last_frame = Box::from_raw(last_sp as *mut LFStackFrame<T>);
                                        sp = (*frame).1; //记录当前栈帧的后续栈指针
                                        (*last_frame).1 = sp; //将上一个栈帧的后续栈指针设置为当前栈帧的后续栈指针
                                        Box::into_raw(last_frame); //将上一个栈帧转化为栈指针，防止被回收
                                    }
                                } else {
                                    //忽略当前栈帧
                                    last_sp = sp; //记录当前栈指针
                                    sp = frame.1; //记录下一个栈指针
                                    Box::into_raw(frame); //将当前栈帧转化为栈指针，防止被回收
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
extern crate crossbeam_channel;

use std::thread;
use std::boxed::Box;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};

use crossbeam_channel::{Sender, Receiver, unbounded};

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
* 无锁栈帧，包括值，上一个栈帧的指针和下一个栈帧的指针，指针为0表示已到头或尾
*/
struct LFStackFrame<T: 'static>(T, usize, usize);

/*
* 整理返回值
*/
pub enum CollectResult {
    Break(bool),    //中止整理，并确定是否移除当前被整理的栈帧
    Continue(bool), //继续整理，并确定是否移除当前被整理的栈帧
}

/*
* 无锁栈
*/
pub struct LFStack<T: 'static> {
    lock:       AtomicBool,                     //原子锁，原子锁的排序必须是SeqCst
    top:        AtomicUsize,                    //栈顶，排序只需要使用Relaxed
    bottom:     AtomicUsize,                    //栈底，排序只需要使用Relaxed
    high:       AtomicUsize,                    //栈高，排序只需要使用Relaxed
    wait_sent:  Sender<Box<LFStackFrame<T>>>,   //待清理的栈帧缓冲发送器
    wait_recv:  Receiver<Box<LFStackFrame<T>>>, //待清理的栈帧缓冲接收器
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
        let (wait_sent, wait_recv) = unbounded();
        LFStack {
            lock: AtomicBool::new(false),
            top: AtomicUsize::new(0),
            bottom: AtomicUsize::new(0),
            high: AtomicUsize::new(0),
            wait_sent,
            wait_recv,
        }
    }

    //获取当前栈高度
    pub fn size(&self) -> usize {
        self.high.load(Ordering::Relaxed)
    }

    //判断当前栈是否被锁住
    pub fn is_lock(&self) -> bool {
        self.lock.load(Ordering::SeqCst)
    }

    //将值弹出栈顶
    pub fn pop(&self) -> Option<T> {
        loop {
            match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(r) => match r {
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
    
                                //设置新的栈顶
                                self.top.store(top_frame.2, Ordering::Relaxed);
    
                                if top_frame.2 == 0 {
                                    //尾帧，则将栈底设置为空
                                    self.bottom.store(0, Ordering::Relaxed);
                                } else {
                                    //当前帧不是尾帧，则设置下一个顶帧的前继帧指针为空
                                    let mut last_frame = Box::from_raw(top_frame.2 as *mut LFStackFrame<T>);
                                    (*last_frame).1 = 0;
                                    Box::into_raw(last_frame); //将下一个顶帧转化为栈指针，防止被移除
                                }
    
                                self.high.fetch_sub(1, Ordering::Relaxed); //减少栈高度
    
                                self.lock.store(false, Ordering::SeqCst); //释放原子锁
                                return Some(top_frame.0);
                            }
                        }
                    }
                },
                _ => continue,
                
            }
        }
    }

    //将值尝试弹出栈顶
    pub fn try_pop(&self) -> Result<T, Option<()>> {
        let mut count = 5; //最大尝试次数
        while count > 0 {
            match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(r) => match r {
                    true => {
                        count -= 1;
                        pause();
                    },
                    false => {
                        let top = self.top.load(Ordering::Relaxed);
                        if top == 0 {
                            //空栈
                            self.lock.store(false, Ordering::SeqCst); //释放原子锁
                            return Err(Some(())); //返回没有值
                        } else {
                            unsafe {
                                let top_frame = Box::from_raw(top as *mut LFStackFrame<T>);
    
                                //设置新的栈顶
                                self.top.store(top_frame.2, Ordering::Relaxed);
    
                                if top_frame.2 == 0 {
                                    //尾帧，则将栈底设置为空
                                    self.bottom.store(0, Ordering::Relaxed);
                                } else {
                                    //当前帧不是尾帧，则设置下一个顶帧的前继帧指针为空
                                    let mut last_frame = Box::from_raw(top_frame.2 as *mut LFStackFrame<T>);
                                    (*last_frame).1 = 0;
                                    Box::into_raw(last_frame); //将下一个顶帧转化为栈指针，防止被移除
                                }
    
                                self.high.fetch_sub(1, Ordering::Relaxed); //减少栈高度
    
                                self.lock.store(false, Ordering::SeqCst); //释放原子锁
                                return Ok(top_frame.0); //返回值
                            }
                        }
                    }
                },
                _ => continue,
                
            }
        }

        Err(None) //返回弹出失败
    }

    //将值推入栈顶
    pub fn push(&self, val: T) {
        loop {
            match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(r) => match r{
                    true => {
                        pause();
                    },
                    false => {
                        let last_top = self.top.load(Ordering::Relaxed); //获取上一个顶帧
                        let top_frame = Box::into_raw(Box::new(LFStackFrame(val, 0, last_top))); //构建新的顶帧
    
                        //设置新的栈顶
                        self.top.store(top_frame as usize, Ordering::Relaxed);
    
                        if last_top == 0 {
                            //首个帧，则将栈底设置为首帧
                            self.bottom.store(top_frame as usize, Ordering::Relaxed);
                        } else {
                            //当前帧不是首个帧，则设置上一个顶帧的前继帧指针为当前帧指针
                            unsafe {
                                let mut last_frame = Box::from_raw(last_top as *mut LFStackFrame<T>);
                                (*last_frame).1 = top_frame as usize;
                                Box::into_raw(last_frame); //将上一个顶帧转化为栈指针，防止被移除
                            }
                        }
    
                        self.high.fetch_add(1, Ordering::Relaxed); //增加栈高度
    
                        self.lock.store(false, Ordering::SeqCst); //释放原子锁
                        return;
                    }
                },
                _ => continue,
            }
        }
    }

    //将值推入栈顶
    pub fn try_push(&self, val: T) -> Result<(), ()> {
        let mut count = 5; //最大尝试次数
        while count > 0 {
            match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(r) => match r {
                    true => {
                        count -= 1;
                        pause();
                    },
                    false => {
                        let last_top = self.top.load(Ordering::Relaxed); //获取上一个顶帧
                        let top_frame = Box::into_raw(Box::new(LFStackFrame(val, 0, last_top))); //构建新的顶帧
    
                        //设置新的栈顶
                        self.top.store(top_frame as usize, Ordering::Relaxed);
    
                        if last_top == 0 {
                            //首个帧，则将栈底设置为首帧
                            self.bottom.store(top_frame as usize, Ordering::Relaxed);
                        } else {
                            //当前帧不是首个帧，则设置上一个顶帧的前继帧指针为当前帧指针
                            unsafe {
                                let mut last_frame = Box::from_raw(last_top as *mut LFStackFrame<T>);
                                (*last_frame).1 = top_frame as usize;
                                Box::into_raw(last_frame); //将上一个顶帧转化为栈指针，防止被移除
                            }
                        }
    
                        self.high.fetch_add(1, Ordering::Relaxed); //增加栈高度
    
                        self.lock.store(false, Ordering::SeqCst); //释放原子锁
                        return Ok(()); //返回推入成功
                    }
                },
                _ => continue,
            }
        }

        Err(()) //返回推入失败
    }

    //阻塞整理栈，从顶到底，处理器返回None，表示中止整理，返回Some(true)表示移除当前帧，并继续整理，返回Some(false)表示忽略当前帧，并继续整理
    pub fn collect_from_top(&self, handler: Arc<dyn Fn(&mut T) -> CollectResult>) {
        loop {
            match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(r) => match r {
                    true => {
                        pause();
                    },
                    false => {
                        let mut sp = self.top.load(Ordering::Relaxed);
                        let mut last_sp = 0;
    
                        loop {
                            if sp == 0 {
                                //遍历栈结束，已移除所有值，则释放原子锁
                                self.lock.store(false, Ordering::SeqCst);
                                return;
                            } else {
                                unsafe {
                                    let mut frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                    match handler(&mut (*frame).0) {
                                        CollectResult::Continue(true) => {
                                            //需要移除当前栈帧，并继续整理
                                            if last_sp == 0 {
                                                //当前栈帧为顶帧
                                                sp = (*frame).2; //记录当前栈帧的后续栈指针
                                                self.top.store(sp, Ordering::Relaxed); //将当前栈帧的后续栈指针设置为栈顶指针
    
                                                if sp == 0 {
                                                    //已移除所有值，则设置栈底指针为空
                                                    self.bottom.store(0, Ordering::Relaxed);
                                                } else {
                                                    //将下一个栈帧的前继栈指针设置为空
                                                    let mut next_frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                                    (*next_frame).1 = 0;
    
                                                    //将下一个栈帧转化为栈指针，防止被移除
                                                    Box::into_raw(next_frame);
                                                }
                                            } else {
                                                //当前栈帧为后续帧，即上一个栈帧没有移除
                                                let mut last_frame = Box::from_raw(last_sp as *mut LFStackFrame<T>);
                                                sp = (*frame).2; //记录当前栈帧的后续栈指针
                                                (*last_frame).2 = sp; //将上一个栈帧的后续栈指针设置为当前栈帧的后续栈指针
    
                                                if sp > 0 {
                                                    //将下一个栈帧的前继栈指针设置为上一个栈帧的指针
                                                    let mut next_frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                                    (*next_frame).1 = last_sp;
                                                    Box::into_raw(next_frame); //将下一个栈帧转化为栈指针，防止被移除
                                                }
    
                                                //将上一个栈帧转化为栈指针，防止被移除
                                                Box::into_raw(last_frame);
                                            }
    
                                            self.high.fetch_sub(1, Ordering::Relaxed); //减少栈高度
    
                                            self.wait_sent.send(frame); //放入待清理缓冲区
                                        },
                                        CollectResult::Continue(false) => {
                                            //忽略当前栈帧，并继续整理
                                            last_sp = sp; //记录当前栈指针
                                            sp = frame.2; //记录后继栈指针
                                            Box::into_raw(frame); //将当前栈帧转化为栈指针，防止被移除
                                        },
                                        CollectResult::Break(true) => {
                                            //需要移除当前栈帧，并立即中止整理
                                            if last_sp == 0 {
                                                //当前栈帧为顶帧
                                                sp = (*frame).2; //记录当前栈帧的后续栈指针
                                                self.top.store(sp, Ordering::Relaxed); //将当前栈帧的后续栈指针设置为栈顶指针
    
                                                if sp == 0 {
                                                    //已移除所有值，则设置栈底指针为空
                                                    self.bottom.store(0, Ordering::Relaxed);
                                                } else {
                                                    //将下一个栈帧的前继栈指针设置为空
                                                    let mut next_frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                                    (*next_frame).1 = 0;
    
                                                    //将下一个栈帧转化为栈指针，防止被移除
                                                    Box::into_raw(next_frame);
                                                }
                                            } else {
                                                //当前栈帧为后续帧，即上一个栈帧没有移除
                                                let mut last_frame = Box::from_raw(last_sp as *mut LFStackFrame<T>);
                                                (*last_frame).2 = (*frame).2; //将上一个栈帧的后续栈指针设置为当前栈帧的后续栈指针
    
                                                if (*frame).2 > 0 {
                                                    //将下一个栈帧的前继栈指针设置为上一个栈帧的指针
                                                    let mut next_frame = Box::from_raw((*frame).2 as *mut LFStackFrame<T>);
                                                    (*next_frame).1 = last_sp;
                                                    Box::into_raw(next_frame); //将下一个栈帧转化为栈指针，防止被移除
                                                }
    
                                                //将上一个栈帧转化为栈指针，防止被移除
                                                Box::into_raw(last_frame);
                                            }
    
                                            self.high.fetch_sub(1, Ordering::Relaxed); //减少栈高度
    
                                            self.wait_sent.send(frame); //放入待清理缓冲区
    
                                            self.lock.store(false, Ordering::SeqCst); //释放原子锁
                                            return;
                                        },
                                        CollectResult::Break(false) => {
                                            //忽略当前栈帧，并立即中止整理
                                            Box::into_raw(frame); //将当前栈帧转化为栈指针，防止被移除
                                            self.lock.store(false, Ordering::SeqCst); //释放原子锁
                                            return;
                                        },
                                    }
                                }
                            }
                        }
                    }
                },
                _ => continue,
                
            }
        }
    }

    //阻塞整理栈，从底到顶，处理器返回None，表示中止整理，返回Some(true)表示移除当前帧，并继续整理，返回Some(false)表示忽略当前帧，并继续整理
    pub fn collect_from_bottom(&self, handler: Arc<dyn Fn(&mut T) -> CollectResult>) {
        loop {
            match self.lock.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(r) => match r {
                    true => {
                        pause();
                    },
                    false => {
                        let mut sp = self.bottom.load(Ordering::Relaxed);
                        let mut last_sp = 0;
    
                        loop {
                            if sp == 0 {
                                //遍历栈结束，已移除所有值，则释放原子锁
                                self.lock.store(false, Ordering::SeqCst);
                                return;
                            } else {
                                unsafe {
                                    let mut frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                    match handler(&mut (*frame).0) {
                                        CollectResult::Continue(true) => {
                                            //需要移除当前栈帧，并继续整理
                                            if last_sp == 0 {
                                                //当前栈帧为尾帧
                                                sp = (*frame).1; //记录当前栈帧的前继栈指针
                                                self.bottom.store(sp, Ordering::Relaxed); //将当前栈帧的前继栈指针设置为栈底指针
    
                                                if sp == 0 {
                                                    //已移除所有值，则设置栈顶指针为空
                                                    self.top.store(0, Ordering::Relaxed);
                                                } else {
                                                    //将下一个栈帧的后继栈指针设置为空
                                                    let mut next_frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                                    (*next_frame).2 = 0;
    
                                                    //将下一个栈帧转化为栈指针，防止被移除
                                                    Box::into_raw(next_frame);
                                                }
                                            } else {
                                                //当前栈帧为前继帧，即上一个栈帧没有移除
                                                let mut last_frame = Box::from_raw(last_sp as *mut LFStackFrame<T>);
                                                sp = (*frame).1; //记录当前栈帧的前继栈指针
                                                (*last_frame).1 = sp; //将上一个栈帧的前继栈指针设置为当前栈帧的前继栈指针
    
                                                if sp > 0 {
                                                    //将下一个栈帧的后继栈指针设置为上一个栈帧的指针
                                                    let mut next_frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                                    (*next_frame).2 = last_sp;
                                                    Box::into_raw(next_frame); //将下一个栈帧转化为栈指针，防止被移除
                                                }
    
                                                //将上一个栈帧转化为栈指针，防止被移除
                                                Box::into_raw(last_frame);
                                            }
    
                                            self.high.fetch_sub(1, Ordering::Relaxed); //减少栈高度
    
                                            self.wait_sent.send(frame); //放入待清理缓冲区
                                        },
                                        CollectResult::Continue(false) => {
                                            //忽略当前栈帧，并继续整理
                                            last_sp = sp; //记录当前栈指针
                                            sp = frame.1; //记录前继栈指针
                                            Box::into_raw(frame); //将当前栈帧转化为栈指针，防止被移除
                                        },
                                        CollectResult::Break(true) => {
                                            //需要移除当前栈帧，并立即中止整理
                                            if last_sp == 0 {
                                                //当前栈帧为尾帧
                                                sp = (*frame).1; //记录当前栈帧的前继栈指针
                                                self.bottom.store(sp, Ordering::Relaxed); //将当前栈帧的前继栈指针设置为栈底指针
    
                                                if sp == 0 {
                                                    //已移除所有值，则设置栈顶指针为空
                                                    self.top.store(0, Ordering::Relaxed);
                                                } else {
                                                    //将下一个栈帧的后继栈指针设置为空
                                                    let mut next_frame = Box::from_raw(sp as *mut LFStackFrame<T>);
                                                    (*next_frame).2 = 0;
    
                                                    //将下一个栈帧转化为栈指针，防止被移除
                                                    Box::into_raw(next_frame);
                                                }
                                            } else {
                                                //当前栈帧为前继帧，即上一个栈帧没有移除
                                                let mut last_frame = Box::from_raw(last_sp as *mut LFStackFrame<T>);
                                                (*last_frame).1 = (*frame).1; //将上一个栈帧的前继栈指针设置为当前栈帧的前继栈指针
    
                                                if (*frame).1 > 0 {
                                                    //将下一个栈帧的后继栈指针设置为上一个栈帧的指针
                                                    let mut next_frame = Box::from_raw((*frame).1 as *mut LFStackFrame<T>);
                                                    (*next_frame).2 = last_sp;
                                                    Box::into_raw(next_frame); //将下一个栈帧转化为栈指针，防止被移除
                                                }
    
                                                //将上一个栈帧转化为栈指针，防止被移除
                                                Box::into_raw(last_frame);
                                            }
    
                                            self.high.fetch_sub(1, Ordering::Relaxed); //减少栈高度
    
                                            self.wait_sent.send(frame); //放入待清理缓冲区
    
                                            self.lock.store(false, Ordering::SeqCst); //释放原子锁
                                            return;
                                        },
                                        CollectResult::Break(false) => {
                                            //忽略当前栈帧，并立即中止整理
                                            Box::into_raw(frame); //将当前栈帧转化为栈指针，防止被移除
                                            self.lock.store(false, Ordering::SeqCst); //释放原子锁
                                            return;
                                        },
                                    }
                                }
                            }
                        }
                    }
                },
                _ => continue,
            }
        }
    }

    //清空待清理的栈帧
    pub fn clear(&self) {
        self.wait_recv.try_iter().collect::<Vec<Box<LFStackFrame<T>>>>();
    }
}

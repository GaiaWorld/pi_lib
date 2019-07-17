use std::alloc::{System, GlobalAlloc, Layout};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::time::Instant;
use std::f32::MAX;

use common::SysStat;

/*
* 多余的空闲内存上限，单位B，默认512MB
*/
pub const FREE_SYSTEM_MEMORY_MAX_LIMIT: u64 = 536870912;

/*
* 当前虚拟机已分配内存
*/
pub static VM_ALLOCATED: AtomicIsize = AtomicIsize::new(0);

/*
* 当前最大已分配内存限制，默认8GB
*/
static MAX_ALLOCATED_LIMIT: AtomicUsize = AtomicUsize::new(8589934590);

/*
* 当前已分配内存
*/
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

/*
* 计数的系统内存分配器
*/
pub struct CounterSystemAllocator;

unsafe impl GlobalAlloc for CounterSystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        return ret;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

/*
* 获取当前最大已分配内存限制
*/
#[inline]
pub fn get_max_alloced_limit() -> usize {
    MAX_ALLOCATED_LIMIT.load(Ordering::Relaxed)
}

/*
* 设置当前最大已分配内存限制，必须大于当前所有已分配内存，成功返回上次设置的最大已分配内存限制
*/
#[inline]
pub fn set_max_alloced_limit(limit: usize) -> Result<usize, ()> {
    if limit <= all_alloced_size() {
        return Err(());
    }

    Ok(MAX_ALLOCATED_LIMIT.swap(limit, Ordering::SeqCst))
}

/*
* 检查当前所有已分配内存是否已达当前最大已分配内存限制
*/
#[inline]
pub fn is_alloced_limit() -> bool {
    let max_alloced_limit = get_max_alloced_limit();

    (max_alloced_limit > 0) && all_alloced_size() >= max_alloced_limit
}

/*
* 获取当前已分配内存数量，单位B
*/
#[inline]
pub fn alloced_size() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

/*
* 获取虚拟机已分配内存数量，单位B
*/
#[inline]
pub fn vm_alloced_size() -> isize {
    VM_ALLOCATED.load(Ordering::SeqCst)
}

/*
* 获取当前已分配和虚拟机已分配内存数量，单位B
*/
#[inline]
pub fn all_alloced_size() -> usize {
    let vm_size = vm_alloced_size();
    if vm_size < 0 {
        alloced_size()
    } else {
        alloced_size() + vm_size as usize
    }
}

//线程安全的回收多余的空闲系统内存
#[cfg(any(windows))]
pub fn free_sys_mem(_: u64) -> bool {
    true
}

//线程安全的回收多余的空闲系统内存
#[cfg(any(unix))]
pub fn free_sys_mem(limit: u64) -> bool {
    let start_time = Instant::now();
    let sys = SysStat::new().special_platform().unwrap();
    let pid = sys.process_current_pid();
    let current = all_alloced_size();
    if let Some((_, _, res, _, _, _)) = sys.process_memory(pid) {
        if let Some(sub) = res.checked_sub(current as u64) {
            if sub >= limit {
                //多余的空闲内存已达限制，则回收多余的空闲内存
                unsafe {
                    if dukc_manual_free() == 0 {
                        println!("===> Collect System Memory Ok, Free Failed, current: {}, before real: {}, after real: {}, limit: {}, time: {:?}", current, res, after_res, limit, Instant::now() - start_time);
                        false
                    } else {
                        let after_res = if let Some((_, _, after, _, _, _)) = sys.process_memory(pid) {
                            after
                        } else {
                            0
                        };
                        println!("===> Collect System Memory Ok, Free Ok, current: {}, before real: {}, after real: {}, limit: {}, time: {:?}", current, res, after_res, limit, Instant::now() - start_time);
                        true
                    }
                }
            } else {
                //多余的空闲内存未达限制，则忽略
                false
            }
        } else {
            //内存占用异常，则回收失败
            println!("!!!> Collect System Memory Failed, current: {}, real: {}, limit: {}", current, res, limit);
            false
        }
    } else {
        //获取不到当前进程内存占用，则回收失败
        false
    }
}
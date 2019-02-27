use std::alloc::{System, GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

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
* 获取当前已分配内存数量，单位B
*/
#[inline]
pub fn alloced_size() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}
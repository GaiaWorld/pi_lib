/**
 * 全局的线程安全的常量字符串池，为了移植问题，可能需要将实现部分移到其他库
 */

use std::cmp::Ord;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;
use std::vec;
use std::mem::size_of;
use std::usize::MAX;
use std::marker::PhantomData;
//use std::marker::Copy;


// https://amanieu.github.io/parking_lot/parking_lot/struct.RwLock.html
// 高性能的支持升级的读写锁
// 同步原语，可用于运行一次性初始化。用于全局，FFI或相关功能的一次初始化。


// 常量字符串
pub type Str = Arc<String>;

// 可能实现比较大小、hash等方法

// 
// static map : RwLock<HashMap<u64, (Weak<String>, Option<Vec<Weak<String>>>)>> = HashMap::new();

pub fn check(s: &String) -> bool {
	return true
}
pub fn get(s: String) -> Str {
	loop {
		// 先读锁，然后升级成写锁，如果升级失败则放弃读锁重新循环
		return Arc::new(s)
	}
}
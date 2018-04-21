/**
 * 全局的线程安全的原子字符串池，为了移植问题，可能需要将实现部分移到其他库
 */

use std::ops::Deref;
use std::sync::Arc;
//use std::marker::Copy;
use core::convert::From;
use std::hash::{Hash, Hasher};

// https://amanieu.github.io/parking_lot/parking_lot/struct.RwLock.html
// 高性能的支持升级的读写锁
// 同步原语，可用于运行一次性初始化。用于全局，FFI或相关功能的一次初始化。


// 原子字符串
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct Atom(Arc<(String, u64)>);

impl Deref for Atom {
	type Target = String;
	fn deref(&self) -> &String {
		&(*self.0).0
	}
}

impl Atom {
	// 返回的正整数为0表示静态原子，1表示为动态原子
	fn contain(s: Option<&String>, h: u64) -> Option<usize> {
		return None
	}
	fn get_hash(&self) -> u64 {
		(*self.0).1
	}
}

// impl Hash for Atom {
// 	#[inline]
// 	fn hash<H: Hasher>(&self, state: &mut H) {
// 		(*self.0).1.hash(state)
// 	}
// }

// 为静态编译的完美hash的字符串准备的常量数组
// const NB_BUCKETS: usize = 1 << 12;  // 4096
// const BUCKET_MASK: u64 = (1 << 12) - 1;

// struct StringCache {
//     buckets: [Option<Box<(String, u64)>>; NB_BUCKETS],
// }

// lazy_static! {
//     static ref STRING_CACHE: Mutex<StringCache> = Mutex::new(StringCache::new());
// }

// 为动态的原子字符串准备的fnv hashmap 及可升级的rwlock(如果使用CowList, 就可以不需要，改成先读1次，然后再写1次)
// static map : RwLock<HashMap<u64, (version, CowList<(Weak<(String, u64))>>> = HashMap::new();




impl From<String> for Atom {
	#[inline]
	fn from(s: String) -> Atom {
		Atom(Arc::new((s, 0)))
	}
}
impl<'a> From<&'a str> for Atom {
	#[inline]
	fn from(s: &str) -> Atom {
		Atom(Arc::new((String::from(s), 0)))
	}
}
impl From<Vec<u8>> for Atom {
	#[inline]
	fn from(s: Vec<u8>) -> Atom {
		Atom(Arc::new((unsafe { String::from_utf8_unchecked(s) }, 0)))
	}
}
impl<'a> From<&'a [u8]> for Atom {
	#[inline]
	fn from(s: &[u8]) -> Atom {
		Atom(Arc::new((unsafe { String::from_utf8_unchecked(Vec::from(s)) }, 0)))
	}
}
// 为完美hash准备的方法
// impl From<u64> for Atom {
// 	#[inline]
// 	fn from(s: String) -> Atom {
// 		(Arc::new((s, 0)))
// 	}
// }
fn from(s: String) -> Atom {
	//loop {
		// 先读锁，然后升级成写锁，如果升级失败则放弃读锁重新循环
		Atom(Arc::new((s, 0)))
	//}
}
/**
 * 全局的线程安全的原子字符串池，为了移植问题，可能需要将实现部分移到其他库
 */

use std::ops::Deref;
//use std::marker::Copy;
use core::convert::From;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use bon::{Encode, Decode, BonBuffer};
use std::sync::{Arc, Weak};
use std::sync::RwLock;
use cowlist::CowList;
use fnv::FnvHashMap;
// https://amanieu.github.io/parking_lot/parking_lot/struct.RwLock.html
// 高性能的支持升级的读写锁
// 同步原语，可用于运行一次性初始化。用于全局，FFI或相关功能的一次初始化。

// 为动态的原子字符串准备的fnv hashmap 及可升级的rwlock(如果使用CowList, 就可以不需要可升级的rwlock，改成先读1次，然后再写1次)
lazy_static! {
	static ref ATOM_MAP: Table = Table(RwLock::new(FnvHashMap::default()));
}

// 原子字符串
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom(Arc<(String, u64)>);

impl Deref for Atom {
	type Target = String;
	fn deref(&self) -> &String {
		&(*self.0).0
	}
}

impl Atom {
	// 返回的正整数为0表示静态原子，1表示为动态原子
	// fn contain(s: Option<&String>, h: u64) -> Option<usize> {
	// 	return None
	// }
	pub fn get_hash(&self) -> u64 {
		(*self.0).1
	}
}

impl From<String> for Atom {
	#[inline]
	fn from(s: String) -> Atom {
		Atom(ATOM_MAP.or_insert(s))
	}
}

impl<'a> From<&'a str> for Atom {
	#[inline]
	fn from(s: &str) -> Atom {
		Atom(ATOM_MAP.or_insert(String::from(s)))
	}
}

impl From<Vec<u8>> for Atom {
	#[inline]
	fn from(s: Vec<u8>) -> Atom {
		Atom(ATOM_MAP.or_insert(unsafe { String::from_utf8_unchecked(s) }))
	}
}

impl<'a> From<&'a [u8]> for Atom {
	#[inline]
	fn from(s: &[u8]) -> Atom {
		Atom(ATOM_MAP.or_insert(unsafe { String::from_utf8_unchecked(Vec::from(s)) }))
	}
}

impl Encode for Atom{
	fn encode(&self, bb: &mut BonBuffer){
		(*self.0).0.encode(bb);
		(*self.0).1.encode(bb);
	}
}

impl Decode for Atom{
	fn decode(bb: &mut BonBuffer) -> Atom{
		Atom(Arc::new((String::decode(bb), u64::decode(bb))))
	}
}

//todo
// 为完美hash准备的方法
// impl From<u64> for Atom {
// 	#[inline]
// 	fn from(s: String) -> Atom {
// 		(Arc::new((s, 0)))
// 	}
// }
// fn from(s: String) -> Atom {
// 	//loop {
// 		// 先读锁，然后升级成写锁，如果升级失败则放弃读锁重新循环
// 		Atom(Arc::new((s, 0)))
// 	//}
// }

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

// impl StringCache{
// 	pub fn new() -> StringCache{

// 	}
// }
// lazy_static! {
//     static ref STRING_CACHE: Mutex<StringCache> = Mutex::new(StringCache::new());
// }

struct Table(RwLock<FnvHashMap<u64, (usize, CowList<Weak<(String, u64)>>)>>);

impl Table{
	pub fn or_insert(&self, s: String) -> Arc<(String, u64)>{
		let h = str_hash(&s, &mut DefaultHasher::new());

		//读
		let list = {
			let map = self.0.read().unwrap();
			match map.get(&h){
				Some(v) => Some((v.0, v.1.clone())),
				None => None,
			}
		};
		let (version, mut has_nil) = match list {
			Some((ver, cow)) => {
				let mut nil = false;
				match read(&cow, &s, &mut nil){
					Some(r) => return r,
					None => (ver, nil)
				}
			},
			None => (0, false)
		};


		//如果未读到,取到写锁
		let strong = Arc::new((s, h));
		let mut is_end = false;
		let mut map = self.0.write().unwrap();

		//map中不存在为h的key，证明版本未更新，注解插入当前值，并返回强引用
		let list = map.entry(h).or_insert_with(||{
			is_end = true;
			(1, CowList::new(Arc::downgrade(&strong)))
		} );
		if is_end {
			return strong
		}

		//否则， mpa[h]存在， 算出版本差
		let mut diff = list.0 - version;
		//如果版本差不为0， 应该再次读
		if diff != 0{
			has_nil = false;
			for v in list.1.iter(){
				let v1 = v.upgrade();
				match v1 {
					Some(r) => return r,
					None => (),
				};
				diff -= 1;
				if diff == 0{
					break;
				}
			}
		}

		//如果存在无效弱引用，应该删除
		if has_nil == true{
			// list.1.iter_mut().filter(|item|{
			// 	let strong = item.upgrade();
			// 	match strong {
			// 		Some(v) => false,
			// 		None => true,
			// 	}
			// });
		}

		//第二次读取失败,需要写入
		list.1 = list.1.push(Arc::downgrade(&strong));
		list.0 += 1;
		return strong;
	}

}

fn str_hash<T: Hasher>(s: &str, haser: &mut T) -> u64{
	s.hash(haser);
	haser.finish()
}

fn read(list: &CowList<Weak<(String, u64)>>, s: &str, has_nil: &mut bool) -> Option<Arc<(String, u64)>>{
	for o in list.iter(){
		let strong = o.upgrade();
		match strong {
			Some(o) => {
				if o.0 == s{
					return Some(o);
				}
			},
			None => *has_nil = true,
		};
	}
	None
}


#[test]
fn test_atom() {
    let at1 = Atom::from("abc");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 1);
	let at2 = Atom::from("afg");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 2);
	let at3 = Atom::from("afg");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 2);
	assert_eq!((at3.0).0, "afg");
}
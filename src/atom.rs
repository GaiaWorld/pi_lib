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
		let mut state = CowState{
			version:0,
			has_nil:false,
		};

		//读
		let r = self.read(self.0.read().expect("").deref(), &h, &s, &mut state);
		if r.is_some(){
			return r.unwrap();
		}

		//如果未读到,需要写
		let mut map = self.0.write().expect("");
		let list;
		let strong;

		//算出版本差
		match map.get(&h) {
			Some(l) => {
				let diff = l.0 - state.version;
				//如果版本差不为0， 应该再次读
				if diff != 0{
					state.has_nil = false;
					let r = self.read(map.deref(), &h, &s, &mut state);
					if r.is_some(){
						return r.unwrap()
					}
				}

				list = map.get_mut(&h).unwrap();
				//第二次读取失败,需要写入
				strong = Arc::new((s, h));
				list.1 = list.1.push(Arc::downgrade(&strong));
			},
			None => {
				strong = Arc::new((s, h));
				list = map.entry(h).or_insert((1,CowList::new(Arc::downgrade(&strong)))); //如果是None， 证明版本未更新（前提：插入CowList后，永不删除）
			},
		}

		//如果存在无效弱引用，应该删除
		if state.has_nil == true{
			// list.1.iter_mut().filter(|item|{
			// 	let strong = item.upgrade();
			// 	match strong {
			// 		Some(v) => false,
			// 		None => true,
			// 	}
			// });
		}

		list.0 += 1;
		strong
	}

	pub fn read(&self, map: &FnvHashMap<u64, (usize, CowList<Weak<(String, u64)>>)>, h: &u64, s: &str, state: &mut CowState) -> Option<Arc<(String, u64)>>{
		let r = map.get(h);
		match r{
			Some(v) => {
				for o in v.1.iter(){
					let strong = o.upgrade();
					match strong {
						Some(o) => {
							if o.0 == s{
								return Some(o);
							}
						},
						None => state.has_nil = true,
					};
				}
				state.version = v.0;
			},
			None => state.version = 0,
		}
		None
	}
}

struct CowState{
	pub version: usize,
	pub has_nil: bool,//是否存在空壳（即弱引用无效）
}

fn str_hash<T: Hasher>(s: &str, haser: &mut T) -> u64{
	s.hash(haser);
	haser.finish()
}


#[test]
fn test() {
    let at1 = Atom::from("abc");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 1);
	let at2 = Atom::from("afg");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 2);
	let at3 = Atom::from("afg");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 2);

	assert_eq!((at3.0).0, "afg");
}
#![allow(warnings)]

#![feature(core_intrinsics)]
#![feature(nll)]
#![feature(pattern)]
#![feature(weak_counts)]

/**
 * 全局的线程安全的原子字符串池
 * 某些高频单次的Atom，可以在应用层增加一个cache来缓冲Atom，定期检查引用计数来判断是否缓冲。
 */

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

extern crate fnv;
extern crate bon;
extern crate share;
extern crate hash;


use std::mem::replace;
use std::ops::Deref;
use std::convert::From;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::{Entry};
use std::str::pattern::Pattern;
use std::str::Split;
use std::iter::Map;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize, Serializer, Deserializer};

use hash::{XHashMap, DefaultHasher};
use bon::{WriteBuffer, ReadBuffer, Encode, Decode, ReadBonErr};
use share::{Share, ShareWeak, ShareRwLock};

// 同步原语，可用于运行一次性初始化。用于全局，FFI或相关功能的一次初始化。
lazy_static! {
	static ref ATOM_MAP: Table = Table(ShareRwLock::new(XHashMap::default()));
	pub static ref EMPTY: Atom = Atom::from(Vec::new());
}

// 原子字符串
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Atom(Share<(String, usize)>);
unsafe impl Sync for Atom {}
unsafe impl Send for Atom {}

#[cfg(feature = "serde")]
impl Serialize for Atom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.0).0.serialize(serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Atom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

impl Hash for Atom {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write_usize(((self.0).1).clone())
    }
}

impl AsRef<str> for Atom {
    #[inline(always)]
    fn as_ref(&self) -> &str{
        (*self.0).0.as_ref()
    }
}

impl Deref for Atom {
	type Target = String;
    #[inline(always)]
	fn deref(&self) -> &String {
		&(*self.0).0
	}
}

impl Encode for Atom{
	fn encode(&self, bb: &mut WriteBuffer){
		(*self.0).0.encode(bb);
	}
}

impl Decode for Atom{
	fn decode(bb: &mut ReadBuffer) -> Result<Atom, ReadBonErr>{
		Ok(Atom::from(String::decode(bb)?))
	}
}

impl Atom {
	// 返回的正整数为0表示静态原子，1表示为动态原子
	// fn contain(s: Option<&String>, h: usize) -> Option<usize> {
	// 	return None
	// }
    #[inline(always)]
	pub fn get_hash(&self) -> usize {
		(*self.0).1
	}

	pub fn get(hash: usize) -> Option<Atom> {
		ATOM_MAP.get(hash)
	}

	// #[inline(always)]
	// pub fn from_hash(hash: usize) -> Option<Atom> {
		
	// }
}

impl From<String> for Atom {
    #[inline(always)]
	fn from(s: String) -> Atom {
		ATOM_MAP.or_insert(s)
	}
}

impl<'a> From<&'a str> for Atom {
    #[inline(always)]
	fn from(s: &str) -> Atom {
		ATOM_MAP.or_insert(String::from(s))
	}
}

impl From<Vec<u8>> for Atom {
    #[inline(always)]
	fn from(s: Vec<u8>) -> Atom {
		ATOM_MAP.or_insert(unsafe { String::from_utf8_unchecked(s) })
	}
}

impl<'a> From<&'a [u8]> for Atom {
	#[inline(always)]
	fn from(s: &[u8]) -> Atom {
		ATOM_MAP.or_insert(unsafe { String::from_utf8_unchecked(Vec::from(s)) })
	}
}
/// 劈分字符串, 返回trim后的Atom的迭代器
pub fn split<'a, P: Pattern<'a>>(s: &'a String, pat: P) -> Map<Split<'a, P>, fn(&str) -> Atom> {
    s.split(pat).map(|r|{
        Atom::from(r.trim_start().trim_end())
    })
}

//todo
// 为完美hash准备的方法
// impl From<usize> for Atom {
// 	#[inline]
// 	fn from(s: String) -> Atom {
// 		(Share::new((s, 0)))
// 	}
// }
// fn from(s: String) -> Atom {
// 	//loop {
// 		// 先读锁，然后升级成写锁，如果升级失败则放弃读锁重新循环
// 		Atom(Share::new((s, 0)))
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
// const BUCKET_MASK: usize = (1 << 12) - 1;

// struct StringCache {
//     buckets: [Option<Box<(String, usize)>>; NB_BUCKETS],
// }

// impl StringCache{
// 	pub fn new() -> StringCache{

// 	}
// }
// lazy_static! {
//     static ref STRING_CACHE: Mutex<StringCache> = Mutex::new(StringCache::new());
// }

struct Table(ShareRwLock<XHashMap<usize, VerCowList>>);
unsafe impl Send for Table {}

impl Table{
	pub fn get(&self, h: usize) -> Option<Atom>{
		let map = self.0.read().unwrap();
		match map.get(&h) {
			Some(v) => match v.list.value.upgrade() {
				Some(r) => Some(Atom(r)),
				_ => None,
			},
			_ => None,
		}
	}
	pub fn or_insert(&self, s: String) -> Atom {
		let h = str_hash(&s, &mut DefaultHasher::default());
		let optlist = {
			let map = self.0.read().unwrap();
			match map.get(&h) {
				Some(v) => Some(v.clone()),
				_ => None
			}
		};
		let (version, list, strong) = match optlist {
			Some(ver_list) => {
				let mut nil_count = 0;
				 match read_nil(&ver_list.list, &s, &mut nil_count) {
					Some(r) => return r,
					_ => {
						let strong = Share::new((s, h));
						// 如果存在无效弱引用，应该删除
						let next = if nil_count > 1 {
							free(ver_list.list, nil_count)
						}else{
							Some(Share::new(ver_list.list))
						};
						let node = CowList::with_next(Share::downgrade(&strong), next);
						(ver_list.version, node, strong)
					}
				 }
			},
			_ => {
                let strong = Share::new((s, h));
                (0, CowList::new(Share::downgrade(&strong)), strong)
            }
		};

		let mut map = self.0.write().unwrap();
		match map.entry(h) {
			Entry::Occupied(mut e) => {
				let old = e.get_mut();
				if old.version == version { // 版本未更新，插入当前值
					old.version += 1;
					old.list = list;
				}else{ // 版本被更新，需要重新检查
					match read(&old.list, strong.as_ref().0.as_str()) {
						Some(r) => return r,
						_ => {
							// 将自己的节点放到头部
							let list = replace(&mut old.list, list);
							old.list.next = Some(Share::new(list));
						}
					}
				}
			},
			Entry::Vacant(e) => {
				e.insert(VerCowList{list, version: 1});
			}
		}
		Atom(strong)
	}

}
#[inline(always)]
fn str_hash<T: Hasher>(s: &str, hasher: &mut T) -> usize{
	s.hash(hasher);
	hasher.finish() as usize
}

fn read_nil(mut list: &CowList, s: &str, nil_count: &mut usize) -> Option<Atom> {
	loop {
		match list.value.upgrade() {
			Some(r) => {
				if r.0 == s {
					return Some(Atom(r))
				}
			},
			_ => *nil_count += 1,
		}
		match list.next {
			Some(ref r) => list = r,
			_ => return None
		}
	}
}
fn read(mut list: &CowList, s: &str) -> Option<Atom> {
	loop {
		match list.value.upgrade() {
			Some(r) => {
				if r.0 == s {
					return Some(Atom(r))
				}
			},
			_ => (),
		}
		match list.next {
			Some(ref r) => list = r,
			_ => return None
		}
	}
}
fn free(list: CowList, nil_count: usize) -> Option<Share<CowList>> {
	if list.value.strong_count() > 0 {
		let next = (*list.next.unwrap()).clone();
		Some(Share::new(CowList::with_next(list.value.clone(), free(next, nil_count))))
	}else{
		if nil_count == 1 {
			return list.next
		}
		free((*list.next.unwrap()).clone(), nil_count - 1)
	}
}

#[derive(Clone)]
struct VerCowList{
	list:CowList,
	version:usize,
}
unsafe impl Sync for VerCowList {}

#[derive(Clone, Debug)]
struct CowList{
	value:ShareWeak<(String, usize)>,
	next:Option<Share<CowList>>,
}

impl CowList{
	pub fn new(value: ShareWeak<(String, usize)>) -> Self {
		CowList{
			value,
			next: None,
		}
	}
	pub fn with_next(value: ShareWeak<(String, usize)>, next:Option<Share<CowList>>) -> Self {
		CowList{
			value,
			next,
		}
	}
}

#[cfg(test)]
extern crate time;

#[test]
fn test_atom() {

    Atom::from("abc");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 1);
	Atom::from("afg");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 2);
	let at3 = Atom::from("afg");
	assert_eq!(ATOM_MAP.0.read().expect("ATOM_MAP:error").len(), 2);
	assert_eq!((at3.0).0, "afg");
    let mut buf = WriteBuffer::new();
    let a = Atom::from("vvvvvvv");
    a.encode(&mut buf);
	println!("EMPTY: {:?}", *EMPTY);

    let mut map = XHashMap::default();
    let time = time::now_millisecond();
    for _ in 0..1000000 {
        map.insert("xx", "xx");
    }
    println!("insert map time{}", time::now_millisecond() - time);

    let time = time::now_millisecond();
    for i in 0..1000000 {
        Atom::from(i.to_string());
    }
    println!("atom from time{}", time::now_millisecond() - time);

    
    let mut arr = Vec::new();
    for i in 0..1000{
        arr.push(Atom::from(i.to_string()));
    }

    let time = time::now_millisecond();
    for i in 0..1000{
        for _ in 0..1000{
            Atom::from(arr[i].as_str());
        }
    }
    println!("atom1 from time{}", time::now_millisecond() - time);


    let time = time::now_millisecond();
    for i in 0..1000{
        for _ in 0..1000{
            Share::new((arr[i].as_str().to_string(), 5));
        }
    }
    println!("Share::new time{}", time::now_millisecond() - time);

    let time = time::now_millisecond();
    for i in 0..1000{
        for _ in 0..1000{
            arr[i].as_str().to_string();
        }
    }
    println!("to_string time{}", time::now_millisecond() - time);

    let time = time::now_millisecond();
    for i in 0..10{
        for _ in 0..100000{
            let _ = str_hash(arr[i].as_str(), &mut DefaultHasher::default());
        }
    }
    println!("cul hash{}", time::now_millisecond() - time);

    let time = time::now_millisecond();
    let xx = Share::new(1);
    let w = Share::downgrade(&xx);
    for _ in 0..1000000{
            w.upgrade();
    }
    println!("upgrade{}", time::now_millisecond() - time);

    let time = time::now_millisecond();
    let xx = Share::new(1);
    //let w = Share::downgrade(&xx);
    for _ in 0..1000{
        for _ in 0..1000{
            let _a = xx.clone();
        }
    }
    println!("clone {}", time::now_millisecond() - time);

}
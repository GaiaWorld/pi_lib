#![feature(core_intrinsics)] 
#![feature(nll)]
/**
 * 全局的线程安全的原子字符串池，为了移植问题，可能需要将实现部分移到其他库
 * 某些高频单次的Atom，可以在应用层增加一个cache来缓冲Atom，定期检查引用计数来判断是否缓冲。
 */

extern crate fnv;
extern crate bon;

#[macro_use]
extern crate lazy_static;

extern crate flame;
#[macro_use]
extern crate flamer;

use std::ops::Deref;
use std::convert::From;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, Weak};
use std::sync::RwLock;
use std::marker::PhantomData;

use fnv::FnvHashMap;

use bon::{WriteBuffer, ReadBuffer, Encode, Decode, ReadBonErr};

// 同步原语，可用于运行一次性初始化。用于全局，FFI或相关功能的一次初始化。
lazy_static! {
	static ref ATOM_MAP: Table = Table(RwLock::new(FnvHashMap::default()));
}

// 原子字符串
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Atom(Arc<(String, u64)>);

impl Deref for Atom {
	type Target = String;
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

struct Table(RwLock<FnvHashMap<u64, (usize, CowList)>>);

impl Table{
	pub fn or_insert(&self, s: String) -> Arc<(String, u64)>{
		let h = str_hash(&s, &mut DefaultHasher::new());
        let list = {
			let map = self.0.read().unwrap();
			match map.get(&h){
				Some(v) => Some((v.0, v.1.clone())),
				None => None,
			}
		};
		let (version, nil_count, mut list, strong) = match list {
			Some((ver, cow)) => {
				let mut nil_count = 0;
				match read(&cow, &s, &mut nil_count){
					Some(r) => return r,
					None => {
                        let strong = Arc::new((s, h));
                        let mut cow = cow.clone();
                        (ver, nil_count, cow.push(Arc::downgrade(&strong)), strong)
                    }
				}
			},
			None => {
                let strong = Arc::new((s, h));
                (0, 0, CowList::new(Arc::downgrade(&strong)), strong)
            }
		};

        // 如果存在无效弱引用，应该删除 TODO
		if nil_count > 0{
            let l = list.clone();
            let mut iter = l.iter();
            list = CowList::new(iter.next().unwrap().clone());
            loop {
                match iter.next() {
                    Some(node) =>{
                        match node.upgrade() {
                            Some(_n) => list = list.push_uncopy(node.clone()),
                            None => (),
                        }
                    },
                    None => break,
                }
            }
		}

        let mut map = self.0.write().unwrap();

        //map中不存在为h的key，证明版本未更新，插入当前值，并返回强引用
        let mut is_modify = false;
        let entry = map.entry(h).and_modify(|e|{
            if e.0 == version {
                e.1 = list.clone();
                e.0 += 1;
            } else {
                let mut _c = 0;
                match read(&e.1, strong.as_ref().0.as_str(), &mut _c) {
                    Some(_) => (),
                    None => {
                        e.1 = e.1.push(Arc::downgrade(&strong));
                        e.0 += 1;
                    },
                } 
            }
            is_modify = true;
        });
        if is_modify == false { //如果map中不存在值， 并且当前版本值为0， 直接插入list
            if version == 0{
                entry.or_insert((1, list));
            }else {
                entry.or_insert((1, CowList::new(Arc::downgrade(&strong))));
            }
            
        }
        return strong;

        //return Arc::new(("sss".to_string(), 5));
	}


}

fn str_hash<T: Hasher>(s: &str, haser: &mut T) -> u64{
	s.hash(haser);
	haser.finish()
}

fn read(list: &CowList, s: &str, nil_count: &mut usize) -> Option<Arc<(String, u64)>>{
	for o in list.iter(){
		let strong = o.upgrade();
		match strong {
			Some(o) => {
				if o.0 == s{
					return Some(o);
				}
			},
			None => {
                *nil_count += 1;
            },
		};
	}
	None
}

#[derive(Clone)]
struct CowList{
	next:Option<Arc<CowList>>,
	pub value:Weak<(String, u64)>,
}

impl CowList{
	pub fn new(ele: Weak<(String, u64)>) -> Self {
		CowList{
			next: None,
			value: ele
		}
	}

	pub fn push(&mut self, ele: Weak<(String, u64)>) -> CowList {
		CowList{
			next: Some(Arc::new(self.clone())),
			value: ele,
		}
	}

    pub fn push_uncopy(self, ele: Weak<(String, u64)>) -> CowList {
		CowList{
			next: Some(Arc::new(self)),
			value: ele,
		}
	}

	pub fn iter(&self) -> Iter{
		Iter{
			head: Some(&self),
		}
	}
}

pub struct Iter<'a> {
    head: Option<&'a CowList>,
}

impl<'a> Iterator for Iter<'a>{
	type Item = &'a Weak<(String, u64)>;
	fn next(&mut self) -> Option<&'a Weak<(String, u64)>>{
		let list = match self.head {
			Some(list) => list,
			None => return None,
		};

		self.head = match list.next{
            Some(ref list) => Some(list.as_ref()),
            None => None,
        };
		Some(&list.value)
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


    let mut map = FnvHashMap::default();
    let time = time::now_millis();
    for _ in 0..1000000 {
        map.insert("xx", "xx");
    }
    println!("insert map time{}", time::now_millis() - time);

    let time = time::now_millis();
    for i in 0..1000000 {
        Atom::from(i.to_string());
    }
    println!("atom from time{}", time::now_millis() - time);

    
    let mut arr = Vec::new();
    for i in 0..1000{
        arr.push(Atom::from(i.to_string()));
    }

    let time = time::now_millis();
    for i in 0..1000{
        for _ in 0..1000{
            Atom::from(arr[i].as_str());
        }
    }
    println!("atom1 from time{}", time::now_millis() - time);


    let time = time::now_millis();
    for i in 0..1000{
        for _ in 0..1000{
            Arc::new((arr[i].as_str().to_string(), 5));
        }
    }
    println!("arc::new time{}", time::now_millis() - time);

    let time = time::now_millis();
    for i in 0..1000{
        for _ in 0..1000{
            arr[i].as_str().to_string();
        }
    }
    println!("to_string time{}", time::now_millis() - time);

    let time = time::now_millis();
    for i in 0..10{
        for _ in 0..1000{
            let _ = str_hash(arr[i].as_str(), &mut DefaultHasher::new());
        }
    }
    println!("cul hash{}", time::now_millis() - time);

    let time = time::now_millis();
    let xx = Arc::new(1);
    let w = Arc::downgrade(&xx);
    for _ in 0..1000{
        for _ in 0..1000{
            w.upgrade();
        }
    }
    println!("upgrade{}", time::now_millis() - time);

    let time = time::now_millis();
    let xx = Arc::new(1);
    //let w = Arc::downgrade(&xx);
    for _ in 0..1000{
        for _ in 0..1000{
            xx.clone();
        }
    }
    println!("clone {}", time::now_millis() - time);

}
//  // #![feature(generators, generator_trait)]

//! 基于COW的有序表，根据具体的实现支持单线程或多线程安全


// use std::sync::atomic::{AtomicPtr, Ordering as AOrdering};
// use std::mem;
// use std::cmp::Ordering;
// //use std::ops::{Generator, GeneratorState};


// pub enum ActionResult<T> {
// 	Ignore,
// 	Delete,
// 	Upsert(T),
// }
// pub enum ActionResultType {
// 	Insert,
// 	Update,
// 	Delete,
// }

// #[derive(Clone, Debug)]
// pub struct Entry<K: Clone, V: Clone>(pub K, pub V);
// impl<K: Clone, V: Clone> Entry<K, V> {
// 	pub fn new(k: K, v: V) -> Self {
// 		Entry(k, v)
// 	}
// }

// impl<K: Ord + Clone, V: Clone> PartialEq for Entry<K, V> {
// 	fn eq(&self, other: &Entry<K, V>) -> bool {
//         self.0 == other.0
//     }
// }

// impl<K: Ord + Clone, V: Clone> PartialOrd for Entry<K, V> {
// 	fn partial_cmp(&self, other: &Entry<K, V>) -> Option<Ordering> {
//         Some(self.0.cmp(&other.0))
//     }
// }

// impl<K: Ord + Clone, V: Clone> Eq for Entry<K, V> {}

// impl<K: Ord + Clone, V: Clone> Ord for Entry<K, V> {
// 	fn cmp(&self, other: &Entry<K, V>) -> Ordering {
//         self.0.cmp(&other.0)
//     }
// }

// pub trait ImOrdMap {
// 	type Key: Clone;
// 	type Val: Clone;
// 	fn new() -> Self;
// 	fn from_order(Vec<Entry<Self::Key, Self::Val>>) -> Self;
// 	fn is_empty(&self) -> bool;
// 	fn size(&self) -> usize;
// 	fn has(&self, &Self::Key) -> bool;
// 	fn get(&self, key: &Self::Key) -> Option<&Self::Val>;
// 	fn min(&self) -> Option<&Entry<Self::Key, Self::Val>>;
// 	fn max(&self) -> Option<&Entry<Self::Key, Self::Val>>;
// 	fn rank(&self, &Self::Key) -> isize;
// 	fn index(&self, usize) -> Option<&Entry<Self::Key, Self::Val>>;

// 	fn insert(&self, Self::Key, Self::Val) -> Option<Self> where Self: Sized;
// 	fn update(&self, Self::Key, Self::Val, bool) -> Option<(Option<Self::Val>, Self)> where Self: Sized;
// 	fn upsert(&self, Self::Key, Self::Val, bool) -> (Option<Option<Self::Val>>, Self) where Self: Sized;
// 	fn delete(&self, &Self::Key, bool) ->Option<(Option<Self::Val>, Self)> where Self: Sized;
// 	fn remove(&self, usize, bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
// 	fn pop_min(&self, bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
// 	fn pop_max(&self, bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
// 	fn action<F>(&self, &Self::Key, &mut F) -> Option<(ActionResultType, Self)> where F: FnMut(Option<&Self::Val>) -> ActionResult<Self::Val>, Self: Sized;
// 	fn map<F>(&self, &mut F) -> Self where F: FnMut(&Entry<Self::Key, Self::Val>) -> ActionResult<Self::Val>, Self: Sized; 
// }

// pub trait Iter<'a>: ImOrdMap{
// 	type K: 'a + Clone + Ord;
// 	type V: 'a + Clone;
// 	type IterType: Iterator<Item= &'a Entry<Self::K, Self::V>>;
// 	fn iter(&self, Option<&Self::Key>, bool) -> Self::IterType;
// }


// #[derive(Clone)]
// pub struct OrdMap<T:Clone> {
// 	root: T,
// }

// pub struct Keys<'a, T: Iter<'a>>{
// 	inner: T::IterType
// }

// impl<'a, T: Iter<'a>> Iterator for Keys<'a, T>{
// 	type Item = &'a T::K;
// 	fn next(&mut self) -> Option<Self::Item>{
// 		self.inner.next().map(|Entry(k, _)| k)
// 	}
// }

// pub struct Values<'a, T: Iter<'a>>{
// 	inner: T::IterType
// }

// impl<'a, T: Iter<'a>> Iterator for Values<'a, T>{
// 	type Item = &'a T::V;
// 	fn next(&mut self) -> Option<Self::Item>{
// 		self.inner.next().map(|Entry(_, v)| v)
// 	}
// }

// impl<'a, T: ImOrdMap + Clone + Iter<'a>> OrdMap<T> {
// 	/**
// 	 * 新建
// 	 */
// 	pub fn new(map: T) -> Self {
// 		OrdMap {
// 			root: map,
// 		}
// 	}

// 	/**
// 	 * 判断指针是否相等
// 	 */
// 	pub fn ptr_eq(&self, old: &Self) -> bool {
// 		AtomicPtr::new(&self.root as *const T as *mut usize).load(AOrdering::Relaxed) == AtomicPtr::new(&old.root as *const T as *mut usize).load(AOrdering::Relaxed)
// 	}

// 	/**
// 	 * 取根节点
// 	 */
// 	pub fn root(&self) -> &T {
// 		&self.root
// 	}
// 	/**
// 	 * 判空
// 	 */
// 	pub fn is_empty(&self) -> bool {
// 		self.root.is_empty()
// 	}
// 	/**
// 	 * 获取指定树的大小
// 	 */
// 	pub fn size(&self) -> usize {
// 		self.root.size()
// 	}
// 	/**
// 	 * 检查指定的Key在树中是否存在
// 	 */
// 	pub fn has(&self, key: &T::Key) -> bool {
// 		self.root.has(key)
// 	}
// 	/**
// 	 * 获取指定Key在树中的值
// 	 */
// 	pub fn get(&self, key: &T::Key) -> Option<&T::Val> {
// 		self.root.get(key)
// 	}
// 	/**
// 	 * 获取树中最小的键值对
// 	 */
// 	pub fn min(&self) -> Option<&Entry<T::Key, T::Val>> {
// 		self.root.min()
// 	}
// 	/**
// 	 * 获取树中最大的键值对
// 	 */
// 	pub fn max(&self) -> Option<&Entry<T::Key, T::Val>> {
// 		self.root.max()
// 	}
// 	/**
// 	 * 获取指定Key在树中的排名，0表示空树，1表示第一名，负数表示没有该key，排名比该排名小
// 	 */
// 	pub fn rank(&self, key: &T::Key) -> isize {
// 		self.root.rank(key)
// 	}
// 	/**
// 	 * 获取指定排名的键值，必须从1开始，如果超过最大排名，则返回None
// 	 */
// 	pub fn index(&self, i: usize) -> Option<&Entry<T::Key, T::Val>> {
// 		self.root.index(i)
// 	}

// 	//返回从指定键开始的键升序或降序迭代器，如果不指定键，则从最小或最大键开始
// 	//Returns the key ascending or descending iterator starting from the specified key. If the key is not specified, the minimum or maximum key starts.
// 	pub fn keys(&self, key: Option<&T::Key>, descending: bool) -> Keys<'a, T> {
// 		Keys{
// 			inner: self.root.iter(key, descending)
// 		}
// 	}

// 	//返回从指定键开始的值升序或降序迭代器，如果不指定键，则从最小或最大键开始
// 	//Returns the value ascending or descending iterator starting from the specified key. If the key is not specified, the minimum or maximum key starts
// 	pub fn values(&self, key: Option<&T::Key>, descending: bool) -> Values<'a, T> {
// 		Values{
// 			inner: self.root.iter(key, descending)
// 		}
// 	}

// 	pub fn iter(&self, key: Option<&T::Key>, descending: bool) -> T::IterType {
// 		self.root.iter(key, descending)
// 	}

// 	/**
// 	 *  插入一个新的键值对(不允许插入存在的key)
// 	 */
// 	pub fn insert(&mut self, key: T::Key, value: T::Val) -> bool {
// 		match self.root.insert(key, value) {
// 			Some(root) => {
// 				self.root = root;
// 				true
// 			},
// 			_ => false,
// 		}
// 	}
// 	/**
// 	 *  更新键值对(不允许插入不存在的key)，copy决定是否返回旧值
// 	 */
// 	pub fn update(&mut self, key: T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
// 		match self.root.update(key, value, copy) {
// 			Some((r, root)) => {
// 				self.root = root;
// 				Some(r)
// 			},
// 			_ => None,
// 		}
// 	}
// 	/**
// 	 *  放入指定的键值对，copy决定是否返回旧值
// 	 */
// 	pub fn upsert(&mut self, key: T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
// 		let (r, root) = self.root.upsert(key, value, copy);
// 		self.root = root;
// 		r
// 	}
// 	/**
// 	 * 用指定的键，删除一个键值对(有指定key则删除)，copy决定是否返回旧值
// 	 */
// 	pub fn delete(&mut self, key: &T::Key, copy: bool) -> Option<Option<T::Val>> {
// 		match self.root.delete(key, copy) {
// 			Some((r, root)) => {
// 				self.root = root;
// 				Some(r)
// 			},
// 			_ => None,
// 		}
// 	}
// 	/**
// 	 * 用指定的排名，删除一个键值对，copy决定是否返回旧值
// 	 */
// 	pub fn remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
// 		match self.root.remove(i, copy) {
// 			Some((r, root)) => {
// 				self.root = root;
// 				Some(r)
// 			},
// 			_ => None,
// 		}
// 	}
// 	/**
// 	 * 删除最小的键值对，copy决定是否返回旧值
// 	 */
// 	pub fn pop_min(&mut self, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
// 		match self.root.pop_min(copy) {
// 			Some((r, root)) => {
// 				self.root = root;
// 				Some(r)
// 			},
// 			_ => None,
// 		}
// 	}
// 	/**
// 	 * 删除最大的键值对，copy决定是否返回旧值
// 	 */
// 	pub fn pop_max(&mut self, copy: bool) -> Option<Option<Entry<T::Key, T::Val>>> {
// 		match self.root.pop_max(copy) {
// 			Some((r, root)) => {
// 				self.root = root;
// 				Some(r)
// 			},
// 			_ => None,
// 		}
// 	}
// 	/**
// 	 * 对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
// 	 */
// 	pub fn action<F>(&mut self, key: &T::Key, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&T::Val>) -> ActionResult<T::Val> {
// 		match self.root.action(key, func) {
// 			Some((r, root)) => {
// 				self.root = root;
// 				Some(r)
// 			},
// 			_ => None,
// 		}
// 	}

// 	/**
// 	 * 对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
// 	 */
// 	pub fn map<F>(&mut self, func: &mut F) where F: FnMut(&Entry<T::Key, T::Val>) -> ActionResult<T::Val> {
// 		self.root = self.root.map(func);
// 	}

// 	/**
// 	 *  多线程下，安全的插入一个新的键值对(不允许插入存在的key)
// 	 */
// 	pub fn safe_insert(&mut self, key: &T::Key, value: T::Val) -> bool {
// 		let mut old = &self.root;
// 		loop {
// 			match old.insert(key.clone(), value.clone()) {
// 				Some(root) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed) {
// 					Ok(_) => {
// 						mem::forget(root);
// 						return true
// 					}
// 					Err(val) => old = &*(val as *const T),
// 				}},
// 				_ => return false,
// 			}
// 		}
// 	}
// 	/**
// 	 *  多线程下，安全的更新键值对(不允许插入不存在的key)，copy决定是否返回旧值
// 	 */
// 	pub fn safe_update(&mut self, key: &T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
// 		let mut old = &self.root;
// 		loop {
// 			match old.update(key.clone(), value.clone(), copy) {
// 				Some((r, root)) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed) {
// 					Ok(_) =>{
// 						mem::forget(root);
// 						 return Some(r)
// 					}
// 					Err(val) => old = &*(val as *const T),
// 				}},
// 				_ => return None,
// 			}
// 		}
// 	}
// 	/**
// 	 *  多线程下，安全的放入指定的键值对，copy决定是否返回旧值
// 	 */
// 	pub fn safe_upsert(&mut self, key: &T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
// 		let mut old = &self.root;
// 		loop {
// 			let (r, root) = old.upsert(key.clone(), value.clone(), copy);
// 			unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed) {
// 				Ok(_) =>{
// 					mem::forget(root);
// 					return r
// 				}
// 				Err(val) => old = &*(val as *const T),
// 			}}
// 		}
// 	}
// 	/**
// 	 * 多线程下，安全的用指定的键，删除一个键值对(有指定key则删除)，copy决定是否返回旧值
// 	 */
// 	pub fn safe_delete(&mut self, key: &T::Key, copy: bool) -> Option<Option<T::Val>> {
// 		let mut old = &self.root;
// 		loop {
// 			match old.delete(key, copy) {
// 				Some((r, root)) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed) {
// 					Ok(_) =>{
// 						mem::forget(root);
// 						return Some(r)
// 					}
// 					Err(val) => old = &*(val as *const T),
// 				}},
// 				_ => return None,
// 			}
// 		}
// 	}
// 	/**
// 	 * 多线程下，安全的用指定的排名，删除一个键值对，copy决定是否返回旧值
// 	 */
// 	pub fn safe_remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
// 		let mut old = &self.root;
// 		loop {
// 			match old.remove(i, copy) {
// 				Some((r, root)) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed) {
// 					Ok(_) =>{
// 						mem::forget(root);
// 						return Some(r)
// 					}
// 					Err(val) => old = &*(val as *const T),
// 				}},
// 				_ => return None,
// 			}
// 		}
// 	}
// 	/**
// 	 * 多线程下，安全的删除最小的键值对，copy决定是否返回旧值
// 	 */
// 	pub fn safe_pop_min(&mut self, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
// 		let mut old = &self.root;
// 		loop {
// 			match old.pop_min(copy) {
// 				Some((r, root)) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed){
// 					Ok(_) =>{
// 						mem::forget(root);
// 						return Some(r)
// 					}
// 					Err(val) => old = &*(val as *const T),
// 				}},
// 				_ => return None,
// 			}
// 		}
// 	}
// 	/**
// 	 * 多线程下，安全的删除最大的键值对，copy决定是否返回旧值
// 	 */
// 	pub fn safe_pop_max(&mut self, copy: bool) -> Option<Option<Entry<T::Key, T::Val>>> {
// 		let mut old = &self.root;
// 		loop {
// 			match old.pop_max(copy) {
// 				Some((r, root)) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed){
// 					Ok(_) =>{
// 						mem::forget(root);
// 						return Some(r)
// 					}
// 					Err(val) => old = &*(val as *const T),
// 				}},
// 				_ => return None,
// 			}
// 		}
// 	}
// 	/**
// 	 * 多线程下，安全的对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
// 	 */
// 	pub fn safe_action<F>(&mut self, key: &T::Key, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&T::Val>) -> ActionResult<T::Val> {
// 		let mut old = &self.root;
// 		loop {
// 			match old.action(key, func) {
// 				Some((r, root)) => unsafe {match AtomicPtr::new(&self.root as *const T as *mut usize).compare_exchange(old as *const T as *mut usize, &root as *const T as *mut usize, AOrdering::Relaxed, AOrdering::Relaxed) {
// 					Ok(_) =>{
// 						mem::forget(root);
// 						return Some(r)
// 					}
// 					Err(val)=> old = &*(val as *const T),
// 				}},
// 				_ => return None,
// 			}
// 		}
// 	}
// }

// //====================================













use std::intrinsics;
use std::mem;
use std::cmp::Ordering;
//use std::ops::{Generator, GeneratorState};

/// 操作结果值
pub enum ActionResult<T> {
	Ignore,
	Delete,
	Upsert(T),
}
/// 函数操作结果类型
pub enum ActionResultType {
	Insert,
	Update,
	Delete,
}

/// 键值条目
#[derive(Clone, Debug)]
pub struct Entry<K: Clone, V: Clone>(pub K, pub V);
impl<K: Clone, V: Clone> Entry<K, V> {
	pub fn new(k: K, v: V) -> Self {
		Entry(k, v)
	}
}

impl<K: Ord + Clone, V: Clone> PartialEq for Entry<K, V> {
	fn eq(&self, other: &Entry<K, V>) -> bool {
        self.0 == other.0
    }
}

impl<K: Ord + Clone, V: Clone> PartialOrd for Entry<K, V> {
	fn partial_cmp(&self, other: &Entry<K, V>) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<K: Ord + Clone, V: Clone> Eq for Entry<K, V> {}

impl<K: Ord + Clone, V: Clone> Ord for Entry<K, V> {
	fn cmp(&self, other: &Entry<K, V>) -> Ordering {
        self.0.cmp(&other.0)
    }
}
/// 不可变有序表
pub trait ImOrdMap {
	type Key: Clone;
	type Val: Clone;
	/// 创建表
	fn new() -> Self;
	/// 创建并加载有序列表的kv值
	fn from_order(vec: Vec<Entry<Self::Key, Self::Val>>) -> Self;
	/// 判断是否为空
	fn is_empty(&self) -> bool;
	/// 获得键值条目的数量
	fn size(&self) -> usize;
	/// 判断指定的键是否存在
	fn has(&self, key: &Self::Key) -> bool;
	/// 获得指定的键对应的值
	fn get(&self, key: &Self::Key) -> Option<&Self::Val>;
	/// 获得最小键对应的值
	fn min(&self) -> Option<&Entry<Self::Key, Self::Val>>;
	/// 获得最大键对应的值
	fn max(&self) -> Option<&Entry<Self::Key, Self::Val>>;
	/// 获得指定键的排位
	fn rank(&self, key: &Self::Key) -> isize;
	/// 获得指定排位的键值
	fn index(&self, order: usize) -> Option<&Entry<Self::Key, Self::Val>>;

	/// 插入键值，返回新的表，如果已有该键则插入失败，返回None
	fn insert(&self, key: Self::Key, val: Self::Val) -> Option<Self> where Self: Sized;
	/// 更新键值，如果已有该键则更新失败，则返回None，如果copy为true，则返回(原值, 新的表)，如果copy为false，则返回(None, 新的表)，
	fn update(&self, key: Self::Key, val: Self::Val, copy: bool) -> Option<(Option<Self::Val>, Self)> where Self: Sized;
	/// 更新或插入键值，如果copy为true，则返回(Option<原值>, 新的表)，如果copy为false，则返回(None, 新的表)，
	fn upsert(&self, key: Self::Key, val: Self::Val, copy: bool) -> (Option<Option<Self::Val>>, Self) where Self: Sized;
	/// 删除指定的键，如果没有该键则删除失败，则返回None，如果copy为true，则返回(原值, 新的表)，如果copy为false，则返回(None, 新的表)，
	fn delete(&self, key: &Self::Key, copy: bool) ->Option<(Option<Self::Val>, Self)> where Self: Sized;
	/// 删除指定的键，如果没有该键则删除失败，则返回None，如果copy为true，则返回(Option<原键值>, 新的表)，如果copy为false，则返回(None, 新的表)，
	fn remove(&self, order: usize, copy: bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
	/// 弹出最小键值，如果表为空，则返回None，如果copy为true，则返回(Option<原键值>, 新的表)，如果copy为false，则返回(None, 新的表)，
	fn pop_min(&self, copy: bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
	/// 弹出最大键值，如果表为空，则返回None，如果copy为true，则返回(Option<原键值>, 新的表)，如果copy为false，则返回(None, 新的表)，
	fn pop_max(&self, copy: bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
	/// 用指定的函数操作指定的键
	fn action<F>(&self, key: &Self::Key, func: &mut F) -> Option<(ActionResultType, Self)> where F: FnMut(Option<&Self::Val>) -> ActionResult<Self::Val>, Self: Sized;
	/// 用指定的函数遍历表，返回操作后的新表
	fn map<F>(&self, func: &mut F) -> Self where F: FnMut(&Entry<Self::Key, Self::Val>) -> ActionResult<Self::Val>, Self: Sized; 
}

/// 为不可变有序表定义的迭代器
pub trait Iter<'a>: ImOrdMap{
	type K: 'a + Clone + Ord;
	type V: 'a + Clone;
	type IterType: Iterator<Item= &'a Entry<Self::K, Self::V>> + Send + 'a;
	/// 迭代方法， 从指定的键开始， 升序或降序遍历
	fn iter(&self, key: Option<&Self::Key>, descending: bool) -> Self::IterType;
}

/// 有序表
#[derive(Clone)]
pub struct OrdMap<T:Clone> {
	root: T,
}
/// 遍历的键集
pub struct Keys<'a, T: Iter<'a>>{
	inner: T::IterType
}

unsafe impl<'a, T: Iter<'a>> Send for Keys<'a, T> {}

/// 键集的迭代器
impl<'a, T: Iter<'a>> Iterator for Keys<'a, T>{
	type Item = &'a T::K;
	/// 返回下一个键
	fn next(&mut self) -> Option<Self::Item>{
		self.inner.next().map(|Entry(k, _)| k)
	}
}
/// 遍历的值集
pub struct Values<'a, T: Iter<'a>>{
	inner: T::IterType
}
/// 值集的迭代器
impl<'a, T: Iter<'a>> Iterator for Values<'a, T>{
	type Item = &'a T::V;
	/// 返回下一个值
	fn next(&mut self) -> Option<Self::Item>{
		self.inner.next().map(|Entry(_, v)| v)
	}
}

impl<'a, T: ImOrdMap + Clone + Iter<'a>> OrdMap<T> {
	/// 新建

	pub fn new(map: T) -> Self {
		OrdMap {
			root: map,
		}
	}

	/// v判断指针是否相等
	pub fn ptr_eq(&self, old: &Self) -> bool {
		unsafe { intrinsics::atomic_load_relaxed(&self.root as *const T as *const usize) == intrinsics::atomic_load_relaxed(&old.root as *const T as *const usize) }
	}

	/// 取根节点
	pub fn root(&self) -> &T {
		&self.root
	}
	/// 判空
	pub fn is_empty(&self) -> bool {
		self.root.is_empty()
	}
	/// 获取指定树的大小
	pub fn size(&self) -> usize {
		self.root.size()
	}
	/// 检查指定的Key在树中是否存在
	pub fn has(&self, key: &T::Key) -> bool {
		self.root.has(key)
	}
	/// 获取指定Key在树中的值
	pub fn get(&self, key: &T::Key) -> Option<&T::Val> {
		self.root.get(key)
	}
	/// 获取树中最小的键值对
	pub fn min(&self) -> Option<&Entry<T::Key, T::Val>> {
		self.root.min()
	}
	/// 获取树中最大的键值对
	pub fn max(&self) -> Option<&Entry<T::Key, T::Val>> {
		self.root.max()
	}
	/// 获取指定Key在树中的排名，0表示空树，1表示第一名，负数表示没有该key，排名比该排名小
	pub fn rank(&self, key: &T::Key) -> isize {
		self.root.rank(key)
	}
	/// 获取指定排名的键值，必须从1开始，如果超过最大排名，则返回None
	pub fn index(&self, i: usize) -> Option<&Entry<T::Key, T::Val>> {
		self.root.index(i)
	}

	/// 返回从指定键开始的键升序或降序迭代器，如果不指定键，则从最小或最大键开始
	/// Returns the key ascending or descending iterator starting from the specified key. If the key is not specified, the minimum or maximum key starts.
	pub fn keys(&self, key: Option<&T::Key>, descending: bool) -> Keys<'a, T> {
		Keys{
			inner: self.root.iter(key, descending)
		}
	}

	/// 返回从指定键开始的值升序或降序迭代器，如果不指定键，则从最小或最大键开始
	/// Returns the value ascending or descending iterator starting from the specified key. If the key is not specified, the minimum or maximum key starts
	pub fn values(&self, key: Option<&T::Key>, descending: bool) -> Values<'a, T> {
		Values{
			inner: self.root.iter(key, descending)
		}
	}
	/// 从指定键开始的值升序或降序迭代
	pub fn iter(&self, key: Option<&T::Key>, descending: bool) -> T::IterType {
		self.root.iter(key, descending)
	}

	/// 插入一个新的键值对(不允许插入存在的key)
	pub fn insert(&mut self, key: T::Key, value: T::Val) -> bool {
		match self.root.insert(key, value) {
			Some(root) => {
				self.root = root;
				true
			},
			_ => false,
		}
	}
	/// 更新键值对(不允许插入不存在的key)，copy决定是否返回旧值
	pub fn update(&mut self, key: T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		match self.root.update(key, value, copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/// 放入指定的键值对，copy决定是否返回旧值
	pub fn upsert(&mut self, key: T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		let (r, root) = self.root.upsert(key, value, copy);
		self.root = root;
		r
	}
	/// 用指定的键，删除一个键值对(有指定key则删除)，copy决定是否返回旧值
	pub fn delete(&mut self, key: &T::Key, copy: bool) -> Option<Option<T::Val>> {
		match self.root.delete(key, copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/// 用指定的排名，删除一个键值对，copy决定是否返回旧值
	pub fn remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		match self.root.remove(i, copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/// 删除最小的键值对，copy决定是否返回旧值
	pub fn pop_min(&mut self, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		match self.root.pop_min(copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/// 删除最大的键值对，copy决定是否返回旧值
	pub fn pop_max(&mut self, copy: bool) -> Option<Option<Entry<T::Key, T::Val>>> {
		match self.root.pop_max(copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/// 对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
	pub fn action<F>(&mut self, key: &T::Key, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&T::Val>) -> ActionResult<T::Val> {
		match self.root.action(key, func) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}

	/// 对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
	pub fn map<F>(&mut self, func: &mut F) where F: FnMut(&Entry<T::Key, T::Val>) -> ActionResult<T::Val> {
		self.root = self.root.map(func);
	}

	/// 多线程下，安全的插入一个新的键值对(不允许插入存在的key)
	pub fn safe_insert(&mut self, key: &T::Key, value: T::Val) -> bool {
		let mut old = &self.root;
		loop {
			match old.insert(key.clone(), value.clone()) {
				Some(root) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) => {
						//mem::forget(root);
						return true
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return false,
			}
		}
	}
	/// 多线程下，安全的更新键值对(不允许插入不存在的key)，copy决定是否返回旧值
	pub fn safe_update(&mut self, key: &T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		let mut old = &self.root;
		loop {
			match old.update(key.clone(), value.clone(), copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) =>{
						mem::forget(root);
						 return Some(r)
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/// 多线程下，安全的放入指定的键值对，copy决定是否返回旧值
	pub fn safe_upsert(&mut self, key: &T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		let mut old = &self.root;
		loop {
			let (r, root) = old.upsert(key.clone(), value.clone(), copy);
			unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
				(_, true) =>{
					mem::forget(root);
					return r
				}
				(val, _) => old = &*(val as *const T),
			}}
		}
	}
	/// 多线程下，安全的用指定的键，删除一个键值对(有指定key则删除)，copy决定是否返回旧值
	pub fn safe_delete(&mut self, key: &T::Key, copy: bool) -> Option<Option<T::Val>> {
		let mut old = &self.root;
		loop {
			match old.delete(key, copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) =>{
						mem::forget(root);
						return Some(r)
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/// 多线程下，安全的用指定的排名，删除一个键值对，copy决定是否返回旧值
	pub fn safe_remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		let mut old = &self.root;
		loop {
			match old.remove(i, copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) =>{
						mem::forget(root);
						return Some(r)
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/// 多线程下，安全的删除最小的键值对，copy决定是否返回旧值
	pub fn safe_pop_min(&mut self, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		let mut old = &self.root;
		loop {
			match old.pop_min(copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) =>{
						mem::forget(root);
						return Some(r)
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/// 多线程下，安全的删除最大的键值对，copy决定是否返回旧值
	pub fn safe_pop_max(&mut self, copy: bool) -> Option<Option<Entry<T::Key, T::Val>>> {
		let mut old = &self.root;
		loop {
			match old.pop_max(copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) =>{
						mem::forget(root);
						return Some(r)
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/// 多线程下，安全的对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
	pub fn safe_action<F>(&mut self, key: &T::Key, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&T::Val>) -> ActionResult<T::Val> {
		let mut old = &self.root;
		loop {
			match old.action(key, func) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&self.root as *const T as *mut usize, *(old as *const T as *const usize), *(&root as *const T as *const usize)) {
					(_, true) =>{
						mem::forget(root);
						return Some(r)
					}
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
}

//====================================

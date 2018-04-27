// #![feature(generators, generator_trait)]

/**
 * 基于COW的有序表，根据具体的实现支持单线程或多线程安全
 */

use std::marker::PhantomData;
use std::intrinsics;
//use std::ops::{Generator, GeneratorState};


pub enum ActionResult<T> {
	Ignore,
	Delete,
	Upsert(T),
}
pub enum ActionResultType {
	Insert,
	Update,
	Delete,
}
// TODO 考虑提供RcEntry，这样复制成本低一些，kv都不要求支持clone了，但内存更分散，访问k要多一层访问。
#[derive(Clone)]
// pub struct Entry<K: Clone, V: Clone>(K, V);
pub struct Entry<K: Clone, V: Clone> {
	pub key: K,
	pub value: V,
}
impl<K: Clone, V: Clone> Entry<K, V> {
	pub fn new(k: K, v: V) -> Self {
		Entry {
			key: k,
			value: v,
		}
	}
	pub fn key(&self) -> &K {
		&self.key
	}
	pub fn value(&self) -> &V {
		&self.value
	}
}

pub trait ImOrdMap {
	type Key: Clone;
	type Val: Clone;
	fn new() -> Self;
	//fn from_order(Vec<Entry<Self::Key, Self::Val>>) -> Self;
	fn is_empty(&self) -> bool;
	fn size(&self) -> usize;
	fn has(&self, &Self::Key) -> bool;
	fn get(&self, key: &Self::Key) -> Option<&Self::Val>;
	//fn get(&self, &Self::Key) -> Option<&Self::Val>;
	fn min(&self) -> Option<&Entry<Self::Key, Self::Val>>;
	fn max(&self) -> Option<&Entry<Self::Key, Self::Val>>;
	fn rank(&self, &Self::Key) -> isize;
	fn index(&self, usize) -> Option<&Entry<Self::Key, Self::Val>>;
	// fn Self::Keys(&self, Self::Key: Option<&Self::Key>, descending: bool) -> Generator;
	//fn Self::Values(&self, Self::Key: Option<&Self::Key>, descending: bool) -> gen;
	//fn entrys(&self, Self::Key: Option<&Self::Key>, descending: bool) -> gen;
	fn select<F>(&self, Option<&Self::Key>, &mut F) where F: FnMut(&Entry<Self::Key, Self::Val>);

	fn insert(&self, Self::Key, Self::Val) -> Option<Self> where Self: Sized;
	fn update(&self, Self::Key, Self::Val, bool) -> Option<(Option<Self::Val>, Self)> where Self: Sized;
	fn upsert(&self, Self::Key, Self::Val, bool) -> (Option<Option<Self::Val>>, Self) where Self: Sized;
	fn delete(&self, &Self::Key, bool) ->Option<(Option<Self::Val>, Self)> where Self: Sized;
	fn remove(&self, usize, bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
	fn pop_min(&self, bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
	fn pop_max(&self, bool) -> Option<(Option<Entry<Self::Key, Self::Val>>, Self)> where Self: Sized;
	fn action<F>(&self, &Self::Key, &mut F) -> Option<(ActionResultType, Self)> where F: FnMut(Option<&Self::Val>) -> ActionResult<Self::Val>, Self: Sized;
	// fn map(&self, Fn) -> (usize, Self);

}


#[derive(Clone)]
pub struct OrdMap<T:Clone> {
	root: T,
}

impl<T: ImOrdMap + Clone> OrdMap<T> {
	/**
	 * 新建
	 */
	pub fn new(map: T) -> Self {
		OrdMap {
			root: map,
		}
	}
	/**
	 * 判断是否被修改
	 */
	pub fn is_modify(&self, old: &Self) -> bool {
		unsafe { intrinsics::atomic_load_relaxed(&(&self.root as *const T as usize)) != intrinsics::atomic_load_relaxed(&(&old.root as *const T as usize)) }
	}
	/**
	 * 比较并交换
	 */
	pub fn cxchg(&mut self, old: &mut Self, new: &mut Self) -> bool {
		let (_, r) = unsafe { intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), &old.root as *const T as usize, &new.root as *const T as usize) };
		return r
	}
	/**
	 * 取根节点
	 */
	pub fn root(&self) -> &T {
		&self.root
	}
	/**
	 * 判空
	 */
	pub fn is_empty(&self) -> bool {
		self.root.is_empty()
	}
	/**
	 * 获取指定树的大小
	 */
	pub fn size(&self) -> usize {
		self.root.size()
	}
	/**
	 * 检查指定的Key在树中是否存在
	 */
	pub fn has(&self, key: &T::Key) -> bool {
		self.root.has(key)
	}
	/**
	 * 获取指定Key在树中的值
	 */
	pub fn get(&self, key: &T::Key) -> Option<&T::Val> {
		self.root.get(key)
	}
	/**
	 * 获取树中最小的键值对
	 */
	pub fn min(&self) -> Option<&Entry<T::Key, T::Val>> {
		self.root.min()
	}
	/**
	 * 获取树中最大的键值对
	 */
	pub fn max(&self) -> Option<&Entry<T::Key, T::Val>> {
		self.root.max()
	}
	/**
	 * 获取指定Key在树中的排名，0表示空树，1表示第一名，负数表示没有该key，排名比该排名小
	 */
	pub fn rank(&self, key: &T::Key) -> isize {
		self.root.rank(key)
	}
	/**
	 * 获取指定排名的键值，必须从1开始，如果超过最大排名，则返回None
	 */
	pub fn index(&self, i: usize) -> Option<&Entry<T::Key, T::Val>> {
		self.root.index(i)
	}
	// /**
	//  * 返回从指定键开始的键迭代器，升序或降序，如果不指定键，则从最大或最小键开始，
	//  */
	// fn keys(&self, Option<&Self::Key>, descending: bool) -> Generator;
	//fn values(&self, Option<&Self::Key>, descending: bool) -> gen;
	//fn entrys(&self, Option<&Self::Key>, descending: bool) -> gen;
	/**
	 * 选择器方法，从指定键开始进行选择，TODO 升序或降序，如果不指定键，则从最小键开始
	 */
	pub fn select<F>(&self, key: Option<&T::Key>, func: &mut F) where F: FnMut(&Entry<T::Key, T::Val>) {
		self.root.select(key, func)
	}
	/**
	 *  插入一个新的键值对(不允许插入存在的key)
	 */
	pub fn insert(&mut self, key: T::Key, value: T::Val) -> bool {
		match self.root.insert(key, value) {
			Some(root) => {
				self.root = root;
				true
			},
			_ => false,
		}
	}
	/**
	 *  更新键值对(不允许插入不存在的key)，copy决定是否返回旧值
	 */
	pub fn update(&mut self, key: T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		match self.root.update(key, value, copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/**
	 *  放入指定的键值对，copy决定是否返回旧值
	 */
	pub fn upsert(&mut self, key: T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		let (r, root) = self.root.upsert(key, value, copy);
		self.root = root;
		r
	}
	/**
	 * 用指定的键，删除一个键值对(有指定key则删除)，copy决定是否返回旧值
	 */
	pub fn delete(&mut self, key: &T::Key, copy: bool) -> Option<Option<T::Val>> {
		match self.root.delete(key, copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/**
	 * 用指定的排名，删除一个键值对，copy决定是否返回旧值
	 */
	pub fn remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		match self.root.remove(i, copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/**
	 * 删除最小的键值对，copy决定是否返回旧值
	 */
	pub fn pop_min(&mut self, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		match self.root.pop_min(copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/**
	 * 删除最大的键值对，copy决定是否返回旧值
	 */
	pub fn pop_max(&mut self, copy: bool) -> Option<Option<Entry<T::Key, T::Val>>> {
		match self.root.pop_max(copy) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	/**
	 * 对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
	 */
	pub fn action<F>(&mut self, key: &T::Key, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&T::Val>) -> ActionResult<T::Val> {
		match self.root.action(key, func) {
			Some((r, root)) => {
				self.root = root;
				Some(r)
			},
			_ => None,
		}
	}
	// fn map(&mut self, func: &mut F) ->usize where F: FnMut(Option<&Self::Val>) {

	/**
	 *  多线程下，安全的插入一个新的键值对(不允许插入存在的key)
	 */
	pub fn safe_insert(&mut self, key: &T::Key, value: T::Val) -> bool {
		let mut old = &self.root;
		loop {
			match old.insert(key.clone(), value.clone()) {
				Some(root) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return true,
					(val, _) => old = &*(val as *const T),
				}},
				_ => return false,
			}
		}
	}
	/**
	 *  多线程下，安全的更新键值对(不允许插入不存在的key)，copy决定是否返回旧值
	 */
	pub fn safe_update(&mut self, key: &T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		let mut old = &self.root;
		loop {
			match old.update(key.clone(), value.clone(), copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return Some(r),
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/**
	 *  多线程下，安全的放入指定的键值对，copy决定是否返回旧值
	 */
	pub fn safe_upsert(&mut self, key: &T::Key, value: T::Val, copy: bool) -> Option<Option<T::Val>> {
		let mut old = &self.root;
		loop {
			let (r, root) = old.upsert(key.clone(), value.clone(), copy);
			unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
				(_, true) => return r,
				(val, _) => old = &*(val as *const T),
			}}
		}
	}
	/**
	 * 多线程下，安全的用指定的键，删除一个键值对(有指定key则删除)，copy决定是否返回旧值
	 */
	pub fn safe_delete(&mut self, key: &T::Key, copy: bool) -> Option<Option<T::Val>> {
		let mut old = &self.root;
		loop {
			match old.delete(key, copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return Some(r),
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/**
	 * 多线程下，安全的用指定的排名，删除一个键值对，copy决定是否返回旧值
	 */
	pub fn safe_remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		let mut old = &self.root;
		loop {
			match old.remove(i, copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return Some(r),
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/**
	 * 多线程下，安全的删除最小的键值对，copy决定是否返回旧值
	 */
	pub fn safe_pop_min(&mut self, copy: bool) ->Option<Option<Entry<T::Key, T::Val>>> {
		let mut old = &self.root;
		loop {
			match old.pop_min(copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return Some(r),
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/**
	 * 多线程下，安全的删除最大的键值对，copy决定是否返回旧值
	 */
	pub fn safe_pop_max(&mut self, copy: bool) -> Option<Option<Entry<T::Key, T::Val>>> {
		let mut old = &self.root;
		loop {
			match old.pop_max(copy) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return Some(r),
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
	/**
	 * 多线程下，安全的对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
	 */
	pub fn safe_action<F>(&mut self, key: &T::Key, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&T::Val>) -> ActionResult<T::Val> {
		let mut old = &self.root;
		loop {
			match old.action(key, func) {
				Some((r, root)) => unsafe {match intrinsics::atomic_cxchg_failrelaxed(&mut (&self.root as *const T as usize), old as *const T as usize, &root as *const T as usize) {
					(_, true) => return Some(r),
					(val, _) => old = &*(val as *const T),
				}},
				_ => return None,
			}
		}
	}
}

//====================================

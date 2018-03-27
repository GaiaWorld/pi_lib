// #![feature(generators, generator_trait)]

/**
 * 基于COW的有序表，根据具体的实现支持单线程或多线程安全
 */

use std::marker::PhantomData;
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
#[derive(Copy)]
pub struct Entry<K, V> {
	key: K,
	value: V,
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
impl<K: Clone, V: Clone> Clone for Entry<K, V> {
    fn clone(&self) -> Self {
		Entry {
			key: self.key.clone(),
			value: self.value.clone(),
		}
	}
}

pub trait ImOrdMap<K, V> {
	fn new() -> Self;
	//fn clone(&self) -> Self;
	//fn from_order(Vec<Entry<K, V>>) -> Self;
	fn is_empty(&self) -> bool;
	fn size(&self) -> usize;
	fn has(&self, &K) -> bool;
	fn get(&self, key: &K) -> Option<&V>;
	//fn get(&self, &K) -> Option<&V>;
	fn min(&self) -> Option<&Entry<K, V>>;
	fn max(&self) -> Option<&Entry<K, V>>;
	fn rank(&self, &K) -> isize;
	fn index(&self, usize) -> Option<&Entry<K, V>>;
	// fn keys(&self, Option<&K>) -> Generator;
	//fn values(&self, Option<&K>) -> gen;
	//fn entrys(&self, Option<&K>) -> gen;
	fn select<F>(&self, Option<&K>, &mut F) where F: FnMut(&Entry<K, V>);

	fn insert(&self, K, V) -> Option<Self> where Self: Sized;
	fn update(&self, K, V, bool) -> Option<(Option<V>, Self)> where Self: Sized;
	fn upsert(&self, K, V, bool) -> (Option<Option<V>>, Self) where Self: Sized;
	fn delete(&self, &K, bool) ->Option<(Option<V>, Self)> where Self: Sized;
	fn remove(&self, usize, bool) -> Option<(Option<Entry<K, V>>, Self)> where Self: Sized;
	fn pop_min(&self, bool) -> Option<(Option<Entry<K, V>>, Self)> where Self: Sized;
	fn pop_max(&self, bool) -> Option<(Option<Entry<K, V>>, Self)> where Self: Sized;
	fn action<F>(&self, &K, &mut F) -> Option<(ActionResultType, Self)> where F: FnMut(Option<&V>) -> ActionResult<V>, Self: Sized;
	// fn map(&self, Fn) -> (usize, Self);

}

#[derive(Copy)]
pub struct OrdMap<K, V, T> {
	root: T,
	_k_marker:PhantomData<K>,
	_v_marker:PhantomData<V>,
}
impl<K, V, T: Clone> Clone for OrdMap<K, V, T> {
    fn clone(&self) -> Self {
		OrdMap {
		  root: self.root.clone(),
		  _k_marker: PhantomData,
		  _v_marker: PhantomData,
		}
	}
}
impl<K, V, T> OrdMap<K, V, T> where T: ImOrdMap<K, V> {
	/**
	 * 新建
	 */
	pub fn new(map: T) -> Self {
		OrdMap {
		  root: map,
		  _k_marker: PhantomData,
		  _v_marker: PhantomData,
		}
	}
	/**
	 * 判空
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
	pub fn has(&self, key: &K) -> bool {
		self.root.has(key)
	}
	/**
	 * 获取指定Key在树中的值
	 */
	pub fn get(&self, key: &K) -> Option<&V> {
		self.root.get(key)
	}
	/**
	 * 获取树中最小的键值对
	 */
	pub fn min(&self) -> Option<&Entry<K, V>> {
		self.root.min()
	}
	/**
	 * 获取树中最大的键值对
	 */
	pub fn max(&self) -> Option<&Entry<K, V>> {
		self.root.max()
	}
	/**
	 * 获取指定Key在树中的排名，0表示空树，1表示第一名，负数表示没有该key，排名比该排名小
	 */
	pub fn rank(&self, key: &K) -> isize {
		self.root.rank(key)
	}
	/**
	 * 获取指定排名的键值，必须从1开始，如果超过最大排名，则返回None
	 */
	pub fn index(&self, i: usize) -> Option<&Entry<K, V>> {
		self.root.index(i)
	}
	// /**
	//  * 返回从指定键开始的键迭代器，如果不指定键，则从最小键开始
	//  */
	// fn keys(&self, Option<&K>) -> Generator;
	//fn values(&self, Option<&K>) -> gen;
	//fn entrys(&self, Option<&K>) -> gen;
	/**
	 * 选择器方法，从指定键开始进行选择，如果不指定键，则从最小键开始
	 */
	pub fn select<F>(&self, key: Option<&K>, func: &mut F) where F: FnMut(&Entry<K, V>) {
		self.root.select(key, func)
	}
	/**
	 *  插入一个新的键值对(不允许插入存在的key)
	 */
	pub fn insert(&mut self, key: K, value: V) -> bool {
		let r = self.root.insert(key, value);
		match r {
			Some(rr) => {
				self.root = rr;
				true
			},
			_ => false,
		}
	}
	/**
	 *  插入一个新的键值对(不允许插入存在的key)
	 */
	pub fn update(&mut self, key: K, value: V, copy: bool) -> Option<Option<V>> {
		let r = self.root.update(key, value, copy);
		match r {
			Some((rr, root)) => {
				self.root = root;
				Some(rr)
			},
			_ => None,
		}
	}
	/**
	 *  放入指定的键值对，返回值0 true表示
	 */
	pub fn upsert(&mut self, key: K, value: V, copy: bool) -> Option<Option<V>> {
		let (r, root) = self.root.upsert(key, value, copy);
		self.root = root;
		r
	}
	/**
	 * 用指定的键，删除一个键值对(有指定key则删除)
	 */
	pub fn delete(&mut self, key: &K, copy: bool) -> Option<Option<V>> {
		let r = self.root.delete(key, copy);
		match r {
			Some((rr, root)) => {
				self.root = root;
				Some(rr)
			},
			_ => None,
		}
	}
	/**
	 * 用指定的排名，删除一个键值对
	 */
	pub fn remove(&mut self, i: usize, copy: bool) ->Option<Option<Entry<K, V>>> {
		let r = self.root.remove(i, copy);
		match r {
			Some((rr, root)) => {
				self.root = root;
				Some(rr)
			},
			_ => None,
		}
	}
	/**
	 * 删除最小的键值对
	 */
	pub fn pop_min(&mut self, copy: bool) ->Option<Option<Entry<K, V>>> {
		let r = self.root.pop_min(copy);
		match r {
			Some((rr, root)) => {
				self.root = root;
				Some(rr)
			},
			_ => None,
		}
	}
	/**
	 * 删除最大的键值对
	 */
	pub fn pop_max(&mut self, copy: bool) -> Option<Option<Entry<K, V>>> {
		let r = self.root.pop_max(copy);
		match r {
			Some((rr, root)) => {
				self.root = root;
				Some(rr)
			},
			_ => None,
		}
	}
	/**
	 * 对指定的键用指定的函数进行操作，函数返回ActionResult, 表示放弃 删除，否则为更新或插入值
	 */
	pub fn action<F>(&mut self, key: &K, func: &mut F) -> Option<ActionResultType> where F: FnMut(Option<&V>) -> ActionResult<V> {
		let r = self.root.action(key, func);
		match r {
			Some((rr, root)) => {
				self.root = root;
				Some(rr)
			},
			_ => None,
		}
	}
	// fn map(&mut self, func: &mut F) ->usize where F: FnMut(Option<&V>) {

}

//====================================

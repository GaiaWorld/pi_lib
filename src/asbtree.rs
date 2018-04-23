// #![feature(generators, generator_trait)]

/**
 * 写时复制的sbtree，支持单线程或多线程安全
 */

use std::option::Option;
use std::cmp::{Ord, Ordering};
//use std::rc::Rc;
//use std::ops::{Generator, GeneratorState};
use std::sync::Arc;
//use std::fmt::{Debug};
use ordmap::{ActionResult, ActionResultType, Entry, ImOrdMap, OrdMap};


pub type TreeMap<K, V> = OrdMap<Tree<K, V>>;
pub fn new<K: Ord+Clone, V: Clone>() -> TreeMap<K, V> {
	let tree:Tree<K, V> = Tree::new();
	OrdMap::new(tree)
}

pub type Tree<K, V> = Option<Arc<Node<K, V>>>;
#[inline]
pub fn new_tree<K: Clone, V: Clone>(n: Node<K, V>) -> Tree<K, V> {
	Some(Arc::new(n))
}
pub struct Node<K: Clone, V: Clone> {
	size: usize,
	left: Tree<K, V>,
	entry: Entry<K, V>,
	right: Tree<K, V>,
}

// pub type Root<K, V> = Option<Node<K, V>>;
// pub enum Node<K, V> {
//     One(Rc<Entry<K, V>>),
//     Two(Rc<(Entry<K, V>, Entry<K, V>)>),
//     Three(Rc<(usize, Node<K, V>, Entry<K, V>, Node<K, V>)>),
//     Five(Rc<(usize, Node<K, V>, Entry<K, V>, Node<K, V>, Entry<K, V>, Node<K, V>)>),
// }
// pub enum Node<K, V> {
//     Null,
//     Two(Rc<(usize, Node<K, V>, Entry<K, V>, Node<K, V>)>),
//     Three(Rc<(usize, Node<K, V>, Entry<K, V>, Node<K, V>, Entry<K, V>, Node<K, V>)>),
// }

impl<K: Ord+Clone, V: Clone> Node<K, V> {
	/**
	 * 新建
	 */
    fn new(s: usize, l: Tree<K, V>, e: Entry<K, V>, r: Tree<K, V>) -> Self {
		Node {
			size: s,
			left: l,
			entry: e,
			right: r,
		}
	}
	// 节点左旋
	//#[inline]
	fn left_ratote (size: usize, left: &Tree<K, V>, e: Entry<K, V>, right: &Self) -> Self {
		let lsize = match left {
			&Some(ref x) => (*x).size,
			_ => 0,
		};
		let rsize = match right.left {
			Some(ref x) => (*x).size,
			_ => 0,
		};
		Self::new(size, new_tree(Self::new(lsize + rsize + 1, left.clone(), e, right.left.clone())), right.entry.clone(), right.right.clone())
	}
	// 节点右旋
	//#[inline]
	fn right_ratote (size: usize, left: &Self, e: Entry<K, V>, right: &Tree<K, V>) -> Self {
		let rsize = match right {
			&Some(ref x) => (*x).size,
			_ => 0,
		};
		let lsize = match left.right {
			Some(ref x) => (*x).size,
			_ => 0,
		};
		Self::new(size, left.left.clone(), left.entry.clone(), new_tree(Self::new(lsize + rsize + 1, left.right.clone(), e, right.clone())))
	}

	//Maintain操作，Maintain(T)用于修复以T为根的 SBT。调用Maintain(T)的前提条件是T的子树都已经是SBT。
	// 左节点增加大小，Maintain操作
	//#[inline]
	fn maintain_left (size: usize, left: &Tree<K, V>, e: Entry<K, V>, right: &Tree<K, V>) -> Tree<K, V> {
		match right {
			&Some(ref x) => {
				match left {
					&Some(ref y) => {
						match (*y).left {
							Some(ref z) if (*z).size > (*x).size => return new_tree(Self::right_ratote(size, &*y, e, right)),
							_ => (),
						};
						match (*y).right {
							Some(ref z) if (*z).size > (*x).size => {
								return new_tree(Self::right_ratote(size, &Self::left_ratote((*y).size, &(*y).left, (*y).entry.clone(), &*z), e, right))
							},
							_ => (),
						}
					},
					_ => (),
				}
			},
			_ => {
				match left {
					&Some(ref x) if (*x).size > 1 => return new_tree(Self::right_ratote(size, &(*x), e, &None)),
					_ => (),
				}
			},
		};
		new_tree(Self::new(size, left.clone(), e, right.clone()))
	}
	// 右节点增加大小，Maintain操作
	//#[inline]
	fn maintain_right (size: usize, left: &Tree<K, V>, e: Entry<K, V>, right: &Tree<K, V>) -> Tree<K, V> {
		match left {
			&Some(ref x) => {
				match right {
					&Some(ref y) => {
						match (*y).right {
							Some(ref z) if (*z).size > (*x).size => return new_tree(Self::left_ratote(size, left, e, &(*y))),
							_ => (),
						};
						match (*y).left {
							Some(ref z) if (*z).size > (*x).size => {
								return new_tree(Self::left_ratote(size, left, e, &Self::right_ratote((*y).size, &*z, (*y).entry.clone(), &(*y).right)))
							},
							_ => (),
						}
					},
					_ => (),
				}
			},
			_ => {
				match right {
					&Some(ref x) if (*x).size > 1 => return new_tree(Self::left_ratote(size, &None, e, &(*x))),
					_ => (),
				}
			},
		};
		new_tree(Self::new(size, left.clone(), e, right.clone()))
	}
	// 节点删除操作，选Size大的子树旋转，旋转到叶子节点，然后删除
	fn delete(size: usize, left: &Tree<K, V>, right: &Tree<K, V>) -> Tree<K, V> {
		match left {
			&Some(ref l) => match right {
				&Some(ref r) => {
					if l.size > r.size {
						match l.right {
							Some(ref lr) => Self::maintain_right(size, &l.left, l.entry.clone(), &Self::delete(lr.size + r.size, &l.right, right)),
							_ => Self::maintain_right(size, &l.left, l.entry.clone(), &right),
						}
					}else {
						match r.left {
							Some(ref rl) => Self::maintain_left(size, &Self::delete(rl.size + l.size, left, &r.left), r.entry.clone(), &r.right),
							_ => Self::maintain_left(size, &left, r.entry.clone(), &r.right),
						}
					}
				},
				_ => left.clone(),
			},
			_ => right.clone(),
		}
	}

	fn select<F>(&self, func: &mut F) where F: FnMut(&Entry<K, V>) {
		match self.left {
			Some(ref x) => x.select(func),
			_ => ()
		};
		func(&self.entry);
		match self.right {
			Some(ref x) => x.select(func),
			_ => ()
		};
	}
	fn select_key<F>(&self, key: &K, func: &mut F) where F: FnMut(&Entry<K, V>) {

	}
	// 递归删除最小的键值对
	fn pop_min(&self, copy: bool) -> (Option<Entry<K, V>>, Tree<K, V>) {
		match self.left {
			Some(ref n) => {
				let (v, r) = n.pop_min(copy);
				(v, Self::maintain_right(self.size - 1, &r, self.entry.clone(), &self.right))
			},
			_ if copy => (Some(self.entry.clone()), self.right.clone()),
			_ => (None, self.right.clone()),
		}
	}
	// 递归删除最大的键值对
	fn pop_max(&self, copy: bool) -> (Option<Entry<K, V>>, Tree<K, V>) {
		match self.right {
			Some(ref n) => {
				let (v, r) = n.pop_max(copy);
				(v, Self::maintain_left(self.size - 1, &self.left, self.entry.clone(), &r))
			},
			_ if copy => (Some(self.entry.clone()), self.left.clone()),
			_ => (None, self.left.clone()),
		}
	}
	// 递归删除最大的键值对
	fn remove(&self, i: usize, copy: bool) -> (Option<Entry<K, V>>, Tree<K, V>) {
		if i == 0 {
			return self.pop_min(copy);
		}
		if i > self.size {
			return self.pop_max(copy);
		}
		match self.left {
			Some(ref n) if i > n.size => {
				let (v, r) = match self.right {
					Some(ref x) => x.remove(i - n.size, copy),
					_ => panic!("invalid tree"),
				};
				(v, Self::maintain_left(self.size - 1, &self.left, self.entry.clone(), &r))
			},
			Some(ref n) if i < n.size => {
				let (v, r) = n.remove(i, copy);
				(v, Self::maintain_right(self.size - 1, &r, self.entry.clone(), &self.right))
			},
			None => {
				let (v, r) = match self.right {
					Some(ref x) => x.remove(i - 1, copy),
					_ => panic!("invalid tree"),
				};
				(v, Self::maintain_left(self.size - 1, &self.left, self.entry.clone(), &r))
			}
			_ if copy => (Some(self.entry.clone()), Self::delete(self.size - 1, &self.left, &self.right)),
			_ => (None, Self::delete(self.size - 1, &self.left, &self.right)),
		}
	}

}



// impl<K: Ord+Clone, V: Clone>  Display for NSBTree {
// 	fn fmt(&self, f: &mut Formatter) -> Result;
// }

impl<K: Ord+Clone, V: Clone> ImOrdMap for Tree<K, V> { // 
	type Key = K;
	type Val = V;
	/**
	 * 新建
	 */
	fn new() -> Self {
		None
	}
	// /**
	//  * 克隆
	//  */
	// fn clone(&self) -> Self {
	// 	match self {
	// 		&Some(ref node) => Some(node.clone()),
	// 		_ => None,
	// 	}
	// }
	/**
	 * 判空
	 */
	fn is_empty(&self) -> bool {
		match self {
		  &None => true,
		  _ => false,
		}
	}
	/**
	 * 获取指定树的大小
	 */
	fn size(&self) -> usize {
		match self {
			&Some(ref node) => node.size,
			_ => 0,
		}
	}
	/**
	 * 检查指定的Key在树中是否存在
	 */
	fn has(&self, key: &K) -> bool {
		// 迭代查找
		let mut tree = self;
		loop {
			match tree {
				&Some(ref node) => match key.cmp(&node.entry.key()) {
					Ordering::Greater => tree = &node.right,
					Ordering::Less => tree = &node.left,
					_ => {
						return true;
					},
				},
				_ => break,
			}
		}
		false
	}
	/**
	 * 获取指定Key在树中的值
	 */
	fn get(&self, key: &K) -> Option<&V> {
		// 迭代查找
		let mut tree = self;
		loop {
			match tree {
				&Some(ref node) => match key.cmp(&node.entry.key()) {
					Ordering::Greater => tree = &node.right,
					Ordering::Less => tree = &node.left,
					_ => {
						return Some(&node.entry.value());
					},
				},
				_ => break,
			}
		}
		None
	}
	/**
	 * 获取树中最小的键值对
	 */
	fn min(&self) -> Option<&Entry<K, V>> {
		match self {
			&Some(ref n) => {
				// 迭代查找
				let mut node = n;
				loop {
					match node.left {
						Some(ref t) => node = &(*t),
						_ => break,
					}
				};
				Some(&node.entry)
			},
			_ => None
		}
	}
	/**
	 * 获取树中最小的键值对
	 */
	fn max(&self) -> Option<&Entry<K, V>> {
		match self {
			&Some(ref n) => {
				// 迭代查找
				let mut node = n;
				loop {
					match node.right {
						Some(ref t) => node = &(*t),
						_ => break,
					}
				};
				Some(&node.entry)
			},
			_ => None
		}
	}
	/**
	 * 获取指定Key在树中的排名，0表示空树，1表示第一名，负数表示没有该key，排名比该排名小
	 */
	fn rank(&self, key: &K) -> isize {
		match self {
			&Some(ref n) => {
				let mut node = n;
				let mut c: isize = 1;
				loop {
					match key.cmp(&node.entry.key()) {
						Ordering::Greater => {
							match node.left {
								Some(ref x) => c += ((*x).size as isize) + 1,
								_ => c += 1,
							};
							match node.right {
								Some(ref x) => node = &(*x),
								_ => break,
							}
						},
						Ordering::Less => match node.left {
							Some(ref x) => node = &(*x),
							_ => break,
						},
						_ => match node.left {
							Some(ref x) => return ((*x).size as isize) + c,
							_ => return c,
						},
					}
				}
				return -c;
			},
			_ => 0
		}
	}
	/**
	 * 获取指定排名的键值，必须从1开始，如果超过最大排名，则返回None
	 */
	fn index(&self, i: usize) -> Option<&Entry<K, V>>{
		if i == 0 {
			return None;
		}
		if i == 1 {
			return self.min();
		}
		match self {
			&Some(ref n) => {
				if i > n.size {
					return None;
				}
				if i == n.size {
					return self.max();
				}
				let mut node = n;
				let mut j = i - 1;
				loop {
					match node.left {
						Some(ref x) => {
							let c = (*x).size;
							if j > c {
								j -= c + 1;
								match node.right {
									Some(ref y) => node = &(*y),
									_ => break,
								}
							}else if j < c {
								node = &(*x);
							}else{
								break;
							}
						},
						None => {
							if j == 0 {
								break;
							}
							j -= 1;
							match node.right {
								Some(ref x) => node = &(*x),
								_ => break,
							}
						},
					}
				}
				Some(&node.entry)
			},
			_ => None
		}
	}
	/**
	 * 选择器方法，从指定键开始进行选择，如果不指定键，则从最小键开始
	 */
	fn select<F>(&self, key: Option<&K>, func: &mut F) where F: FnMut(&Entry<K, V>) {
		match self {
			&Some(ref node) => {
				match key {
					Some(ref k) => node.select_key(k, func),
					_ => node.select(func),
				};
			},
			_ => (),
		}
	}

	// 递归插入
	fn insert(&self, key: K, value: V) -> Option<Self> {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.key()) {
				Ordering::Greater => match node.right.insert(key, value) {
					Some(r) => Some(Node::maintain_right(node.size + 1, &node.left, node.entry.clone(), &r)),
					_ => None,
				},
				Ordering::Less => match node.left.insert(key, value) {
					Some(r) => Some(Node::maintain_left(node.size + 1, &r, node.entry.clone(), &node.right)),
					_ => None,
				},
				_ => None,
			},
			_ => Some(new_tree(Node::new(1, None, Entry::new(key, value), None))),
		}
	}
	// 递归更新
	fn update(&self, key: K, value: V, copy: bool) -> Option<(Option<V>, Self)> {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.key()) {
				Ordering::Greater => match node.right.update(key, value, copy) {
					Some((v, r)) => Some((v, new_tree(Node::new(node.size, node.left.clone(), node.entry.clone(), r)))),
					_ => None,
				},
				Ordering::Less => match node.left.update(key, value, copy) {
					Some((v, r)) => Some((v, new_tree(Node::new(node.size, r, node.entry.clone(), node.right.clone())))),
					_ => None,
				},
				_ if copy => Some((Some(node.entry.value().clone()), new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone())))),
				_ => Some((None, new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone())))),
			},
			_ => None,
		}
	}
	// 递归放入
	fn upsert(&self, key: K, value: V, copy: bool) -> (Option<Option<V>>, Self) {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.key()) {
				Ordering::Greater => match node.right.upsert(key, value, copy) {
					(None, r) => (None, Node::maintain_right(node.size + 1, &node.left, node.entry.clone(), &r)),
					(v, r) => (v, new_tree(Node::new(node.size, node.left.clone(), node.entry.clone(), r))),
				},
				Ordering::Less => match node.left.upsert(key, value, copy) {
					(None, r) => (None, Node::maintain_left(node.size + 1, &r, node.entry.clone(), &node.right)),
					(v, r) => (v, new_tree(Node::new(node.size, r, node.entry.clone(), node.right.clone()))),
				},
				_ if copy => (Some(Some(node.entry.value().clone())), new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone()))),
				_ => (Some(None), new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone()))),
			},
			_ => (None, new_tree(Node::new(1, None, Entry::new(key, value), None))),
		}
	}

	// 递归删除
	fn delete(&self, key: &K, copy: bool) -> Option<(Option<V>, Self)> {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.key()) {
				Ordering::Greater => match node.right.delete(key, copy) {
					Some((v, r)) => Some((v, Node::maintain_left(node.size - 1, &node.left, node.entry.clone(), &r))),
					_ => None,
				},
				Ordering::Less => match node.left.delete(key, copy) {
					Some((v, r)) => Some((v, Node::maintain_right(node.size - 1, &r, node.entry.clone(), &node.right))),
					_ => None,
				},
				_ if copy => Some((Some(node.entry.value().clone()), Node::delete(node.size - 1, &node.left, &node.right))),
				_ => Some((None, Node::delete(node.size - 1, &node.left, &node.right))),
			},
			_ => None,
		}
	}
	// 递归删除
	fn remove(&self, i: usize, copy: bool) -> Option<(Option<Entry<K, V>>, Self)> {
		if i == 0 {
			return None;
		}
		match self {
			&Some(ref n) => {
				if i > n.size {
					return None;
				}
				Some(n.remove(i - 1, copy))
			},
			_ => None,
		}
	}
	// 递归删除最小的键值对
	fn pop_min(&self, copy: bool) -> Option<(Option<Entry<K, V>>, Self)> {
		match self {
			&Some(ref node) => Some(node.pop_min(copy)),
			_ => None,
		}
	}
	// 递归删除最大的键值对
	fn pop_max(&self, copy: bool) -> Option<(Option<Entry<K, V>>, Self)> {
		match self {
			&Some(ref node) => Some(node.pop_max(copy)),
			_ => None,
		}
	}
	fn action<F>(&self, key: &K, func: &mut F) -> Option<(ActionResultType, Self)> where F: FnMut(Option<&V>) -> ActionResult<V> {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.key()) {
				Ordering::Greater => match node.right.action(key, func) {
					Some((ActionResultType::Insert, r)) => Some((ActionResultType::Insert, Node::maintain_right(node.size + 1, &node.left, node.entry.clone(), &r))),
					Some((ActionResultType::Update, r)) => Some((ActionResultType::Update, new_tree(Node::new(node.size, node.left.clone(), node.entry.clone(), r)))),
					Some((ActionResultType::Delete, r)) => Some((ActionResultType::Delete, Node::maintain_left(node.size - 1, &node.left, node.entry.clone(), &r))),
					_ => None,
				},
				Ordering::Less => match node.left.action(key, func) {
					Some((ActionResultType::Insert, r)) => Some((ActionResultType::Insert, Node::maintain_left(node.size + 1, &r, node.entry.clone(), &node.right))),
					Some((ActionResultType::Update, r)) => Some((ActionResultType::Update, new_tree(Node::new(node.size, r, node.entry.clone(), node.right.clone())))),
					Some((ActionResultType::Delete, r)) => Some((ActionResultType::Delete, Node::maintain_right(node.size - 1, &r, node.entry.clone(), &node.right))),
					_ => None,
				},
				_ => match func(Some(node.entry.value())) {
					ActionResult::Upsert(r) => Some((ActionResultType::Update, new_tree(Node::new(node.size, node.left.clone(), Entry::new(key.clone(), r), node.right.clone())))),
					ActionResult::Delete => Some((ActionResultType::Delete, Node::delete(node.size - 1, &node.left, &node.right))),
					_ => None,
				},
			},
			_ => match func(None) {
				ActionResult::Upsert(r) => Some((ActionResultType::Insert, new_tree(Node::new(1, None, Entry::new(key.clone(), r), None)))),
				_ => None,
			},
		}
	}

}

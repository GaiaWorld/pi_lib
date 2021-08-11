
use std::rc::Rc;
#[macro_export]
macro_rules! custom_ref { ($x:ident) => (

use std::cmp::{Ord, Ordering};
//use std::ops::{Generator, GeneratorState};
use std::marker::PhantomData;
use std::mem::zeroed;
use std::ops::Deref;

//use std::fmt::{Debug};
use ordmap::{ActionResult, ActionResultType, Entry, ImOrdMap, Iter};


#[inline]
pub fn new_tree<K: Clone, V: Clone>(n: Node<K, V>) -> Tree<K, V> {
	Some($x::new(n))
}
/// 写时复制的sbtree，支持单线程或多线程安全
pub type Tree<K, V> = Option<$x<Node<K, V>>>;

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

	fn actions<F>(node: &Tree<K, V>,  func: &mut F) -> Option<Tree<K, V>> where F: FnMut(&Entry<K, V>) -> ActionResult<V>{
		match node{
			Some(n) => {
				let result = func(&n.entry);
				let (left, right) = match Self::actions(&n.left, func) {
					Some(l) => {
						match Self::actions(&n.right, func) {
							Some(r) => (l, r),
							None => (l, n.right.clone())
						}
					}
					None => {
						match Self::actions(&n.right, func) {
							Some(r) => (n.left.clone(), r),
							None => {
								match result{
									ActionResult::Ignore => return None,
									_ => (n.left.clone(), n.right.clone()),
								}
							}
						}
					}
				};

				let l_size = match &left {
					&Some(ref l) => l.size,
					None => 0,
				};

				let r_size = match &right {
					&Some(ref r) => r.size,
					None => 0,
				};
		
				let root = match result{
					ActionResult::Ignore => Node::new(1 + r_size + l_size, left, n.entry.clone(), right),
					ActionResult::Delete => {
						match Node::delete(r_size + l_size, &left, &right) {
							Some(e) => Node::new(e.size, e.left.clone(), e.entry.clone(), e.right.clone()),
							None => return Some(None),
						}
					},
					ActionResult::Upsert(val) => Node::new(1 + r_size + l_size, left, Entry(n.entry.0.clone(), val), right),
				};

				Some(new_tree(Node::ratotes(root)))
			},
			None => None
		}
	}

	//多次旋转
	fn ratotes (self) -> Self {
		match &self.left {
			&Some(ref l) => {
				match &self.right {
					&Some(ref r) => {
						match (*r).right {
							Some(ref rr) if (*rr).size > (*l).size => return Self::ratotes(Self::left_ratote(self.size, &self.left, self.entry.clone(), &*r)),
							_ => (),
						};

						match (*r).left {
							Some(ref rl) if (*rl).size > (*r).size => {
								return Self::ratotes(Self::left_ratote(self.size, &self.left, self.entry.clone() , &Self::right_ratote((*r).size, &*rl, (*r).entry.clone(), &(*r).left)))
							},
							_ => (),
						}

						match (*l).left {
							Some(ref ll) if (*ll).size > (*r).size => return Self::ratotes(Self::right_ratote(self.size, &*l, self.entry.clone(), &self.right)),
							_ => (),
						};
						
						match (*l).right {
							Some(ref lr) if (*lr).size > (*r).size => {
								return Self::ratotes(Self::right_ratote(self.size, &Self::left_ratote((*l).size, &(*r).left , (*l).entry.clone(), &*lr), self.entry.clone(), &self.right))
							},
							_ => (),
						}
					},
					_ => (),
				}
			},
			_ => {
				match &self.right {
					&Some(ref r) if (*r).size > 1 => return Self::ratotes(Self::left_ratote(self.size, &None, self.entry.clone(), &(*r))),
					_ => (),
				}
			},
		};
		self
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

impl< K: Ord+Clone, V: Clone> ImOrdMap for Tree<K, V> { // 
	type Key = K;
	type Val = V;
	/**
	 * 新建
	 */
	fn new() -> Self {
		None
	}

	fn from_order(mut arr: Vec<Entry<K, V>>) -> Self{
		match arr.len(){
			0 => None,
			_ => {
				arr.sort();
				let len = arr.len();
				new_tree(creat_node(&mut arr, 0, len))
			}
		}
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
				&Some(ref node) => match key.cmp(&node.entry.0) {
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
				&Some(ref node) => match key.cmp(&node.entry.0) {
					Ordering::Greater => tree = &node.right,
					Ordering::Less => tree = &node.left,
					_ => {
						return Some(&node.entry.1);
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
					match key.cmp(&node.entry.0) {
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

	// 递归插入
	fn insert(&self, key: K, value: V) -> Option<Self> {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.0) {
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
			&Some(ref node) => match key.cmp(&node.entry.0) {
				Ordering::Greater => match node.right.update(key, value, copy) {
					Some((v, r)) => Some((v, new_tree(Node::new(node.size, node.left.clone(), node.entry.clone(), r)))),
					_ => None,
				},
				Ordering::Less => match node.left.update(key, value, copy) {
					Some((v, r)) => Some((v, new_tree(Node::new(node.size, r, node.entry.clone(), node.right.clone())))),
					_ => None,
				},
				_ if copy => Some((Some(node.entry.1.clone()), new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone())))),
				_ => Some((None, new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone())))),
			},
			_ => None,
		}
	}
	// 递归放入
	fn upsert(&self, key: K, value: V, copy: bool) -> (Option<Option<V>>, Self) {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.0) {
				Ordering::Greater => match node.right.upsert(key, value, copy) {
					(None, r) => (None, Node::maintain_right(node.size + 1, &node.left, node.entry.clone(), &r)),
					(v, r) => (v, new_tree(Node::new(node.size, node.left.clone(), node.entry.clone(), r))),
				},
				Ordering::Less => match node.left.upsert(key, value, copy) {
					(None, r) => (None, Node::maintain_left(node.size + 1, &r, node.entry.clone(), &node.right)),
					(v, r) => (v, new_tree(Node::new(node.size, r, node.entry.clone(), node.right.clone()))),
				},
				_ if copy => (Some(Some(node.entry.1.clone())), new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone()))),
				_ => (Some(None), new_tree(Node::new(node.size, node.left.clone(), Entry::new(key, value), node.right.clone()))),
			},
			_ => (None, new_tree(Node::new(1, None, Entry::new(key, value), None))),
		}
	}

	// 递归删除
	fn delete(&self, key: &K, copy: bool) -> Option<(Option<V>, Self)> {
		match self {
			&Some(ref node) => match key.cmp(&node.entry.0) {
				Ordering::Greater => match node.right.delete(key, copy) {
					Some((v, r)) => Some((v, Node::maintain_left(node.size - 1, &node.left, node.entry.clone(), &r))),
					_ => None,
				},
				Ordering::Less => match node.left.delete(key, copy) {
					Some((v, r)) => Some((v, Node::maintain_right(node.size - 1, &r, node.entry.clone(), &node.right))),
					_ => None,
				},
				_ if copy => Some((Some(node.entry.1.clone()), Node::delete(node.size - 1, &node.left, &node.right))),
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
			&Some(ref node) => match key.cmp(&node.entry.0) {
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
				_ => match func(Some(&node.entry.1)) {
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

	fn map<F>(&self, func: &mut F) -> Self where F: FnMut(&Entry<Self::Key, Self::Val>) -> ActionResult<Self::Val>{
		match Node::actions(self, func){
			Some(node) => node,
			None => self.clone()
		}
	}
}
//升序或降序引用迭代器
pub struct IterTree<'a, K: 'a + Clone, V: 'a + Clone>{
    arr: [*const Node<K, V>; 32],
	len: usize,
	next_fn: fn(&mut IterTree<'a, K, V>) -> Option<&'a Entry<K, V>>,
	marker: PhantomData<&'a Node<K, V>>
}

unsafe impl<'a, K: 'a + Clone, V: 'a + Clone> Send for IterTree<'a, K, V> {}

impl<'a, K: 'a + Clone, V: 'a + Clone> IterTree<'a, K, V>{
	fn next_ascending(it: &mut IterTree<'a, K, V>) -> Option<&'a Entry<K, V>>{
		match it.len{
			0 => return None,
			_ => {
				it.len -= 1;
				let node = unsafe{&*it.arr[it.len]};
				match node.right{
					Some(ref right) => {
						it.arr[it.len] = right.deref() as *const Node<K, V>;
						it.len += 1;
						it.len += down_l(&mut it.arr, &right, it.len);
					}
					None => {}
				};
				return Some(&node.entry);
			},
		}
	}

	fn next_descending(it: &mut IterTree<'a, K, V>) -> Option<&'a Entry<K, V>>{
		match it.len{
			0 => return None,
			_ => {
				it.len -= 1;
				let node = unsafe{&*it.arr[it.len]};
				match node.left{
					Some(ref left) => {
						it.arr[it.len ] = left.deref() as *const Node<K, V>;
						it.len += 1;
						it.len += down_r(&mut it.arr, &left, it.len);
					}
					None => {}
				};
				return Some(&node.entry);
			},
		}
	}
}

impl<'a, K: Clone, V: Clone> Iterator for IterTree<'a, K, V>{
	type Item = &'a Entry<K, V>;
	fn next(&mut self) -> Option<Self::Item>{
		(self.next_fn)(self)
	}
}

impl<'a, K: 'a + Clone + Ord, V: 'a + Clone> Iter<'a> for Tree<K, V>{
	type K = K;
	type V = V;
	type IterType = IterTree<'a, K, V>;
	fn iter(&self, key: Option<&K>, descending: bool) -> Self::IterType{
		let mut it = IterTree{
			arr: unsafe{zeroed()},
			len: 0,
			next_fn: IterTree::next_ascending,
			marker: PhantomData
		};
		match self {
			Some(ref node) => {
				match key{
					Some(k) => {
						match descending{
							true => {it.len = some_down_key_r(&mut it.arr, node, k, it.len); it.next_fn = IterTree::next_descending;},
							false => it.len = some_down_key_l(&mut it.arr, node, k, it.len)
						};
					},
					None => {
						it.arr[it.len] = node.deref();
						it.len += 1;
						match descending{
							true => {it.len += down_r(&mut it.arr, node, it.len); it.next_fn = IterTree::next_descending;},
							false => it.len += down_l(&mut it.arr, node, it.len)
						};
					}
				}
			},
			_ => {}
		}
		it
	}
} 

fn creat_node<K: Ord + Clone, V: Clone>(arr: &mut Vec<Entry<K, V>>, start: usize, len: usize) -> Node<K, V>{
	let r_size = (len-1)/2;
	let l_size = len - r_size - 1;
	let index = start + l_size;

	let r_node = match r_size{
		0 => None,
		_ => new_tree(creat_node(arr, index + 1, r_size))
	};

	let mut root = Node::new(len, None, arr.pop().unwrap(), r_node);

	root.left = match l_size{
		0 => None,
		_ => {new_tree(creat_node(arr, index - l_size, l_size))}
	};
	root
}

fn down_r<'a, K: Clone, V: Clone>(arr: &mut [*const Node<K, V>; 32], last: &'a Node<K, V>, index: usize) -> usize{
	match last.right{
		Some(ref v) => {
			arr[index] = v.deref();
			down_r(arr, v, index + 1) + 1
		},
		None => 0,
	}
}

fn down_l<'a, K: Clone, V: Clone>(arr: &mut [*const Node<K, V>; 32], last: &'a Node<K, V>, index: usize) -> usize{
	match last.left{
		Some(ref v) => {
			arr[index] = v.deref();
			down_l(arr, v, index + 1) + 1
		},
		None => 0,
	}
}

fn down_key_l<'a, K: Ord + Clone, V: Clone>(arr: &mut [*const Node<K, V>; 32], last: &'a Node<K, V>, key: &K, index: usize) -> usize{
	match last.left{
		Some(ref v) => {
			some_down_key_l(arr, v, key, index)
		},
		None => 0
	}
}

fn down_key_r<'a, K: Ord + Clone, V: Clone>(arr: &mut [*const Node<K, V>; 32], last: &'a Node<K, V>, key: &K, index: usize) -> usize{
	match last.right{
		Some(ref v) => {
			some_down_key_r(arr, v, key, index)
		},
		None => 0
	}
}

fn some_down_key_l<'a, K: Ord + Clone, V: Clone>(arr: &mut [*const Node<K, V>; 32], v: &'a Node<K, V>, key: &K, index: usize) -> usize{
	match key.cmp(&v.entry.0) {
		Ordering::Less => {
			arr[index] = v;
			return down_key_l(arr, v, key, index + 1) + 1;
		},
		Ordering::Greater => {
			match v.right{
				Some(ref r) => return some_down_key_l(arr, r, key, index),
				None => return 0,
			}
		},
		Ordering::Equal => {
			arr[index] = v;
			return 1;
		}
	} 
}

fn some_down_key_r<'a, K: Ord + Clone, V: Clone>(arr: &mut [*const Node<K, V>; 32], v: &'a Node<K, V>, key: &K, index: usize) -> usize{
	match key.cmp(&v.entry.0) {
		Ordering::Less => {
			match v.left{
				Some(ref l) => return some_down_key_r(arr, l, key, index),
				None => return 0,
			}
		},
		Ordering::Greater => {
			arr[index] = v;
			return down_key_r(arr, v, key, index + 1) + 1;
		},
		Ordering::Equal => {
			arr[index] = v;
			return 1;
		}
	} 
}

)}

custom_ref!(Rc);
pub fn new<K: Clone+Ord, V: Clone>() -> Tree<K, V> {
	None
}

#[test]
pub fn test_sbtree(){
	use ordmap::{ActionResult, Entry, ImOrdMap, Iter};
	//测试迭代---------------------------------------------------------------------------------
	let mut tree = Tree::new();
	for i in 1..101{
		let r = tree.insert(i,i);
		match r{
			Some(t) => {tree = t;}
			None => {}
		}
	}

	let mut i = 1;
	for v in Iter::iter(&mut tree, None, false) {//升序迭代
		//print!("{},", v.0);
		assert_eq!(v.0, i);
		i += 1;
	}

	let mut i = 100;
	for v in Iter::iter(&mut tree, None, true){//降序迭代
		//print!("{},", v.0);
		assert_eq!(v.0, i);
		i -= 1;
	}

	let mut i = 50;
	for v in Iter::iter(&mut tree, Some(&50), false){//从指定键升序迭代
		//print!("{},", v.0);
		assert_eq!(v.0, i);
		i += 1;
	}
	assert_eq!(i, 101);

	let mut i = 50;
	for v in  Iter::iter(&mut tree, Some(&50), true){//从指定键降序迭代
		assert_eq!(v.0, i);
		i -= 1;
	}
	assert_eq!(i, 0);

	//测试from_order---------------------------------------------------------------------------------
	let mut arr = Vec::new();
	for i in 1..100{
		arr.push(Entry(i, i));
	}
	let mut tree = Tree::from_order(arr);
	let mut i = 1;
	for v in  Iter::iter(&mut tree, None, false){
		//print!("{},", v.0);
		assert_eq!(v.0, i);
		i += 1;
	}

	//测试map---------------------------------------------------------------------------------
	let mut arr = Vec::new();
	for i in 1..101{
		arr.push(Entry(i, i));
	}
	let tree = Tree::from_order(arr);
	let mut tree_new = <Tree<i32, i32> as ImOrdMap>::map(&tree, &mut |el: &Entry<i32, i32>|{ 
		if el.0 < 10 || el.0 == 45 || el.0 == 52{
			return ActionResult::Upsert(1000);
		}else if el.0 > 90 || el.0 == 30 || el.0 == 58{
		 	return ActionResult::Delete;
		}else{
			return ActionResult::Ignore
		}
	});
	let mut i = 1;
	for el in  Iter::iter(&mut tree_new, None, false){
		//print!("{},", el.1);
		if el.0 < 10 || el.0 == 45 || el.0 == 52{
			assert_eq!(el.1, 1000);
		}else{
			assert_eq!(el.1, i);
		}

		if el.0 == 29 || el.0 == 57{
		 	i += 1;
		}
		i += 1;
	}
}
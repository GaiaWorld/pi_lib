extern crate slotmap;

use std::{ops::{Index, IndexMut}, fmt::Debug};

use slotmap::{Key, SecondaryMap};

pub enum InsertType {
    Back,
    Front,
}

#[derive(Default)]
pub struct Tree<K: Key + Debug, T: Default> {
    map: SecondaryMap<K, Node<K, T>>,
    statistics_count: bool,
}

impl<K: Key + Debug, T: Default> Index<K> for Tree<K, T> {
    type Output = Node<K, T>;

    fn index(&self, index: K) -> &Node<K, T> {
        &self.map[index]
    }
}

impl<K: Key, T: Default> IndexMut<K> for Tree<K, T> {
    fn index_mut(&mut self, index: K) -> &mut Node<K, T> {
        &mut self.map[index]
    }
}

pub trait Empty {
	fn empty() -> Self;
}

impl<K: Key + Empty, T: Default> Tree<K, T> {
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			map: SecondaryMap::with_capacity(capacity),
    		statistics_count: false,
		}
	}

    pub fn is_statistics_count(&self) -> bool {
        self.statistics_count
    }
    pub fn set_statistics_count(&mut self, b: bool) {
        self.statistics_count = b
    }
    pub fn get(&self, id: K) -> Option<&Node<K, T>> {
        self.map.get(id)
    }
    pub fn get_mut(&mut self, id: K) -> Option<&mut Node<K, T>> {
        self.map.get_mut(id)
    }
    pub unsafe fn get_unchecked(&self, id: K) -> &Node<K, T> {
        self.map.get_unchecked(id)
    }
    pub unsafe fn get_unchecked_mut(&mut self, id: K) -> &mut Node<K, T> {
        self.map.get_unchecked_mut(id)
    }
    pub fn create(&mut self, id: K) {
		let node = Node::default();
        self.map.insert(id, node);
    }
    /// index为0表示插入到子节点队列前， 如果index大于子节点队列长度，则插入到子节点队列最后。parent如果为0 表示设置为根节点。 如果parent的layer大于0
	/// order表示在子节点中的顺序，当大于子节点长度时，插入到队列最后
    pub fn insert_child(&mut self, id: K, parent: Option<K>, mut order: usize) -> usize {
        if let Some(parent) = parent {
            let (layer, prev, next) = match self.map.get(parent) {
                Some(n) if order >= n.children.len => (
                    if n.layer > 0 { n.layer + 1 } else { 0 },
                    n.children.tail,
                    None,
                ),
                Some(n) if order + order >= n.children.len => {
                    let mut prev = n.children.tail;
                    let mut next = None;
                    order = n.children.len - order;
                    while order < usize::max_value() && prev.is_some(){
                        order -= 1;
                        next = prev;
                        let node = unsafe { self.map.get_unchecked(next.clone().unwrap()) };
                        prev = node.prev;
                    }
                    (if n.layer > 0 { n.layer + 1 } else { 0 }, prev, next)
                }
                Some(n) => {
                    let mut prev = None;
                    let mut next = n.children.head;
                    while order < usize::max_value() && next.is_some() {
                        order -= 1;
                        prev = next;
                        let node = unsafe { self.map.get_unchecked(prev.unwrap()) };
                        next = node.next;
                    }
                    (if n.layer > 0 { n.layer + 1 } else { 0 }, prev, next)
                }
                _ => panic!("invalid parent: {:?}", parent),
            };
            self.insert_node(id, parent, layer, prev, next)
        } else {
            self.insert_root(id)
        }
    }
    /// 根据InsertType插入到brother的前或后。 brother的layer大于0
    pub fn insert_brother(&mut self, id: K, brother: K, insert: InsertType) -> usize {
        let (parent, layer, prev, next) = match self.map.get(brother) {
            Some(n) => match insert {
                InsertType::Front => (n.parent, n.layer, n.prev, Some(brother)),
                InsertType::Back => (n.parent, n.layer, Some(brother), n.next),
            },
            _ => panic!("invalid brother: {:?}", brother),
        };
        if let Some(parent) = parent {
            self.insert_node(id, parent, layer, prev, next)
        } else {
            self.insert_root(id)
        }
    }
    /// 获得节点信息， 一般用于remove和destroy
    pub fn get_info(&mut self, id: K) -> Option<(Option<K>, usize, usize, Option<K>, Option<K>, Option<K>)> {
        match self.map.get(id) {
            Some(n) => Some((n.parent, n.layer, n.count, n.prev, n.next, n.children.head)),
            _ => return None,
        }
    }
    /// 如果的节点的layer大于0，表示在树上
    pub fn remove(
        &mut self,
        id: K,
        (parent, layer, count, prev, next, head): (Option<K>, usize, usize, Option<K>, Option<K>, Option<K>),
    ) {
        if layer > 0 {
            self.remove_tree(head);
        }
        if let Some(parent) = parent {
            self.remove_node(parent, count + 1, prev, next)
        }
        let node = unsafe { self.map.get_unchecked_mut(id) };
        node.parent = None;
        node.layer = 0;
        node.prev = None;
        node.next = None;
    }
    /// 销毁子节点， recursive表示是否递归销毁
    pub fn destroy(
        &mut self,
        id: K,
        (parent, layer, count, prev, next, mut head): (Option<K>, usize, usize, Option<K>, Option<K>, Option<K>),
        recursive: bool,
    ) {
        if recursive {
            self.recursive_destroy(id, head);
        } else {
            self.map.remove(id);
            if layer > 0 {
                while let Some(head_value) = head {
                    let child = {
                        let n = unsafe { self.map.get_unchecked_mut(head_value) };
                        n.parent= None;
                        n.layer = 0;
                        head = n.next;
                        n.prev = None;
                        n.next = None;
                        n.children.head
                    };
                    self.remove_tree(child);
                }
            } else {
                while let Some(head_value) = head {
                    let n = unsafe { self.map.get_unchecked_mut(head_value) };
                    n.parent= None;
                    head = n.next;
                    n.prev = None;
                    n.next = None;
                }
            }
        }
        if let Some(parent) = parent {
            self.remove_node(parent, count + 1, prev, next)
        }
    }
    /// 迭代指定节点的所有子元素
    pub fn iter_mut(&mut self, node_children_head: Option<K>) -> ChildrenMutIterator<K, T> {
        ChildrenMutIterator {
            inner: &mut self.map,
            head: node_children_head,
        }
    }
    /// 迭代指定节点的所有子元素
    pub fn iter(&self, node_children_head: Option<K>) -> ChildrenIterator<K, T> {
        ChildrenIterator {
            inner: &self.map,
            head: node_children_head,
        }
    }
    /// 迭代指定节点的所有递归子元素
    pub fn recursive_iter(&self, node_children_head: Option<K>) -> RecursiveIterator<K, T> {
		let (head, len) = match node_children_head {
			Some(h) => (h, 1),
			None => (K::empty(), 0)
		};
        RecursiveIterator {
            inner: &self.map,
            arr: [
                head,
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
                K::empty(),
            ],
            len,
        }
    }
    fn insert_root(&mut self, id: K) -> usize {
		log::info!("zzzzzzzzzzzzzzzzzzzzzzzzzzzz");
        // 设置为根节点
        let head = match self.map.get_mut(id) {
            Some(n) => {
				log::info!("n.parent: {:?}, layer:{:?}", n.parent, n.layer);
                if n.parent.is_some() {
                    panic!("has a parent node, id: {:?}", id)
                }
                if n.layer > 0 {
                    panic!("already on the tree, id: {:?}", id)
                }
                n.layer = 1;
                n.children.head
            }
            _ => {
				log::info!("xxxxxxxxxxxxxxxxxxx");
				panic!("invalid id: {:?}", id);
			}
        };
		log::info!("bbbbbbbbbbbbbbbbbbbb");
        self.insert_tree(head, 2);
        1
    }
    // 插入节点, 如果id就在parent内则为调整位置
    fn insert_node(
        &mut self,
        id: K,
        parent: K,
        layer: usize,
        prev: Option<K>,
        next: Option<K>,
    ) -> usize {
		// // 调用该方法，该节点可能已经存在，并且是将该节点插入到原位置
		// // 如果插入到原位置，则无需操作
		// if id == prev || id == next {
		// 	return layer;
		// }
        let (count, fix_prev, fix_next) = match self.map.get_mut(id) {
            Some(n) => {
				if let Some(n_parent) = n.parent {
					if n_parent != parent {
                        panic!("has a parent node, id: {:?}", id)
                    }

					// 调整
                    let fix_prev = n.prev;
                    let fix_next = n.next;
                    n.prev = prev;
					n.next = next;
                    (0, fix_prev, fix_next)
				} else {
					if n.layer > 0 {
                        panic!("already on the tree, id: {:?}", id)
                    }
					// 不存在父节，直接挂在树上
					n.parent = Some(parent);
                    n.layer = layer;
                    n.prev = prev;
					n.next = next;
                    (n.count + 1, n.children.head, None)
				}
            }
            _ => panic!("invalid id: "),//panic!("invalid id: {}", id),
		};
        // 修改prev和next的节点
        if let Some(prev) = prev {
            let node = unsafe { self.map.get_unchecked_mut(prev) };
            node.next = Some(id);
        }
        if let Some(next) = next {
            let node = unsafe { self.map.get_unchecked_mut(next) };
            node.prev = Some(id);
        }
        if count == 0 {
            // 同层调整
            if let Some(fix_prev) = fix_prev {
                let node = unsafe { self.map.get_unchecked_mut(fix_prev) };
                node.next = fix_next;
            }
            if let Some(fix_next) = fix_next {
                let node = unsafe { self.map.get_unchecked_mut(fix_next) };
                node.prev = fix_prev;
            }

            if prev.is_none() || next.is_none() || fix_prev.is_none() || fix_next.is_none() {
                let node = unsafe { self.map.get_unchecked_mut(parent) };
                if prev.is_none() {
                    node.children.head = Some(id);
                } else if fix_prev.is_none() {
                    node.children.head = fix_next;
                }
                if next.is_none() {
                    node.children.tail = Some(id);
                } else if fix_next.is_none() {
                    node.children.tail = fix_prev;
                }
            }
            return layer;
        }
        let p = {
            // 修改parent的children, count
			let node = unsafe { self.map.get_unchecked_mut(parent) };
            if prev.is_none() {
                node.children.head = Some(id);
            }
            if next.is_none() {
                node.children.tail =Some(id);
            }
            node.children.len += 1;
            node.count += count;
            node.parent
        };
        if self.statistics_count {
            // 递归向上修改count
            self.modify_count(p, count as isize);
        }
        if layer > 0 {
            self.insert_tree(fix_prev, layer + 1);
		}
        layer
    }
    // 插入到树上， 就是递归设置每个子节点的layer
    fn insert_tree(&mut self, mut id: Option<K>, layer: usize) {
        while let Some(id_value) = id {
            let head = {
                let n = unsafe { self.map.get_unchecked_mut(id_value) };
                n.layer = layer;
                id = n.next;
                n.children.head
            };
            self.insert_tree(head, layer + 1);
        }
    }
    // 从树上移除， 就是递归设置每个子节点的layer为0
    fn remove_tree(&mut self, mut id: Option<K>) {
        while let Some(id_value) = id {
            let head = {
                let n = unsafe { self.map.get_unchecked_mut(id_value) };
                n.layer = 0;
                id = n.next;
                n.children.head
            };
            self.remove_tree(head);
        }
    }
    // 递归销毁
    fn recursive_destroy(&mut self, parent: K, mut id: Option<K>) {
		self.map.remove(parent);
        while let Some(id_value) = id {
            let (next, head) = {
                let n = unsafe { self.map.get_unchecked(id_value) };
                (n.next, n.children.head)
            };
            self.recursive_destroy(id_value, head);
            id = next;
        }
    }
    // 递归向上，修改节点的count
    fn modify_count(&mut self, mut id: Option<K>, count: isize) {
        while let Some(id_value) = id {
            let n = unsafe { self.map.get_unchecked_mut(id_value) };
            n.count = (n.count as isize + count) as usize;
            id = n.parent;
        }
    }
    // 移除节点
    fn remove_node(&mut self, parent: K, count: usize, prev: Option<K>, next: Option<K>) {
        // 修改prev和next的节点
        if let Some(prev) = prev {
            let node = unsafe { self.map.get_unchecked_mut(prev) };
			node.next = next;
        }
        if let Some(next) = next {
            let node = unsafe { self.map.get_unchecked_mut(next) };
			node.prev = prev;
        }
        
            // 修改parent的children, count
            let node = unsafe { self.map.get_unchecked_mut(parent) };
            if prev.is_none() {
                node.children.head = next;
            }
            if next.is_none() {
                node.children.tail = prev;
            }
            node.children.len -= 1;
            let p = node.parent;
            

        if self.statistics_count {
			node.count -= count;
            // 递归向上修改count
            self.modify_count(p, -(count as isize));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node<K: Key, T: Default> {
    parent: Option<K>,      // 父节点
    layer: usize,       // 表示第几层，如果不在根上，则为0。 在根上，则起步为1
    count: usize,       // 所有的递归子节点的总数量
    prev: Option<K>,        // 前ab节点
    next: Option<K>,        // 后ab节点
    children: NodeList<K>, // 子节点列表
    pub data: T,
}

impl<K: Key, T: Default> Default for Node<K, T> {
	fn default() -> Self{
		Node {
			parent: None,
			layer: 0,
			count: 0,
			prev: None,
			next: None,
			children: NodeList::default(),
			data: T::default(),
		}
	}
}
impl<K: Key, T: Default> Node<K, T> {
    pub fn parent(&self) -> Option<K> {
        self.parent
    }
    pub fn layer(&self) -> usize {
        self.layer
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn prev(&self) -> Option<K> {
        self.prev
    }
    pub fn next(&self) -> Option<K> {
        self.next
    }
    pub fn children(&self) -> &NodeList<K> {
        &self.children
    }
}

#[derive(Debug, Clone)]
pub struct NodeList<K: Key> {
    pub head: Option<K>,
    pub tail: Option<K>,
    pub len: usize,
}

impl<K: Key> Default for NodeList<K> {
	fn default() -> Self{
		NodeList{
			head: None,
			tail: None,
			len: 0,
		}
	}
}
pub struct ChildrenMutIterator<'a, K: Key, T: Default> {
    inner: &'a mut SecondaryMap<K, Node<K, T>>,
    head: Option<K>,
}
impl<'a, K: Key, T: Default> Iterator for ChildrenMutIterator<'a, K, T> {
    type Item = (K, &'a mut Node<K, T>);

    fn next(&mut self) -> Option<Self::Item> {
		let head = match self.head {
			None => return None,
			Some(r) => r,
		};

        let inner = unsafe { &mut *(self.inner as *mut SecondaryMap<K, Node<K, T>>) };
        let n = unsafe { inner.get_unchecked_mut(head) };
        let next = n.next;
        let r = Some((head, n));
        self.head = next;
        r
    }
}
pub struct ChildrenIterator<'a, K: Key, T: Default> {
    inner: &'a SecondaryMap<K, Node<K, T>>,
    head: Option<K>,
}

impl<'a, K: Key, T: Default> Iterator for ChildrenIterator<'a, K, T> {
    type Item = (K, &'a Node<K, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let head = match self.head {
			None => return None,
			Some(r) => r,
		};
        let n = unsafe { self.inner.get_unchecked(head) };
        let r = Some((head, n));
        self.head = n.next;
        r
    }
}

pub struct RecursiveIterator<'a, K: Key, T: Default> {
    inner: &'a SecondaryMap<K, Node<K, T>>,
    arr: [K; 32],
    len: usize,
}

impl<'a, K: Key, T: Default> Iterator for RecursiveIterator<'a, K, T> {
    type Item = (K, &'a Node<K, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let head = self.arr[self.len];
        let n = unsafe { self.inner.get_unchecked(head) };
        let r = Some((head, n));
        if let Some(next) = n.next {
            self.arr[self.len] = next;
            self.len += 1;
        }
        if let Some(head) = n.children.head {
            self.arr[self.len] = head;
            self.len += 1;
        }
        r
    }
}

#[test]
fn test11() {
    //let n: Option<> = None;
    let mut tree: Tree<()> = Tree::default();
    tree.create(1);
    tree.create(11);
    tree.create(12);
    tree.create(111);
    tree.create(112);
    tree.create(121);
    tree.create(122);
    tree.create(123);
    tree.create(124);
    tree.insert_child(11, 1, 10);
    tree.insert_child(12, 1, 10);
    tree.insert_child(111, 11, 0);
    tree.insert_child(112, 11, 1);
    tree.insert_child(122, 12, 1);
    tree.insert_brother(121, 122, InsertType::Front);
    tree.insert_brother(123, 122, InsertType::Back);
    tree.insert_child(124, 12, 8);
    tree.insert_child(1, 0, 0);
    test_println(&tree);
    let info = tree.get_info(12).unwrap();
    println!("info--------------------------------");
    tree.destroy(12, info, true);
    test_println(&tree);
    for (i, _) in tree.iter(unsafe { tree.get_unchecked(1) }.children.head) {
        println!("i: {}", i);
    }
}
#[cfg(test)]
fn test_println(tree: &Tree<()>) {
    println!("test_println --------------------------------");
    for i in 1..200 {
        match tree.get(i) {
            Some(n) => {
                println!("id: {}, {:?}", i, n);
            }
            _ => (),
        }
    }
    println!("test_println over--------------------------------");
}

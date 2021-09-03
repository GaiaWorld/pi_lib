extern crate map_new;

use std::ops::{Index, IndexMut};

use map_new::vecmap::VecMap;

pub enum InsertType {
    Back,
    Front,
}

#[derive(Default)]
pub struct IdTree<T: Default> {
    map: VecMap<Node<T>>,
    statistics_count: bool,
}

impl<T: Default> Index<usize> for IdTree<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Node<T> {
        &self.map[index]
    }
}

impl<T: Default> IndexMut<usize> for IdTree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Node<T> {
        &mut self.map[index]
    }
}

impl<T: Default> IdTree<T> {
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			map: VecMap::with_capacity(capacity),
    		statistics_count: false,
		}
	}

    pub fn is_statistics_count(&self) -> bool {
        self.statistics_count
    }
    pub fn set_statistics_count(&mut self, b: bool) {
        self.statistics_count = b
    }
    pub fn get(&self, id: usize) -> Option<&Node<T>> {
        self.map.get(id)
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Node<T>> {
        self.map.get_mut(id)
    }
    pub unsafe fn get_unchecked(&self, id: usize) -> &Node<T> {
        self.map.get_unchecked(id)
    }
    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut Node<T> {
        self.map.get_unchecked_mut(id)
    }
    pub fn create(&mut self, id: usize, version: T) {
		let mut node = Node::default();
		node.data = version;
        self.map.insert(id, node);
    }
    /// index为0表示插入到子节点队列前， 如果index大于子节点队列长度，则插入到子节点队列最后。parent如果为0 表示设置为根节点。 如果parent的layer大于0
    pub fn insert_child(&mut self, id: usize, parent: usize, mut index: usize) -> usize {
        if parent < usize::max_value() {
            let (layer, prev, next) = match self.map.get(parent) {
                Some(n) if index >= n.children.len => (
                    if n.layer > 0 { n.layer + 1 } else { 0 },
                    n.children.tail,
                    usize::max_value(),
                ),
                Some(n) if index + index >= n.children.len => {
                    let mut prev = n.children.tail;
                    let mut next = usize::max_value();
                    index = n.children.len - index;
                    while index < usize::max_value() && prev < usize::max_value() {
                        index -= 1;
                        next = prev;
                        let node = unsafe { self.map.get_unchecked(next) };
                        prev = node.prev;
                    }
                    (if n.layer > 0 { n.layer + 1 } else { 0 }, prev, next)
                }
                Some(n) => {
                    let mut prev = usize::max_value();
                    let mut next = n.children.head;
                    while index < usize::max_value() && next < usize::max_value() {
                        index -= 1;
                        prev = next;
                        let node = unsafe { self.map.get_unchecked(next) };
                        next = node.next;
                    }
                    (if n.layer > 0 { n.layer + 1 } else { 0 }, prev, next)
                }
                _ => panic!("invalid parent: {}", parent),
            };
            self.insert_node(id, parent, layer, prev, next)
        } else {
            self.insert_root(id)
        }
    }
    /// 根据InsertType插入到brother的前或后。 brother的layer大于0
    pub fn insert_brother(&mut self, id: usize, brother: usize, insert: InsertType) -> usize {
        let (parent, layer, prev, next) = match self.map.get(brother) {
            Some(n) => match insert {
                InsertType::Front => (n.parent, n.layer, n.prev, brother),
                InsertType::Back => (n.parent, n.layer, brother, n.next),
            },
            _ => panic!("invalid brother: {}", brother),
        };
        if parent < usize::max_value() {
            self.insert_node(id, parent, layer, prev, next)
        } else {
            self.insert_root(id)
        }
    }
    /// 获得节点信息， 一般用于remove和destroy
    pub fn get_info(&mut self, id: usize) -> Option<(usize, usize, usize, usize, usize, usize)> {
        match self.map.get(id) {
            Some(n) => Some((n.parent, n.layer, n.count, n.prev, n.next, n.children.head)),
            _ => return None,
        }
    }
    /// 如果的节点的layer大于0，表示在树上
    pub fn remove(
        &mut self,
        id: usize,
        (parent, layer, count, prev, next, head): (usize, usize, usize, usize, usize, usize),
    ) {
        if layer > 0 {
            self.remove_tree(head);
        }
        if parent < usize::max_value() {
            self.remove_node(parent, count + 1, prev, next)
        }
        let node = unsafe { self.map.get_unchecked_mut(id) };
        node.parent = usize::max_value();
        node.layer = 0;
        node.prev = usize::max_value();
        node.next = usize::max_value();
    }
    /// 销毁子节点， recursive表示是否递归销毁
    pub fn destroy(
        &mut self,
        id: usize,
        (parent, layer, count, prev, next, mut head): (usize, usize, usize, usize, usize, usize),
        recursive: bool,
    ) {
        if recursive {
            self.recursive_destroy(id, head);
        } else {
            self.map.remove(id);
            if layer > 0 {
                while head < usize::max_value() {
                    let child = {
                        let n = unsafe { self.map.get_unchecked_mut(head) };
                        n.parent= usize::max_value();
                        n.layer = 0;
                        head = n.next;
                        n.prev = usize::max_value();
                        n.next = usize::max_value();
                        n.children.head
                    };
                    self.remove_tree(child);
                }
            } else {
                while head < usize::max_value() {
                    let n = unsafe { self.map.get_unchecked_mut(head) };
                    n.parent= usize::max_value();
                    head = n.next;
                    n.prev = usize::max_value();
                    n.next = usize::max_value();
                }
            }
        }
        if parent < usize::max_value() {
            self.remove_node(parent, count + 1, prev, next)
        }
    }
    /// 迭代指定节点的所有子元素
    pub fn iter_mut(&mut self, node_children_head: usize) -> ChildrenMutIterator<T> {
        ChildrenMutIterator {
            inner: &mut self.map,
            head: node_children_head,
        }
    }
    /// 迭代指定节点的所有子元素
    pub fn iter(&self, node_children_head: usize) -> ChildrenIterator<T> {
        ChildrenIterator {
            inner: &self.map,
            head: node_children_head,
        }
    }
    /// 迭代指定节点的所有递归子元素
    pub fn recursive_iter(&self, node_children_head: usize) -> RecursiveIterator<T> {
        RecursiveIterator {
            inner: &self.map,
            arr: [
                node_children_head,
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
                usize::max_value(),
            ],
            len: if node_children_head == 0 { 0 } else { 1 },
        }
    }
    fn insert_root(&mut self, id: usize) -> usize {
		log::info!("zzzzzzzzzzzzzzzzzzzzzzzzzzzz");
        // 设置为根节点
        let head = match self.map.get_mut(id) {
            Some(n) => {
				log::info!("n.parent: {}, layer:{}", n.parent, n.layer);
                if n.parent < usize::max_value() {
                    panic!("has a parent node, id: {}", id)
                }
                if n.layer > 0 {
                    panic!("already on the tree, id: {}", id)
                }
                n.layer = 1;
                n.children.head
            }
            _ => {
				log::info!("xxxxxxxxxxxxxxxxxxx");
				panic!("invalid id: {}", id);
			}
        };
		log::info!("bbbbbbbbbbbbbbbbbbbb");
        self.insert_tree(head, 2);
        1
    }
    // 插入节点, 如果id就在parent内则为调整位置
    fn insert_node(
        &mut self,
        id: usize,
        parent: usize,
        layer: usize,
        prev: usize,
        next: usize,
    ) -> usize {
		// 调用该方法，该节点可能已经存在，并且是将该节点插入到原位置
		// 如果插入到原位置，则无需操作
		if id == prev || id == next {
			return layer;
		}
        let (count, fix_prev, fix_next) = match self.map.get_mut(id) {
            Some(n) => {
                if n.parent != parent {
                    if n.parent < usize::max_value() {
                        panic!("has a parent node, id: {}", id)
                    }
                    if n.layer > 0 {
                        panic!("already on the tree, id: {}", id)
                    }
                    n.parent = parent;
                    n.layer = layer;
                    n.prev = prev;
					n.next = next;
                    (n.count + 1, n.children.head, usize::max_value())
                } else {
                    // 调整
                    let fix_prev = n.prev;
                    let fix_next = n.next;
                    n.prev = prev;
					n.next = next;
                    (0, fix_prev, fix_next)
                }
            }
            _ => panic!("invalid id: {}", id),
		};
        // 修改prev和next的节点
        if prev < usize::max_value() {
            let node = unsafe { self.map.get_unchecked_mut(prev) };
            node.next = id;
        }
        if next < usize::max_value() {
            let node = unsafe { self.map.get_unchecked_mut(next) };
            node.prev = id;
        }
        if count == 0 {
            // 同层调整
            if fix_prev < usize::max_value() {
                let node = unsafe { self.map.get_unchecked_mut(fix_prev) };
                node.next = fix_next;
            }
            if fix_next < usize::max_value() {
                let node = unsafe { self.map.get_unchecked_mut(fix_next) };
                node.prev = fix_prev;
            }
            if prev == usize::max_value() || next == usize::max_value() || fix_prev == usize::max_value() || fix_next == usize::max_value() {
                let node = unsafe { self.map.get_unchecked_mut(parent) };
                if prev == usize::max_value() {
                    node.children.head = id;
                } else if fix_prev == usize::max_value() {
                    node.children.head = fix_next;
                }
                if next == usize::max_value() {
                    node.children.tail = id;
                } else if fix_next == usize::max_value() {
                    node.children.tail = fix_prev;
                }
            }
            return layer;
        }
        let p = {
            // 修改parent的children, count
			let node = unsafe { self.map.get_unchecked_mut(parent) };
            if prev == usize::max_value() {
                node.children.head = id;
            }
            if next == usize::max_value() {
                node.children.tail = id;
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
    fn insert_tree(&mut self, mut id: usize, layer: usize) {
        while id < usize::max_value() {
            let head = {
                let n = unsafe { self.map.get_unchecked_mut(id) };
                n.layer = layer;
                id = n.next;
                n.children.head
            };
            self.insert_tree(head, layer + 1);
        }
    }
    // 从树上移除， 就是递归设置每个子节点的layer为0
    fn remove_tree(&mut self, mut id: usize) {
        while id < usize::max_value() {
            let head = {
                let n = unsafe { self.map.get_unchecked_mut(id) };
                n.layer = 0;
                id = n.next;
                n.children.head
            };
            self.remove_tree(head);
        }
    }
    // 递归销毁
    fn recursive_destroy(&mut self, parent: usize, mut id: usize) {
		self.map.remove(parent);
        while id < usize::max_value() {
            let (next, head) = {
                let n = unsafe { self.map.get_unchecked(id) };
                (n.next, n.children.head)
            };
            self.recursive_destroy(id, head);
            id = next;
        }
    }
    // 递归向上，修改节点的count
    fn modify_count(&mut self, mut id: usize, count: isize) {
        while id < usize::max_value() {
            let n = unsafe { self.map.get_unchecked_mut(id) };
            n.count = (n.count as isize + count) as usize;
            id = n.parent;
        }
    }
    // 移除节点
    fn remove_node(&mut self, parent: usize, count: usize, prev: usize, next: usize) {
        // 修改prev和next的节点
        if prev < usize::max_value() {
            let node = unsafe { self.map.get_unchecked_mut(prev) };
			node.next = next;
        }
        if next < usize::max_value() {
            let node = unsafe { self.map.get_unchecked_mut(next) };
			node.prev = prev;
        }
        
            // 修改parent的children, count
            let node = unsafe { self.map.get_unchecked_mut(parent) };
            if prev == usize::max_value() {
                node.children.head = next;
            }
            if next == usize::max_value() {
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
pub struct Node<T: Default> {
    parent: usize,      // 父节点
    layer: usize,       // 表示第几层，如果不在根上，则为0。 在根上，则起步为1
    count: usize,       // 所有的递归子节点的总数量
    prev: usize,        // 前ab节点
    next: usize,        // 后ab节点
    children: NodeList, // 子节点列表
    pub data: T,
}

impl<T: Default> Default for Node<T> {
	fn default() -> Self{
		Node {
			parent: usize::max_value(),
			layer: 0,
			count: 0,
			prev: usize::max_value(),
			next: usize::max_value(),
			children: NodeList::default(),
			data: T::default(),
		}
	}
}
impl<T: Default> Node<T> {
    pub fn parent(&self) -> usize {
        self.parent
    }
    pub fn layer(&self) -> usize {
        self.layer
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn prev(&self) -> usize {
        self.prev
    }
    pub fn next(&self) -> usize {
        self.next
    }
    pub fn children(&self) -> &NodeList {
        &self.children
    }
}

#[derive(Debug, Clone)]
pub struct NodeList {
    pub head: usize,
    pub tail: usize,
    pub len: usize,
}

impl Default for NodeList {
	fn default() -> Self{
		NodeList{
			head: usize::max_value(),
			tail: usize::max_value(),
			len: 0,
		}
	}
}
pub struct ChildrenMutIterator<'a, T: Default> {
    inner: &'a mut VecMap<Node<T>>,
    head: usize,
}
impl<'a, T: Default> Iterator for ChildrenMutIterator<'a, T> {
    type Item = (usize, &'a mut Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.head == usize::max_value() {
            return None;
        }
        let inner = unsafe { &mut *(self.inner as *mut VecMap<Node<T>>) };
        let n = unsafe { inner.get_unchecked_mut(self.head) };
        let next = n.next;
        let r = Some((self.head, n));
        self.head = next;
        r
    }
}
pub struct ChildrenIterator<'a, T: Default> {
    inner: &'a VecMap<Node<T>>,
    head: usize,
}

impl<'a, T: Default> Iterator for ChildrenIterator<'a, T> {
    type Item = (usize, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.head == usize::max_value() {
            return None;
        }
        let n = unsafe { self.inner.get_unchecked(self.head) };
        let r = Some((self.head, n));
        self.head = n.next;
        r
    }
}

pub struct RecursiveIterator<'a, T: Default> {
    inner: &'a VecMap<Node<T>>,
    arr: [usize; 32],
    len: usize,
}

impl<'a, T: Default> Iterator for RecursiveIterator<'a, T> {
    type Item = (usize, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let head = self.arr[self.len];
        let n = unsafe { self.inner.get_unchecked(head) };
        let r = Some((head, n));
        if n.next < usize::max_value() {
            self.arr[self.len] = n.next;
            self.len += 1;
        }
        if n.children.head < usize::max_value() {
            self.arr[self.len] = n.children.head;
            self.len += 1;
        }
        r
    }
}

#[test]
fn test11() {
    //let n: Option<> = None;
    let mut tree: IdTree<()> = IdTree::default();
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
fn test_println(tree: &IdTree<()>) {
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

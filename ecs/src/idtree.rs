
use std::{
  mem::{replace},
  marker::PhantomData,
};

use map::{Map, vecmap::VecMap};
use pointer::cell::TrustCell;
use Share;


use system::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};
use component::SingleCase;

pub type CellIdTree<T> = TrustCell<IdTree<VecMap<Node<T>>>>;
impl<T> Notify for CellIdTree<T> {
    fn add_create(&self, listener: CreateFn) {
        self.borrow_mut().notify.create.push_back(listener)
    }
    fn add_delete(&self, listener: DeleteFn) {
        self.borrow_mut().notify.delete.push_back(listener)
    }
    fn add_modify(&self, listener: ModifyFn) {
        self.borrow_mut().notify.modify.push_back(listener)
    }
    fn create_event(&self, id: usize) {
        self.borrow().notify.create_event(id);
    }
    fn delete_event(&self, id: usize) {
        self.borrow().notify.delete_event(id);
    }
    fn modify_event(&self, id: usize, field: &'static str, index: usize) {
        self.borrow().notify.modify_event(id, field, index);
    }
    fn remove_create(&self, listener: &CreateFn) {
        self.borrow_mut().notify.create.delete(listener);
    }
    fn remove_delete(&self, listener: &DeleteFn) {
        self.borrow_mut().notify.delete.delete(listener);
    }
    fn remove_modify(&self, listener: &ModifyFn) {
        self.borrow_mut().notify.modify.delete(listener);
    }
}
impl<T: Share> SingleCase for CellIdTree<T> {
}

pub enum InsertType{
    Back,
    Front,
}

#[derive(Default)]
pub struct IdTree<M> {
  map: M,
  notify: NotifyImpl,
}

impl<T, M> IdTree<M> where M:Map<Key=usize, Val=Node<T>> {
    pub fn new(m: M) -> Self {
        IdTree{
          map: m,
          notify: NotifyImpl::default(),
        }
    }
    pub fn get(&self, id: usize) -> Option<&Node<T>> {
      self.map.get(&id)
    }
    pub unsafe fn get_unchecked(&self, id: usize) -> &Node<T> {
      self.map.get_unchecked(&id)
    }
    pub unsafe fn set_bind(&mut self, id: usize, bind: T) -> Option<T> {
      match self.map.get_mut(&id) {
        Some(n) => Some(replace(&mut n.bind, bind)),
        _ => None
      }
    }
    pub fn create(&mut self, id: usize, bind: T) {
      self.map.insert(id, Node::new(bind));
    }
    /// index为0表示插入到子节点队列前， 如果index大于子节点队列长度，则插入到子节点队列最后。parent如果为0 表示设置为根节点。 如果parent的layer大于0，表示在树上，则会发出创建事件
    pub fn insert_child(&mut self, id: usize, parent: usize, mut index: usize) {
      if parent > 0 {
          let (layer, prev, next) = match self.map.get(&parent) {
            Some(n) => {
              let mut prev = 0;
              let mut next = n.children.head;
              while index > 0 && next > 0 {
                index -= 1;
                prev = next;
                let node = unsafe { self.map.get_unchecked(&next) };
                next = node.next;
              }
              (if n.layer > 0 {n.layer + 1}else{0}, prev, next)
          },
          _ => panic!("invalid parent: {}", parent)
        };
        self.insert_node(id, parent, layer, prev, next)
      }else{
        // 设置为根节点
        let head = match self.map.get_mut(&id) {
          Some(n) =>{
            if n.parent > 0 {
              panic!("has a parent node, id: {}", id)
            }
            if n.layer > 0 {
              panic!("already on the tree, id: {}", id)
            }
            n.layer = 1;
            n.children.head
          },
          _ => panic!("invalid id: {}", id)
        };
        self.insert_tree(head, 2);
        self.notify.create_event(parent);
      }
    }
    /// 根据InsertType插入到brother的前或后。 brother的layer大于0，表示在树上，则会发出创建事件
    pub fn insert_brother(&mut self, id: usize, brother: usize, insert: InsertType) {
        let (parent, layer, prev, next) = match self.map.get(&brother) {
          Some(n) => match insert {
            InsertType::Front => (n.parent, n.layer, n.prev, brother),
            InsertType::Back => (n.parent, n.layer, brother, n.next)
        },
        _ => panic!("invalid brother: {}", brother)
      };
      self.insert_node(id, parent, layer, prev, next)
    }
    /// 如果的节点的layer大于0，表示在树上，则会发出移除事件
    pub fn remove(&mut self, id: usize) {
      let (parent, layer, count, prev, next, head) = match self.map.get(&id) {
        Some(n) => {
          if n.parent == 0 && n.layer == 0 {
            return
          }
          (n.parent, n.layer, n.count, n.prev, n.next, n.children.head)
        },
        _ => panic!("invalid id: {}", id)
      };
      if layer > 0 {
        self.notify.delete_event(id);
        self.remove_tree(head);
      }
      if parent > 0 {
        self.remove_node(parent, count + 1, prev, next)
      }
      let node = unsafe { self.map.get_unchecked_mut(&id) };
      node.parent = 0;
      node.layer = 0;
      node.prev = 0;
      node.next = 0;
    }
    /// 销毁子节点， recursive表示是否递归销毁
    pub fn destroy(&mut self, id: usize, recursive: bool) {
      let (parent, layer, count, prev, next, mut head) = match self.map.get(&id) {
        Some(n) => {
          if n.parent == 0 && n.layer == 0 {
            return
          }
          (n.parent, n.layer, n.count, n.prev, n.next, n.children.head)
        },
        _ => panic!("invalid id: {}", id)
      };
      if layer > 0 {
        self.notify.delete_event(id);
        if recursive {
          self.recursive_destroy(id, head);
        }else {
          self.map.remove(&id);
          while head > 0 {
            let child = {
              let n = unsafe { self.map.get_unchecked_mut(&head) };
              n.parent = 0;
              n.layer = 0;
              head = n.next;
              n.prev = 0;
              n.next = 0;
              n.children.head
            };
            self.remove_tree(child);
          }
        }
      }else if recursive {
        self.recursive_destroy(id, head);
      }else{
        self.map.remove(&id);
        while head > 0 {
          let n = unsafe { self.map.get_unchecked_mut(&head) };
          n.parent = 0;
          head = n.next;
          n.prev = 0;
          n.next = 0;
        }
      }
      if parent > 0 {
        self.remove_node(parent, count + 1, prev, next)
      }
    }
    /// 迭代指定节点的所有递归子元素
    pub fn iter(&self, node_children_head: usize) -> ChildrenIterator<M, T> {
      ChildrenIterator{
        inner: &self.map,
        arr: [node_children_head, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
        len: if node_children_head == 0 {0}else{1},
        marker: PhantomData,
      }
    }
    // 插入节点
    fn insert_node(&mut self, id: usize, parent: usize, layer: usize, prev: usize, next: usize) {
      let (head, count) = match self.map.get_mut(&id) {
        Some(n) =>{
          if n.parent > 0 {
            panic!("has a parent node, id: {}", id)
          }
          if n.layer > 0 {
            panic!("already on the tree, id: {}", id)
          }
          n.parent = parent;
          n.layer = layer;
          n.prev = prev;
          n.next = next;
          (n.children.head, n.count + 1)
        },
        _ => panic!("invalid id: {}", id)
      };
      // 修改prev和next的节点
      if prev > 0 {
        let node = unsafe { self.map.get_unchecked_mut(&prev) };
        node.next = id;
      }
      if next > 0 {
        let node = unsafe { self.map.get_unchecked_mut(&next) };
        node.prev = id;
      }
      let p = {
        // 修改parent的children, count
        let node = unsafe { self.map.get_unchecked_mut(&parent) };
        if prev == 0 {
          node.children.head = id;
        }
        node.children.len += 1;
        node.count += count;
        node.parent
      };
      // 递归向上修改count
      self.modify_count(p, count as isize);
      if layer > 0 {
        self.insert_tree(head, layer + 1);
        self.notify.create_event(parent);
      }
    }
    // 插入到树上， 就是递归设置每个子节点的layer
    fn insert_tree(&mut self, mut id: usize, layer: usize) {
      while id > 0 {
        let head = {
          let n = unsafe { self.map.get_unchecked_mut(&id) };
          n.layer = layer;
          id = n.next;
          n.children.head
        };
        self.insert_tree(head, layer + 1);
      }
    }
    // 从树上移除， 就是递归设置每个子节点的layer为0
    fn remove_tree(&mut self, mut id: usize) {
      while id > 0 {
        let head = {
          let n = unsafe { self.map.get_unchecked_mut(&id) };
          n.layer = 0;
          id = n.next;
          n.children.head
        };
        self.remove_tree(head);
      }
    }
    // 递归销毁
    fn recursive_destroy(&mut self, parent: usize, mut id: usize) {
      self.map.remove(&parent);
      while id > 0 {
        let (next, head) = {
          let n = unsafe { self.map.get_unchecked(&id) };
          (n.next, n.children.head)
        };
        self.recursive_destroy(id, head);
        id = next;
      }
    }
    // 递归向上，修改节点的count
    fn modify_count(&mut self, mut id: usize, count: isize) {
      while id > 0 {
        let n = unsafe { self.map.get_unchecked_mut(&id) };
        n.count = (n.count as isize + count) as usize;
        id = n.parent;
      }
    }
    // 移除节点
    fn remove_node(&mut self, parent: usize, count: usize, prev: usize, next: usize) {
      // 修改prev和next的节点
      if prev > 0 {
        let node = unsafe { self.map.get_unchecked_mut(&prev) };
        node.next = next;
      }
      if next > 0 {
        let node = unsafe { self.map.get_unchecked_mut(&next) };
        node.prev = prev;
      }
      let p = {
        // 修改parent的children, count
        let node = unsafe { self.map.get_unchecked_mut(&parent) };
        if prev == 0 {
          node.children.head = next;
        }
        node.children.len -= 1;
        node.count -= count;
        node.parent
      };
      // 递归向上修改count
      self.modify_count(p, -(count as isize));
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node<T> {
  pub bind: T,             // 绑定
  pub parent: usize,       // 父节点
  pub layer: usize,        // 表示第几层，如果不在根上，则为0。 在根上，则起步为1
  pub count: usize,        // 所有的递归子节点的总数量
  pub prev: usize,         // 前ab节点
  pub next: usize,         // 后ab节点
  pub children: NodeList, // 子节点列表
}
impl<T> Node<T> {
  fn new(bind: T) -> Node<T> {
    Node {
      bind: bind,
      parent: 0,
      layer: 0,
      count: 0,
      prev: 0,
      next: 0,
      children: NodeList::default(),
    }
  }
}
#[derive(Debug, Clone, Default)]
pub struct NodeList {
  pub head: usize,
  pub len: usize,
}

pub struct ChildrenIterator<'a, M, T> {
    inner: &'a M,
    arr: [usize; 32],
    len: usize,
    marker: PhantomData<T>,
}

impl<'a, M, T> Iterator for ChildrenIterator<'a, M, T> where M:Map<Key=usize, Val=Node<T>> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
      if self.len == 0 {
        return None
      }
      self.len -= 1;
      let node = self.arr[self.len];
      let r = Some(node);
      let n = unsafe {self.inner.get_unchecked(&node)};
      if n.next > 0 {
        self.arr[self.len] = n.next;
        self.len += 1;
      }
      if n.children.head > 0 {
        self.arr[self.len] = n.children.head;
        self.len += 1;
      }
      r
    }
}



#[test]
fn test11(){
    let mut tree: IdTree<VecMap<Node<usize>>> =IdTree::new(VecMap::default());
    tree.create(1, 1);
    tree.create(11, 2);
    tree.create(12, 3);
    tree.create(111, 4);
    tree.create(112, 5);
    tree.create(121, 3);
    tree.create(122, 3);
    tree.create(123, 3);
    tree.create(124, 3);
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
    tree.destroy(12, true);
    test_println(&tree);
    for i in tree.iter(unsafe{tree.get_unchecked(1)}.children.head) {
      println!("i: {}", i);
    }
}
#[cfg(test)]
fn test_println(tree: &IdTree<VecMap<Node<usize>>>){
  println!("--------------------------------");
  for i in 1..200{
      match tree.get(i) {
        Some(n) => {
          println!("id: {}, {:?}", i, n);
        },
        _ => ()
      }
  }
}

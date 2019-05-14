
use std::{
  mem::{replace},
};

use map::{vecmap::VecMap};
use monitor::{NotifyImpl};

pub enum InsertType{
    Back,
    Front,
}

#[derive(Default)]
pub struct IdTree {
  map: VecMap<Node>,
}

impl IdTree {
    pub fn get(&self, id: usize) -> Option<&Node> {
      self.map.get(id)
    }
    pub unsafe fn get_unchecked(&self, id: usize) -> &Node {
      self.map.get_unchecked(id)
    }
    pub fn create(&mut self, id: usize) {
      self.map.insert(id, Node::default());
    }
    /// index为0表示插入到子节点队列前， 如果index大于子节点队列长度，则插入到子节点队列最后。parent如果为0 表示设置为根节点。 如果parent的layer大于0，表示在树上，则会发出创建事件
    pub fn insert_child(&mut self, id: usize, parent: usize, mut index: usize, notify: Option<&NotifyImpl>) {
      if parent > 0 {
          let (layer, prev, next) = match self.map.get(parent) {
            Some(n) => {
              let mut prev = 0;
              let mut next = n.children.head;
              while index > 0 && next > 0 {
                index -= 1;
                prev = next;
                let node = unsafe { self.map.get_unchecked(next) };
                next = node.next;
              }
              (if n.layer > 0 {n.layer + 1}else{0}, prev, next)
          },
          _ => panic!("invalid parent: {}", parent)
        };
        self.insert_node(id, parent, layer, prev, next, notify)
      }else{
        // 设置为根节点
        let head = match self.map.get_mut(id) {
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
        match notify {
            Some(n) => n.create_event(id),
            _ => ()
        };
      }
    }
    /// 根据InsertType插入到brother的前或后。 brother的layer大于0，表示在树上，则会发出创建事件
    pub fn insert_brother(&mut self, id: usize, brother: usize, insert: InsertType, notify: Option<&NotifyImpl>) {
        let (parent, layer, prev, next) = match self.map.get(brother) {
          Some(n) => match insert {
            InsertType::Front => (n.parent, n.layer, n.prev, brother),
            InsertType::Back => (n.parent, n.layer, brother, n.next)
        },
        _ => panic!("invalid brother: {}", brother)
      };
      self.insert_node(id, parent, layer, prev, next, notify)
    }
    /// 如果的节点的layer大于0，表示在树上，则会发出移除事件
    pub fn remove(&mut self, id: usize, notify: Option<&NotifyImpl>) -> Option<usize> {
      let (parent, layer, count, prev, next, head) = match self.map.get(id) {
        Some(n) => {
          if n.parent == 0 && n.layer == 0 {
            return Some(n.layer)
          }
          (n.parent, n.layer, n.count, n.prev, n.next, n.children.head)
        },
        _ => return None
      };
      if layer > 0 {
        match notify {
            Some(n) => n.delete_event(id),
            _ => ()
        };
        self.remove_tree(head);
      }
      if parent > 0 {
        self.remove_node(parent, count + 1, prev, next)
      }
      let node = unsafe { self.map.get_unchecked_mut(id) };
      node.parent = 0;
      node.layer = 0;
      node.prev = 0;
      node.next = 0;
      Some(layer)
    }
    /// 销毁子节点， recursive表示是否递归销毁
    pub fn destroy(&mut self, id: usize, recursive: bool, notify: Option<&NotifyImpl>) {
      let (parent, layer, count, prev, next, mut head) = match self.map.get(id) {
        Some(n) => {
          if n.parent == 0 && n.layer == 0 {
            return
          }
          (n.parent, n.layer, n.count, n.prev, n.next, n.children.head)
        },
        _ => panic!("invalid id: {}", id)
      };
      if layer > 0 {
        match notify {
            Some(n) => n.delete_event(id),
            _ => ()
        };
        if recursive {
          self.recursive_destroy(id, head);
        }else {
          self.map.remove(id);
          while head > 0 {
            let child = {
              let n = unsafe { self.map.get_unchecked_mut(head) };
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
        self.map.remove(id);
        while head > 0 {
          let n = unsafe { self.map.get_unchecked_mut(head) };
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
    /// 迭代指定节点的所有子元素
    pub fn iter(&self, node_children_head: usize) -> ChildrenIterator {
      ChildrenIterator{
        inner: &self.map,
        head: node_children_head,
      }
    }
    /// 迭代指定节点的所有递归子元素
    pub fn recursive_iter(&self, node_children_head: usize) -> RecursiveIterator {
      RecursiveIterator{
        inner: &self.map,
        arr: [node_children_head, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
        len: if node_children_head == 0 {0}else{1},
      }
    }
    // 插入节点
    fn insert_node(&mut self, id: usize, parent: usize, layer: usize, prev: usize, next: usize, notify: Option<&NotifyImpl>) {
      let (head, count) = match self.map.get_mut(id) {
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
        let node = unsafe { self.map.get_unchecked_mut(prev) };
        node.next = id;
      }
      if next > 0 {
        let node = unsafe { self.map.get_unchecked_mut(next) };
        node.prev = id;
      }
      let p = {
        // 修改parent的children, count
        let node = unsafe { self.map.get_unchecked_mut(parent) };
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
        match notify {
            Some(n) => n.create_event(id),
            _ => ()
        };
      }
    }
    // 插入到树上， 就是递归设置每个子节点的layer
    fn insert_tree(&mut self, mut id: usize, layer: usize) {
      while id > 0 {
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
      while id > 0 {
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
      while id > 0 {
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
      while id > 0 {
        let n = unsafe { self.map.get_unchecked_mut(id) };
        n.count = (n.count as isize + count) as usize;
        id = n.parent;
      }
    }
    // 移除节点
    fn remove_node(&mut self, parent: usize, count: usize, prev: usize, next: usize) {
      // 修改prev和next的节点
      if prev > 0 {
        let node = unsafe { self.map.get_unchecked_mut(prev) };
        node.next = next;
      }
      if next > 0 {
        let node = unsafe { self.map.get_unchecked_mut(next) };
        node.prev = prev;
      }
      let p = {
        // 修改parent的children, count
        let node = unsafe { self.map.get_unchecked_mut(parent) };
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
pub struct Node {
  pub parent: usize,       // 父节点
  pub layer: usize,        // 表示第几层，如果不在根上，则为0。 在根上，则起步为1
  pub count: usize,        // 所有的递归子节点的总数量
  pub prev: usize,         // 前ab节点
  pub next: usize,         // 后ab节点
  pub children: NodeList, // 子节点列表
}

#[derive(Debug, Clone, Default)]
pub struct NodeList {
  pub head: usize,
  pub len: usize,
}


pub struct ChildrenIterator<'a> {
    inner: &'a VecMap<Node>,
    head: usize,
}

impl<'a> Iterator for ChildrenIterator<'a> {
    type Item = (usize, &'a Node);

    fn next(&mut self) -> Option<Self::Item> {
      if self.head == 0 {
        return None
      }
      let n = unsafe {self.inner.get_unchecked(self.head)};
      let r = Some((self.head, n));
      self.head = n.next;
      r
    }
}

pub struct RecursiveIterator<'a> {
    inner: &'a VecMap<Node>,
    arr: [usize; 32],
    len: usize,
}

impl<'a> Iterator for RecursiveIterator<'a> {
    type Item = (usize, &'a Node);

    fn next(&mut self) -> Option<Self::Item> {
      if self.len == 0 {
        return None
      }
      self.len -= 1;
      let head = self.arr[self.len];
      let n = unsafe {self.inner.get_unchecked(head)};
      let r = Some((head, n));
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
    let n = None;
    let mut tree: IdTree =IdTree::default();
    tree.create(1);
    tree.create(11);
    tree.create(12);
    tree.create(111);
    tree.create(112);
    tree.create(121);
    tree.create(122);
    tree.create(123);
    tree.create(124);
    tree.insert_child(11, 1, 10, n);
    tree.insert_child(12, 1, 10, n);
    tree.insert_child(111, 11, 0, n);
    tree.insert_child(112, 11, 1, n);
    tree.insert_child(122, 12, 1, n);
    tree.insert_brother(121, 122, InsertType::Front, n);
    tree.insert_brother(123, 122, InsertType::Back, n);
    tree.insert_child(124, 12, 8, n);
    tree.insert_child(1, 0, 0, n);
    test_println(&tree);
    tree.destroy(12, true, n);
    test_println(&tree);
    for (i, _) in tree.iter(unsafe{tree.get_unchecked(1)}.children.head) {
      println!("i: {}", i);
    }
}
#[cfg(test)]
fn test_println(tree: &IdTree){
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


use std::{
    any::TypeId,
    marker::PhantomData,
    intrinsics::type_name,
    ops::{Deref, DerefMut},
};

use map::{Map};
use system::{Notify, NotifyImpl, CreateFn, DeleteFn, ModifyFn};

#[derive(Default)]
pub struct IdTree<M> {
  map: M,
  nofity: NotifyImpl,
}
// impl<M> GetNotify for IdTree<M> {
//     fn get_notify(&self) -> &mut Notify {
//         &mut self.nofity
//     }
// }

impl<T, M> IdTree<M> where M:Map<Key=usize, Val=Node<T>> {
    pub fn new(m: M) -> Self {
        IdTree{
          map: m,
          nofity: NotifyImpl::default(),
        }
    }
    #[inline]
    fn remove_node(&mut self, list: &mut NodeList, prev: usize, next: usize) {
      if prev > 0 {
        let node = unsafe { self.map.get_unchecked_mut(&prev) };
        node.next = next;
      } else {
        list.head = next;
      }
      if next > 0 {
        let node = unsafe { self.map.get_unchecked_mut(&next) };
        node.prev = prev;
      }
      list.len -= 1;
    }
}

#[derive(Debug, Clone)]
pub struct Node<T> {
  bind: T,             // 绑定
  root: usize,          // 根节点Id，如果不在根上，则为0。 如果本节点就是根节点，则root就是自身的id
  parent: usize,       // 父节点
  layer: usize,        // 表示第几层
  count: usize,        // 所有的递归子节点的总数量
  prev: usize,         // 前ab节点
  next: usize,         // 后ab节点
  children: NodeList, // 子节点列表
}
impl<T> Node<T> {
  pub fn new(bind: T, parent: usize, layer: usize) -> Node<T> {
    Node {
      bind: bind,
      root: 0,
      parent: parent,
      layer: layer,
      count: 0,
      prev: 0,
      next: 0,
      children: NodeList::default(),
    }
  }
}

#[derive(Debug, Clone, Default)]
struct NodeList {
  head: usize,
  len: usize,
}
impl NodeList {
  #[inline]
  fn push(&mut self, id: usize) {
    self.head = id;
    self.len += 1;
  }
}

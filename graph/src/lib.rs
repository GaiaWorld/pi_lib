//! 静态有向无环图
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;

// pub mod zindex;

use core::hash::Hash;
use std::mem::replace;
use std::ptr::eq;

use hash::{XHashMap, XHashSet};
use share::{ShareUsize};
use std::fmt::Debug;
use std::{slice::Iter, sync::atomic::Ordering};

pub trait DirectedGraph<K: Hash + Eq + Sized, T> {
    type Node: DirectedGraphNode<K, T>;
    /// 迭代器的关联类型，指定了迭代器`Item`为`K`
    /// type NodeIter: Iterator<Item = &'a K>;
    fn get(&self, key: &K) -> Option<&Self::Node>;
    fn get_mut(&mut self, key: &K) -> Option<&mut Self::Node>;
    fn node_count(&self) -> usize;
    /// 获取from节点的数量
    fn from_len(&self) -> usize;
    /// 获取to节点的数量
    fn to_len(&self) -> usize;
    fn from(&self) -> &[K];
    fn to(&self) -> &[K];
    // /// 全部节点的迭代器
    // fn nodes('a self) -> Self::NodeIter;
    // /// 获取图的froms节点的迭代器
    // fn from(&'a self) -> Self::NodeIter;
    // /// 获取图的froms节点的迭代器
    // fn to(&'a self) -> Self::NodeIter;
    // /// 从from节点开始的深度遍历迭代器
    // fn from_dfs('a self) -> Self::NodeIter;
    // /// 从from节点开始的深度遍历迭代器
    // fn to_dfs('a self) -> Self::NodeIter;
    /// 拓扑排序
    fn topological_sort(&self) -> &[K];
}
pub trait DirectedGraphNode<K: Hash + Eq + Sized, T> {
    /// 迭代器的关联类型，指定了迭代器`Item`为`K`
    //type NodeIter: Iterator<Item = &'a K>;

    /// 获取from节点的数量
    fn from_len(&self) -> usize;
    /// 获取to节点的数量
    fn to_len(&self) -> usize;
    // /// 获取from节点的迭代器
    // fn from(&self) -> Self::NodeIter;
    // /// 获取to节点的迭代器
    // fn to(&self) -> Self::NodeIter;
    fn from(&self) -> &[K];
    fn to(&self) -> &[K];
    /// 获取键的引用
    fn key(&self) -> &K;
    /// 获取值的引用
    fn value(&self) -> &T;
    /// 获取值的可变引用
    fn value_mut(&mut self) -> &mut T;
    /// 读取计数器
    fn load_count(&self) -> usize;
    /// 增加计数器的值
    fn add_count(&self, add: usize) -> usize;
    /// 设置计数器的值
    fn set_count(&self, count: usize);
}
// 遍历邻居的迭代器 TODO 不好实现
pub struct NodeIterator<'a, K>(Iter<'a, K>);

impl<'a, K> Iterator for NodeIterator<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// 具体实现的图
#[derive(Default, Debug)]
pub struct NGraph<K: Hash + Eq + Sized + Debug, T> {
    map: XHashMap<K, NGraphNode<K, T>>,
    from: Vec<K>,
    to: Vec<K>,
    topological: Vec<K>,
}
#[derive(Debug)]
pub struct NGraphNode<K: Hash + Eq + Sized + Debug, T> {
    from: Vec<K>,
    to: Vec<K>,
    key: K,
    value: T,
    count: ShareUsize,
}

impl<K: Hash + Eq + Sized + Debug, T> DirectedGraphNode<K, T> for NGraphNode<K, T> {
    fn from_len(&self) -> usize {
        self.from.len()
    }

    fn to_len(&self) -> usize {
        self.to.len()
    }

    fn from(&self) -> &[K] {
        &self.from[..]
    }

    fn to(&self) -> &[K] {
        &self.to[..]
    }

    fn key(&self) -> &K {
        &self.key
    }

    fn value(&self) -> &T {
        &self.value
    }

    fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    fn load_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    fn add_count(&self, add: usize) -> usize {
        self.count.fetch_add(add, Ordering::SeqCst)
    }

    fn set_count(&self, count: usize) {
        self.count.store(count, Ordering::SeqCst)
    }
}


impl<K: Hash + Eq + Sized + Debug, T> NGraph<K, T> {
    pub fn reset(&self) {
        for n in self.map.values() {
            n.set_count(0)
        }
    }
}
impl<K: Hash + Eq + Sized + Debug, T> DirectedGraph<K, T> for NGraph<K, T> {
    // /// 迭代器的关联类型，指定了迭代器`Item`为`K`
    // type NodeIter = NodeIterator<'a, K>;
    type Node = NGraphNode<K, T>;

    fn get(&self, key: &K) -> Option<&Self::Node> {
        self.map.get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut Self::Node> {
        self.map.get_mut(key)
    }

    fn node_count(&self) -> usize {
        self.map.len()
    }
    fn from_len(&self) -> usize {
        self.from.len()
    }

    fn to_len(&self) -> usize {
        self.to.len()
    }
    fn from(&self) -> &[K] {
        &self.from[..]
    }

    fn to(&self) -> &[K] {
        &self.to[..]
    }
    fn topological_sort(&self) -> &[K] {
        &self.topological[..]
    }
    // fn check_loop(&self) -> Option<&K> {
    //     let mut stack = Vec::new();
    //     let mut arr = (0, self.from());
    //     loop {
    //         while arr.0 < arr.1.len() {
    //             let k = &arr.1[arr.0];
    //             arr.0 += 1;
    //             let n = self.get(k).unwrap();
    //             if n.to_len() > 0 {
    //                 if n.from_len() < n.load_count() {
    //                     self.reset();
    //                     return Some(k);
    //                 }
    //                 // 进入次数加1
    //                 n.add_count(1);
    //                 // 将当前的节点切片放入栈
    //                 stack.push(arr);
    //                 // 切换成检查下一层的节点切片
    //                 arr = (0, n.to());
    //             }
    //         }
    //         match stack.pop() {
    //             Some(r) => arr = r,
    //             _ => {
    //                 self.reset();
    //                 return None;
    //             }
    //         }
    //     }
    // }
}
pub struct NGraphBuilder<K: Hash + Eq + Sized + Debug, T> {
    graph: NGraph<K, T>,
}

impl<K: Hash + Eq + Sized + Clone + Debug, T> NGraphBuilder<K, T> {
    pub fn new() -> Self {
        let graph: NGraph<K, T> = NGraph {
            map: Default::default(),
            from: Default::default(),
            to: Default::default(),
            topological: Default::default(),
        };
        NGraphBuilder { graph }
    }
    pub fn node(mut self, key: K, value: T) -> Self {
        self.graph.map.insert(
            key.clone(),
            NGraphNode {
                from: Default::default(),
                to: Default::default(),
                key,
                value,
                count: Default::default(),
            },
        );
        self
    }
    pub fn edge(mut self, from: K, to: K) -> Self {
        let node = self.graph.map.get_mut(&from).unwrap();
        node.to.push(to.clone());
        let node = self.graph.map.get_mut(&to).unwrap();
        node.from.push(from);
        self
    }
    /// 返回图，或者回环的节点
    pub fn build(mut self) -> Result<NGraph<K, T>, Vec<K>> {
        // 计算开头和结尾的节点
        for (k, v) in self.graph.map.iter() {
            if v.from.is_empty() {
                self.graph.from.push(k.clone());
            }
            if v.to.is_empty() {
                self.graph.to.push(k.clone());
            }
        }
        let mut set = XHashSet::default();
        let mut vec = Vec::new();
        for k in self.graph.from() {
            // 记录第一层节点
            vec.push(k.clone());
            // 初始的节点
            let node = self.graph.get(k).unwrap();
            // 遍历节点的后续节点
            for to in node.to() {
                set.insert(to.clone());
                let n = self.graph.get(to).unwrap();
                // 将节点的计数加1
                n.add_count(1);
                println!("add n: k: {:?}, from:{} count:{}", to, n.from_len(), n.load_count());
            }
        }
        let mut set1 = XHashSet::default();
        while set.len() > 0 {
            println!("set: {:?}", set);
            let mut cycle = true;
            // 遍历后续节点
            for k in set.iter() {
                let n = self.graph.get(k).unwrap();
                println!("set n: k: {:?}, from:{} count:{}", k, n.from_len(), n.load_count());
                // 如果节点的计数等于from_len，表示from都处理了，节点已经就绪
                if n.from_len() == n.load_count() {
                    // 将计数器归0
                    n.set_count(0);
                    // 记录已经就绪的节点
                    vec.push(k.clone());
                    // 遍历该节点的后续节点
                    for to in n.to() {
                        set1.insert(to.clone());
                        let n = self.graph.get(to).unwrap();
                        // 将节点的计数加1
                        n.add_count(1);
                    }
                    cycle = false;
                }else{
                    set1.insert(k.clone());
                }
            }
            if cycle {
                let mut vec = Vec::new();
                vec.extend(set1.into_iter());
                return Result::Err(vec)
            }
            set.clear();
            set1 = replace(&mut set, set1);
        }
        let _ = replace(&mut self.graph.topological, vec);
        Result::Ok(self.graph)
    }
}


#[test]
fn test_graph() {

    struct A (usize);


    let graph = NGraphBuilder::new()
    .node(1, 1)
    .node(2, 2)
    .node(3, 3)
    .node(4, 4)
    .node(5, 5)
    .node(6, 6)
    .node(7, 7)
    .node(8, 8)
    .node(9, 9)
    .node(10, 10)
    .node(11, 11)
    .edge(1, 4)
    .edge(2, 4)
    .edge(2, 5)
    .edge(3, 5)
    .edge(4, 6)
    .edge(4, 7)
    .edge(5, 8)
    .edge(9, 10)
    .edge(10, 11)
    .edge(11, 5)
    .edge(5, 10)
    .build();
    println!("{:?}", graph);
    //assert_eq!(graph.is_ok(), true)
}
//! 有向无环图

use core::hash::Hash;
use hash::{XHashMap, XHashSet};
use log::{debug, error};
use share::ShareUsize;
use std::{fmt::Debug, mem::replace, option::Iter, sync::atomic::Ordering, thread::current};

/// 有向无环图
/// K 节点的键
/// T 节点的值
pub trait DirectedGraph<K: Hash + Eq + Sized, T> {
    /// 节点
    type Node: DirectedGraphNode<K, T>;

    // /// 迭代器的关联类型，指定了迭代器`Item`为`K`
    // type NodeIter: Iterator<Item = &'a K>;

    /// 根据 key 取 节点
    fn get(&self, key: &K) -> Option<&Self::Node>;

    /// 根据 key 取 节点
    fn get_mut(&mut self, key: &K) -> Option<&mut Self::Node>;

    /// 取节点的数量
    fn node_count(&self) -> usize;

    /// 取 有输入节点 的 数量
    fn from_len(&self) -> usize;

    /// 取 有输出节点 的 数量
    fn to_len(&self) -> usize;

    /// 取 输入节点 切片
    fn from(&self) -> &[K];

    /// 取 输出节点 切片
    fn to(&self) -> &[K];

    /// 拓扑排序
    fn topological_sort(&self) -> &[K];

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
}

/// 有向无环图 节点
pub trait DirectedGraphNode<K: Hash + Eq + Sized, T> {
    // /// 迭代器的关联类型，指定了迭代器`Item`为`K`
    // type NodeIter: Iterator<Item = &'a K>;

    /// 取 from节点 的 数量
    fn from_len(&self) -> usize;

    /// 取 to节点 的 数量
    fn to_len(&self) -> usize;

    /// 取 入点 的 切片
    fn from(&self) -> &[K];

    /// 取 出点 的 切片
    fn to(&self) -> &[K];

    /// 取键的引用
    fn key(&self) -> &K;

    /// 取值的引用
    fn value(&self) -> &T;

    /// 取值的可变引用
    fn value_mut(&mut self) -> &mut T;

    /// 读取计数器
    fn load_count(&self) -> usize;

    /// 增加计数器的值
    fn add_count(&self, add: usize) -> usize;

    /// 设置计数器的值
    fn set_count(&self, count: usize);

    // /// 获取from节点的迭代器
    // fn from(&self) -> Self::NodeIter;

    // /// 获取to节点的迭代器
    // fn to(&self) -> Self::NodeIter;
}

// 遍历邻居的迭代器 TODO 不好实现
/// 节点 迭代器
pub struct NodeIterator<'a, K>(Iter<'a, K>);

impl<'a, K> Iterator for NodeIterator<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// 图
#[derive(Default, Debug)]
pub struct NGraph<K: Hash + Eq + Sized + Debug, T> {
    // 所有节点
    map: XHashMap<K, NGraphNode<K, T>>,

    // 入度为0 的 节点
    from: Vec<K>,

    // 出度为0 的 节点
    to: Vec<K>,

    // 拓扑排序后的结果
    topological: Vec<K>,
}

/// 图节点
#[derive(Debug)]
pub struct NGraphNode<K: Hash + Eq + Sized + Debug, T> {
    // 该节点的 入度节点
    from: Vec<K>,

    // 该节点的 出度节点
    to: Vec<K>,
    // 键
    key: K,
    // 值
    value: T,
    // 引用计数
    count: ShareUsize,
}

impl<K: Clone + Hash + Eq + Sized + Debug, T: Clone> Clone for NGraphNode<K, T> {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            key: self.key.clone(),
            value: self.value.clone(),
            count: ShareUsize::new(self.count.load(Ordering::Relaxed)),
        }
    }
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
    pub(crate) fn new() -> Self {
        Self {
            map: Default::default(),
            from: Default::default(),
            to: Default::default(),
            topological: Default::default(),
        }
    }

    /// 重置 图
    pub fn reset(&self) {
        for n in self.map.values() {
            n.set_count(0)
        }
    }
}

impl<K: Hash + Eq + Sized + Debug, T> NGraph<K, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.values().into_iter().map(|v| v.value())
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

impl<K: Clone + Hash + Eq + Sized + Debug, T: Clone> NGraph<K, T> {
    /// 遍历 局部图
    pub fn gen_graph_from_keys(&self, keys: &[K]) -> Self {
        let mut builder = NGraphBuilder::new();

        debug!("gen_graph_from_keys, param keys = {:?}", keys);

        let mut current_keys = vec![];
        for k in keys {
            current_keys.push(k.clone());

            let n = self.map.get(k).unwrap();

            // 防止 keys的 重复 元素
            if !builder.has_node(k) {
                debug!("gen_graph_from_keys, add node k = {:?}", k);
                builder = builder.node(k.clone(), n.value.clone());
            }
        }

        while !current_keys.is_empty() {
            debug!("gen_graph_from_keys, current_keys = {:?}", current_keys);

            let mut next_keys = vec![];

            for curr in current_keys.iter() {
                let curr_node = self.map.get(curr).unwrap();

                // 下一轮 出点
                for next in curr_node.to() {
                    let next_node = self.map.get(next).unwrap();

                    if !builder.has_node(next) {
                        debug!("gen_graph_from_keys, add node next = {:?}", next);
                        builder = builder.node(next.clone(), next_node.value.clone());
                    }

                    debug!("gen_graph_from_keys, add edge = ({:?}, {:?})", curr, next);
                    builder = builder.edge(curr.clone(), next.clone());
                    next_keys.push(next.clone());
                }
            }

            debug!("gen_graph_from_keys, next_keys = {:?}", next_keys);

            let _ = replace(&mut current_keys, next_keys);
        }

        builder.build().unwrap()
    }
}

/// 图 构建器
pub struct NGraphBuilder<K: Hash + Eq + Sized + Debug, T> {
    graph: NGraph<K, T>,
}

impl<K: Hash + Eq + Sized + Clone + Debug, T> NGraphBuilder<K, T> {
    /// 创建 默认
    pub fn new() -> Self {
        NGraphBuilder {
            graph: NGraph::new(),
        }
    }

    /// 用已有的图 重新 构建
    pub fn new_with_graph(mut graph: NGraph<K, T>) -> Self {
        graph.from.clear();
        graph.to.clear();
        graph.topological.clear();

        NGraphBuilder { graph }
    }

    /// 对应节点是否存在
    pub fn has_node(&self, key: &K) -> bool {
        self.graph.map.get(key).is_some()
    }

    /// 添加 节点
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

    /// 添加 边
    pub fn edge(mut self, from: K, to: K) -> Self {
        let node = self.graph.map.get_mut(&from).unwrap();
        node.to.push(to.clone());

        let node = self.graph.map.get_mut(&to).unwrap();
        node.from.push(from);

        self
    }

    /// 移除 节点
    pub fn remove_node(mut self, key: &K) -> Self {
        let from;
        let to;
        {
            let node = self.graph.get(&key).unwrap();
            from = node.from().to_vec();
            to = node.to().to_vec();
        }

        for f in from {
            self.remove_edge_impl(&f, key);
        }

        for t in to {
            self.remove_edge_impl(key, &t);
        }
        self.graph.map.remove(&key);

        self
    }

    /// 移除 边
    pub fn remove_edge(mut self, from: &K, to: &K) -> Self {
        self.remove_edge_impl(from, to);
        self
    }

    // 移除 边，实现
    fn remove_edge_impl(&mut self, from: &K, to: &K) {
        // 到 from 节点删掉 to
        let from_node = self.graph.map.get_mut(&from).unwrap();
        if let Some(index) = from_node.to.iter().position(|v| *v == *to) {
            from_node.to.swap_remove(index);
        }

        // 到 to 节点删掉 from
        let to_node = self.graph.map.get_mut(&to).unwrap();
        if let Some(index) = to_node.from.iter().position(|v| *v == *from) {
            to_node.from.swap_remove(index);
        }
    }

    /// 构建图
    /// 返回Graph，或者 回环的节点
    pub fn build(mut self) -> Result<NGraph<K, T>, Vec<K>> {
        // 计算开头 和 结尾的 节点
        for (k, v) in self.graph.map.iter() {
            // 开头：没有入边的点
            if v.from.is_empty() {
                self.graph.from.push(k.clone());
            }

            // 结尾：没有出边的点
            if v.to.is_empty() {
                self.graph.to.push(k.clone());
            }
        }

        // 已经处理过的节点Key
        let mut topos = Vec::new();
        // 即将处理的节点Key
        let mut handle_set = XHashSet::default();

        debug!("graph's from = {:?}", self.graph.from());

        for k in self.graph.from() {
            topos.push(k.clone());

            // 处理 from 的 下一层
            let node = self.graph.get(k).unwrap();
            // 遍历节点的后续节点
            for to in node.to() {
                handle_set.insert(to.clone());

                // 即将处理：将节点的计数加1
                let n = self.graph.get(to).unwrap();
                n.add_count(1);

                debug!(
                    "add n: k: {:?}, from:{} count:{}",
                    to,
                    n.from_len(),
                    n.load_count()
                );
            }
        }

        // 没有 入点，是 循环图
        if topos.is_empty() && !self.graph.map.is_empty() {
            let mut vec = vec![];
            vec.extend(self.graph.map.keys().cloned());

            error!("graph build error, no from node, cycle's node = {:?}", &vec);
            return Result::Err(vec);
        }

        // 下个循环 要处理的 节点Key
        let mut next_set = XHashSet::default();

        while handle_set.len() > 0 {
            debug!("begin set: {:?}", handle_set);

            // 只有当 这个循环 找不到任何可以处理的节点的时候，循环存在
            let mut cycle = true;

            // 遍历后续节点
            for k in handle_set.iter() {
                let n = self.graph.get(k).unwrap();
                debug!(
                    "set n: k: {:?}, from:{} count:{}",
                    k,
                    n.from_len(),
                    n.load_count()
                );

                // 如果节点的计数等于from_len，表示from都处理了，节点已经就绪
                if n.from_len() != n.load_count() {
                    next_set.insert(k.clone());
                } else {
                    // 只要这一轮 有一个节点 能被处理，就没有 循环
                    // 存在循环，当且仅当 这一轮没有任何节点得到处理
                    cycle = false;

                    // 如果 前面 已经放进去，下一轮就会出问题
                    // demo 请参考 test_complex
                    let _ = next_set.remove(k);

                    topos.push(k.clone());

                    // 将计数器归0
                    n.set_count(0);

                    // 遍历该节点的后续节点
                    for to in n.to() {
                        next_set.insert(to.clone());

                        // 将节点的计数加1
                        let n = self.graph.get(to).unwrap();
                        n.add_count(1);
                    }
                }
            }

            // 有 循环引用，不符合 有向无环图
            if cycle {
                let mut vec = Vec::new();
                vec.extend(next_set.into_iter());

                error!("graph build error, cycle ref, vec = {:?}", &vec);
                return Result::Err(vec);
            }

            // 清空 此次 处理的 节点
            handle_set.clear();

            // 将 下批 和 这批 进行交换
            next_set = replace(&mut handle_set, next_set);
        }

        let _ = replace(&mut self.graph.topological, topos);
        Result::Ok(self.graph)
    }
}

mod tests {
    use log::info;

    use crate::*;

    use std::sync::Once;

    static INIT: Once = Once::new();
    fn setup_logger() {
        use env_logger::{Builder, Env};

        INIT.call_once(|| {
            Builder::from_env(Env::default().default_filter_or("debug")).init();
        });
    }

    // 测试 无节点 的 图
    #[test]
    fn test_empty() {
        setup_logger();

        let graph = NGraphBuilder::<u32, u32>::new().build();

        assert_eq!(graph.is_ok(), true);

        let graph = graph.unwrap();
        assert_eq!(graph.node_count(), 0);

        assert_eq!(graph.from_len(), 0);
        assert_eq!(graph.from(), &[]);

        assert_eq!(graph.to_len(), 0);
        assert_eq!(graph.to(), &[]);

        assert_eq!(graph.topological_sort(), &[]);
    }

    // 测试 1个节点的 图
    #[test]
    fn test_one_node() {
        setup_logger();

        let graph = NGraphBuilder::new().node(10, 111).build();

        assert_eq!(graph.is_ok(), true);

        let graph = graph.unwrap();
        assert_eq!(graph.node_count(), 1);

        assert_eq!(graph.from_len(), 1);
        assert_eq!(graph.from(), &[10]);

        assert_eq!(graph.to_len(), 1);
        assert_eq!(graph.to(), &[10]);

        assert_eq!(graph.topological_sort(), &[10]);
    }

    // 测试 无边 的 图
    #[test]
    fn test_no_edge() {
        setup_logger();

        // 1 2 3
        let graph = NGraphBuilder::new()
            .node(1, 1)
            .node(2, 2)
            .node(3, 3)
            .build();

        assert_eq!(graph.is_ok(), true);

        let graph = graph.unwrap();
        assert_eq!(graph.node_count(), 3);

        assert_eq!(graph.from_len(), 3);
        assert_eq!(graph.from(), &[1, 2, 3]);

        assert_eq!(graph.to_len(), 3);
        assert_eq!(graph.to(), &[1, 2, 3]);

        assert_eq!(graph.topological_sort(), &[1, 2, 3]);
    }

    // 测试 简单的 图
    #[test]
    fn test_simple() {
        setup_logger();

        // 1 --> 2 --> 3
        let graph = NGraphBuilder::new()
            .node(1, 1)
            .node(2, 2)
            .node(3, 3)
            .edge(1, 2)
            .edge(2, 3)
            .build();

        assert_eq!(graph.is_ok(), true);

        let graph = graph.unwrap();
        assert_eq!(graph.node_count(), 3);

        assert_eq!(graph.from_len(), 1);
        assert_eq!(graph.from(), &[1]);

        assert_eq!(graph.to_len(), 1);
        assert_eq!(graph.to(), &[3]);

        assert_eq!(graph.topological_sort(), &[1, 2, 3]);
    }

    // 测试 循环图
    #[test]
    fn test_cycle_graph() {
        setup_logger();

        // 1 --> 2 --> 3 --> 1
        let graph = NGraphBuilder::new()
            .node(1, 1)
            .node(2, 2)
            .node(3, 3)
            .edge(1, 2)
            .edge(2, 3)
            .edge(3, 1)
            .build();

        assert_eq!(graph.is_err(), true);
        if let Err(r) = graph {
            assert_eq!(&r, &[1, 2, 3]);
        }
    }

    // 测试 局部 循环
    #[test]
    fn test_cycle_local() {
        setup_logger();

        // 1 --> 2 <--> 3
        let graph = NGraphBuilder::new()
            .node(1, 1)
            .node(2, 2)
            .node(3, 3)
            .edge(1, 2)
            .edge(2, 3)
            .edge(3, 2)
            .build();

        assert_eq!(graph.is_err(), true);
        if let Err(r) = graph {
            assert_eq!(&r, &[2]);
        }
    }

    // 生成局部图
    #[test]
    fn test_gen_graph() {
        setup_logger();

        // 7 --> 2, 6
        // 2, 3 --> 1
        // 5, 6 --> 4
        let graph = NGraphBuilder::new()
            .node(1, 1)
            .node(2, 2)
            .node(3, 3)
            .node(4, 4)
            .node(5, 5)
            .node(6, 6)
            .node(7, 7)
            .edge(7, 2)
            .edge(7, 6)
            .edge(2, 1)
            .edge(3, 1)
            .edge(5, 4)
            .edge(6, 4)
            .build();

        assert_eq!(graph.is_ok(), true);

        let graph = graph.unwrap();
        let g2 = graph.gen_graph_from_keys(&[7]);
        assert_eq!(g2.node_count(), 5);
        info!("g2 = {:?}", g2);
    }

    // 复杂
    #[test]
    fn test_complex() {
        setup_logger();

        // 1 --> 3
        // 2 --> 3, 4, 5
        // 4 --> 5
        let graph = NGraphBuilder::new()
            .node(1, 1)
            .node(2, 2)
            .node(3, 3)
            .node(4, 4)
            .node(5, 5)
            .edge(1, 3)
            .edge(2, 3)
            .edge(2, 4)
            .edge(2, 5)
            .edge(4, 5)
            .build();

        assert_eq!(graph.is_ok(), true);

        let graph = graph.unwrap();
        assert_eq!(graph.node_count(), 5);

        assert_eq!(graph.from_len(), 2);

        let mut v: Vec<i32> = graph.from().iter().cloned().collect();
        v.sort();
        assert_eq!(&v, &[1, 2]);

        assert_eq!(graph.to_len(), 2);
        let mut v: Vec<i32> = graph.to().iter().cloned().collect();
        v.sort();
        assert_eq!(&v, &[3, 5]);

        assert_eq!(graph.topological_sort(), &[2, 1, 4, 5, 3]);
    }

    #[test]
    fn test_graph() {
        setup_logger();

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

        assert_eq!(graph.is_err(), true);
        if let Err(v) = graph {
            assert_eq!(&v, &[5, 10]);
        }
    }
}

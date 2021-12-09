//! 静态有向无环图
#![feature(associated_type_bounds)]
#![feature(test)]
extern crate test;

use core::hash::Hash;

use flume::{bounded, Receiver, Sender};

use hash::XHashMap;

use r#async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use share::ShareUsize;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use std::sync::Arc;
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
    // /// 拓扑排序的迭代器
    // fn topological_sort('a self) -> Self::NodeIter;
    // /// 检查是否有依赖回环 TODO 以后移除出去
    fn check_loop(&self) -> Option<&K>;
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

#[derive(Default, Debug)]
pub struct NGraph<K: Hash + Eq + Sized + Debug, T> {
    map: XHashMap<K, NGraphNode<K, T>>,
    from: Vec<K>,
    to: Vec<K>,
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
// 遍历邻居的迭代器
pub struct NodeIterator<'a, K>(Iter<'a, K>);

impl<'a, K> Iterator for NodeIterator<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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

    fn check_loop(&self) -> Option<&K> {
        let mut stack = Vec::new();
        let mut arr = (0, self.from());
        loop {
            while arr.0 < arr.1.len() {
                let k = &arr.1[arr.0];
                arr.0 += 1;
                let n = self.get(k).unwrap();
                if n.to_len() > 0 {
                    if n.from_len() < n.load_count() {
                        self.reset();
                        return Some(k);
                    }
                    // 进入次数加1
                    n.add_count(1);
                    // 将当前的节点切片放入栈
                    stack.push(arr);
                    // 切换成检查下一层的节点切片
                    arr = (0, n.to());
                }
            }
            match stack.pop() {
                Some(r) => arr = r,
                _ => {
                    self.reset();
                    return None;
                }
            }
        }
    }
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
    pub fn build(mut self) -> NGraph<K, T> {
        for (k, v) in self.graph.map.iter() {
            if v.from.is_empty() {
                self.graph.from.push(k.clone());
            }
            if v.to.is_empty() {
                self.graph.to.push(k.clone());
            }
        }
        self.graph
    }
}
use futures::future::BoxFuture;
/// 异步图执行
pub async fn async_graph<
    K: Hash + Eq + Sized + Clone + Send + Debug + 'static,
    T: AsyncNode + Send + 'static,
    G: DirectedGraph<K, T, Node: Send + 'static> + Send + 'static,
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
>(rt: AsyncRuntime<O, P>, graph: Arc<G>) {
    // 获得图的to节点的数量
    let mut count = graph.to_len();
    let (producor, consumer) = bounded(count);
    for k in graph.from() {
        let an = AsyncGraphNode {
            graph: graph.clone(),
            key: k.clone(),
            producor: producor.clone(),
            _k: PhantomData,
        };
        let end_r = an.exec(rt.clone(), graph.get(k).unwrap());
        count -= end_r.unwrap();
    }
    println!("wait count:{}", count);
    let r = AsyncGraphResult{
        count,
        consumer,
    };
    let _ = r.reduce().await;
}
pub trait AsyncNode {
    /// 获得需要执行的异步块
    fn get(&self) -> Option<BoxFuture<'static, Result<()>>>;
}
/// 异步结果
pub struct AsyncGraphResult {
    count: usize,                           //派发的任务数量
    consumer: Receiver<Result<usize>>, //异步返回值接收器
}
/*
* 异步结果方法
*/
impl AsyncGraphResult {
    /// 归并所有派发的任务
    pub async fn reduce(mut self) -> Result<()> {
        loop {
            match self.consumer.recv_async().await {
                Err(e) => {
                    println!("reduce err:{:?}", e);
                    //接收错误，则立即返回
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("graph result failed, reason: {:?}", e),
                    ));
                }
                Ok(r) => match r {
                    Ok(count) => {
                        //接收成功，则检查是否全部任务都完毕
                        println!("reduce ok:{}", self.count);
                        self.count -= count;
                        if self.count == 0 {
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        println!("reduce err:{:?}", e);
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("graph node failed, reason: {:?}", e),
                        ))
                    }
                },
            }
        }
    }
}
/// 异步图节点执行
pub struct AsyncGraphNode<
    K: Hash + Eq + Sized + Send + Debug + 'static,
    T: AsyncNode + Send + 'static,
    G: DirectedGraph<K, T, Node: Send + 'static> + Send + 'static,
> {
    graph: Arc<G>,
    key: K,
    producor: Sender<Result<usize>>, //异步返回值生成器
    _k: PhantomData<T>,
}
unsafe impl<
        K: Hash + Eq + Sized + Clone + Send + Debug + 'static,
        T: AsyncNode + Send + 'static,
        G: DirectedGraph<K, T, Node: Send + 'static> + Send + 'static,
    > Send for AsyncGraphNode<K, T, G>
{
}

impl<
        K: Hash + Eq + Sized + Clone + Send + Debug + 'static,
        T: AsyncNode + Send + 'static,
        G: DirectedGraph<K, T, Node: Send + 'static> + Send + 'static,
    > AsyncGraphNode<K, T, G>
{
    /// 执行指定异步图节点到指定的运行时，并返回任务同步情况下的结束数量
    pub fn exec<O, P>(self, rt: AsyncRuntime<O, P>, node: &G::Node) -> Result<usize>
    where
        O: Default + 'static,
        P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    {
        let f = node.value().get();
        if f.is_none() { // 该节点为同步节点，没有异步任务
            return self.exec1(rt, node);
        }
        rt.clone().spawn(rt.alloc(), async move {
            // 执行异步任务
            let value = f.unwrap().await;
            match value {
                Err(e) => {
                    let _ = self.producor.into_send_async(Err(e)).await;
                    return Default::default();
                }
                Ok(_) => (),
            }
            // 获取同步执行的结果， 为了不让node引用穿过await，显示声明它的生命周期
            let r = {
                let node = self.graph.get(&self.key).unwrap();
                self.exec1(rt, node)
            };
            if let Ok(0) = r {
                return Default::default();
            }
            let _ = self.producor.into_send_async(r).await;
            Default::default()
        })?;
        Ok(0)
    }
    /// 递归的同步执行
    fn exec1<O, P>(&self, rt: AsyncRuntime<O, P>, node: &G::Node) -> Result<usize>
    where
        O: Default + 'static,
        P: AsyncTaskPoolExt<O> + AsyncTaskPool<O, Pool = P>,
    {
        // 没有后续的节点，则返回结束的数量1
        if node.to_len() == 0 {
            return Ok(1);
        }
        let mut sync_count = 0; // 记录同步返回结束的数量
        for k in node.to() {
            let n = self.graph.get(k).unwrap();
            println!("node: {:?}, count: {} from: {}", n.key(), n.load_count(), n.from_len());
            // 将所有的to节点的计数加1，如果计数为from_len， 则表示全部的依赖都就绪
            if n.add_count(1) + 1 != n.from_len() {
                //println!("node1: {:?}, count: {} ", n.key(), n.load_count());
                continue;
            }
            // 将状态置为0，创建新的AsyncGraphNode并执行
            n.set_count(0);
            let an = AsyncGraphNode {
                graph: self.graph.clone(),
                key: k.clone(),
                producor: self.producor.clone(),
                _k: PhantomData,
            };
            sync_count += an.exec(rt.clone(), n)?;
        }
        return Ok(sync_count);
    }
}


#[test]
fn test_graph() {
    use r#async::rt::multi_thread::MultiTaskRuntimeBuilder;
    use std::time::Duration;
    use std::any::{Any, TypeId};
    use futures::FutureExt;

    struct A (usize);
    impl AsyncNode for A {
        fn get(&self) -> Option<BoxFuture<'static, Result<()>>> {
            let id = self.0;
            Some(async move {
                println!("A id:{}", id);
                Ok(())
            }.boxed())
        }
    }
    struct B (usize);
    impl AsyncNode for B {
        fn get(&self) -> Option<BoxFuture<'static, Result<()>>> {
            println!("B id:{}", self.0);
            None
        }
    }
    pub enum AB {
        A(A),
        B(B),
    }
    impl AsyncNode for AB {
        fn get(&self) -> Option<BoxFuture<'static, Result<()>>> {
            match self {
                AB::A(a) => a.get(),
                AB::B(b) => b.get(),
            }
        }
    }
    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();
    let rt1 = rt0.clone();
    let graph = NGraphBuilder::new().node(1, AB::A(A(1)))
    .node(2, AB::A(A(2)))
    .node(3, AB::A(A(3)))
    .node(4, AB::A(A(4)))
    .node(5, AB::A(A(5)))
    .node(6, AB::A(A(6)))
    .node(7, AB::A(A(7)))
    .node(8, AB::A(A(8)))
    .node(9, AB::B(B(9)))
    .node(10, AB::B(B(10)))
    .node(11, AB::A(A(11)))
    .edge(1, 4)
    .edge(2, 4)
    .edge(2, 5)
    .edge(3, 5)
    .edge(4, 6)
    .edge(4, 7)
    .edge(5, 8)
    .edge(9, 10)
    .edge(10, 11)
    .build();
    let ag = Arc::new(graph);
    let _ = rt0.spawn(rt0.alloc(), async move {
        let _ = async_graph(AsyncRuntime::Multi(rt1.clone()), ag).await;
        println!("ok");
    });
    std::thread::sleep(Duration::from_millis(5000));
}
#[test]
fn test() {
    use r#async::rt::multi_thread::MultiTaskRuntimeBuilder;
    use std::time::Duration;
    let pool = MultiTaskRuntimeBuilder::default();
    let rt0 = pool.build();
    let rt1 = rt0.clone();
    let _ = rt0.spawn(rt0.alloc(), async move {
        let mut map_reduce = rt1.map_reduce(10);
        let rt2 = rt1.clone();
        let rt3 = rt1.clone();
        let _ = map_reduce.map(AsyncRuntime::Multi(rt1.clone()), async move {
            rt1.wait_timeout(300).await;
            println!("1111");
            Ok(1)
        });

        let _ = map_reduce.map(AsyncRuntime::Multi(rt2.clone()), async move {
            rt2.wait_timeout(1000).await;
            println!("2222");
            Ok(2)
        });
        let _ = map_reduce.map(AsyncRuntime::Multi(rt3.clone()), async move {
            rt3.wait_timeout(600).await;
            println!("3333");
            Ok(3)
        });
        for r in map_reduce.reduce(true).await.unwrap() {
            println!("r: {:?}", r);
        }
    });
    std::thread::sleep(Duration::from_millis(5000));
}
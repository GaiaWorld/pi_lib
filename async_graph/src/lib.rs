#![feature(associated_type_bounds)]

//! 异步执行 静态有向无环图 的运行节点

use core::hash::Hash;
use flume::{bounded, Receiver, Sender};
use futures::future::BoxFuture;
use graph::{DirectedGraph, DirectedGraphNode};
use log::debug;
use r#async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result};
use std::marker::PhantomData;
use std::sync::Arc;

/// 同步执行节点
pub trait Runner {
    fn run(self);
}

/// 可运行节点
pub trait Runnble {
    type R: Runner + Send + 'static;

    /// 判断是否同步运行， None表示不是可运行节点，true表示同步运行， false表示异步运行
    fn is_sync(&self) -> Option<bool>;

    /// 获得需要执行的同步函数
    fn get_sync(&self) -> Self::R;

    /// 获得需要执行的异步块
    fn get_async(&self) -> BoxFuture<'static, Result<()>>;
}

/// 异步图执行
pub async fn async_graph<K, R, G, P>(rt: AsyncRuntime<(), P>, graph: Arc<G>) -> Result<()>
where
    K: Hash + Eq + Sized + Clone + Send + Debug + 'static,
    R: Runnble + 'static,
    G: DirectedGraph<K, R, Node: Send + 'static> + Send + 'static,
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    // 获得图的to节点的数量
    let mut count = graph.to_len();
    let (producor, consumer) = bounded(count);
    for k in graph.from() {
        let an = AsyncGraphNode::new(graph.clone(), k.clone(), producor.clone());
        let end_r = an.exec(rt.clone(), graph.get(k).unwrap());
        // 减去立即执行完毕的数量
        count -= end_r.unwrap();
    }
    // debug!("wait count:{}", count);
    let r = AsyncGraphResult { count, consumer };

    r.reduce().await
}

/// 异步结果
pub struct AsyncGraphResult {
    count: usize,                      //派发的任务数量
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
                    //接收错误，则立即返回
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("graph result failed, reason: {:?}", e),
                    ));
                }
                Ok(r) => match r {
                    Ok(count) => {
                        //接收成功，则检查是否全部任务都完毕
                        self.count -= count;
                        if self.count == 0 {
                            return Ok(());
                        }
                    }
                    Err(e) => {
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
pub struct AsyncGraphNode<K, R, G>
where
    K: Hash + Eq + Sized + Send + Debug + 'static,
    R: Runnble,
    G: DirectedGraph<K, R, Node: Send + 'static> + Send + 'static,
{
    graph: Arc<G>,
    key: K,
    producor: Sender<Result<usize>>, //异步返回值生成器
    _k: PhantomData<R>,
}

impl<K, R, G> AsyncGraphNode<K, R, G>
where
    K: Hash + Eq + Sized + Send + Debug + 'static,
    R: Runnble,
    G: DirectedGraph<K, R, Node: Send + 'static> + Send + 'static,
{
    /// 创建
    pub fn new(graph: Arc<G>, key: K, producor: Sender<Result<usize>>) -> Self {
        AsyncGraphNode {
            graph,
            key,
            producor,
            _k: PhantomData,
        }
    }
}

unsafe impl<K, R, G> Send for AsyncGraphNode<K, R, G>
where
    K: Hash + Eq + Sized + Clone + Send + Debug + 'static,
    R: Runnble,
    G: DirectedGraph<K, R, Node: Send + 'static> + Send + 'static,
{
}

impl<K, R, G> AsyncGraphNode<K, R, G>
where
    K: Hash + Eq + Sized + Clone + Send + Debug + 'static,
    R: Runnble + 'static,
    G: DirectedGraph<K, R, Node: Send + 'static> + Send + 'static,
{
    /// 执行指定异步图节点到指定的运行时，并返回任务同步情况下的结束数量
    pub fn exec<P>(self, rt: AsyncRuntime<(), P>, node: &G::Node) -> Result<usize>
    where
        P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
    {
        match node.value().is_sync() {
            None => {
                // 该节点为空节点
                return self.exec_next(rt, node);
            }
            Some(true) => {
                // 同步节点
                let r = node.value().get_sync();
                rt.clone().spawn(rt.alloc(), async move {
                    // 执行同步任务
                    r.run();
                    self.exec_async(rt).await;
                })?;
            }
            _ => {
                let f = node.value().get_async();
                rt.clone().spawn(rt.alloc(), async move {
                    // 执行异步任务
                    let r = f.await;
                    match r {
                        Err(e) => {
                            let _ = self.producor.into_send_async(Err(e)).await;
                            return;
                        }
                        Ok(_) => (),
                    }
                    self.exec_async(rt).await;
                })?;
            }
        }
        Ok(0)
    }

    /// 递归的异步执行
    async fn exec_async<P>(self, rt: AsyncRuntime<(), P>)
    where
        P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
    {
        // 获取同步执行exec_next的结果， 为了不让node引用穿过await，显示声明它的生命周期
        let r = {
            let node = self.graph.get(&self.key).unwrap();
            self.exec_next(rt, node)
        };
        if let Ok(0) = r {
            return;
        }
        let _ = self.producor.into_send_async(r).await;
    }

    /// 递归的同步执行
    fn exec_next<P>(&self, rt: AsyncRuntime<(), P>, node: &G::Node) -> Result<usize>
    where
        P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
    {
        // 没有后续的节点，则返回结束的数量1
        if node.to_len() == 0 {
            return Ok(1);
        }
        let mut sync_count = 0; // 记录同步返回结束的数量
        for k in node.to() {
            let n = self.graph.get(k).unwrap();
            // debug!("node: {:?}, count: {} from: {}", n.key(), n.load_count(), n.from_len());
            // 将所有的to节点的计数加1，如果计数为from_len， 则表示全部的依赖都就绪
            if n.add_count(1) + 1 != n.from_len() {
                //debug!("node1: {:?}, count: {} ", n.key(), n.load_count());
                continue;
            }
            // 将状态置为0，创建新的AsyncGraphNode并执行
            n.set_count(0);
            let an = AsyncGraphNode::new(self.graph.clone(), k.clone(), self.producor.clone());
            sync_count += an.exec(rt.clone(), n)?;
        }
        return Ok(sync_count);
    }
}

pub trait RunFactory {
    type R: Runner;
    fn create(&self) -> Self::R;
}

pub enum ExecNode<Run, Fac>
where
    Run: Runner,
    Fac: RunFactory<R = Run>,
{
    None,
    Sync(Fac),
    Async(Box<dyn Fn() -> BoxFuture<'static, Result<()>> + 'static + Send + Sync>),
}

impl<Run, Fac> Runnble for ExecNode<Run, Fac>
where
    Run: Runner + Send + 'static,
    Fac: RunFactory<R = Run>,
{
    type R = Run;

    fn is_sync(&self) -> Option<bool> {
        match self {
            ExecNode::None => None,
            ExecNode::Sync(_) => Some(true),
            _ => Some(false),
        }
    }

    fn get_sync(&self) -> Self::R {
        match self {
            ExecNode::Sync(r) => r.create(),
            _ => panic!(),
        }
    }

    fn get_async(&self) -> BoxFuture<'static, Result<()>> {
        match self {
            ExecNode::Async(f) => f(),
            _ => panic!(),
        }
    }
}

mod tests {
    use crate::*;
    use std::sync::Once;

    static INIT: Once = Once::new();
    fn setup_logger() {
        INIT.call_once(|| {
            use env_logger::{Builder, Env};
            Builder::from_env(Env::default().default_filter_or("debug")).init();
        });
    }

    // 模拟 渲染图 调用
    #[test]
    fn test_render_graph() {
        use futures::FutureExt;
        use graph::NGraphBuilder;
        use log::info;
        use r#async::rt::{multi_thread::MultiTaskRuntimeBuilder, AsyncRuntime};
        use std::time::Duration;

        // 异步函数不需要
        struct DumpNode;
        impl Runner for DumpNode {
            fn run(self) {}
        }
        impl RunFactory for DumpNode {
            type R = DumpNode;
            fn create(&self) -> Self::R {
                DumpNode
            }
        }

        // 异步调用
        fn asyn(id: usize) -> ExecNode<DumpNode, DumpNode> {
            let f = move || -> BoxFuture<'static, Result<()>> {
                async move {
                    info!("async id:{}", id);

                    Ok(())
                }
                .boxed()
            };

            ExecNode::Async(Box::new(f))
        }

        setup_logger();

        let pool = MultiTaskRuntimeBuilder::default();
        let rt0 = pool.build();
        let rt1 = rt0.clone();

        // 1 --> 3, 4
        // 2 --> 5
        // 3 --> 6
        // 5 --> 6
        let graph = NGraphBuilder::new()
            .node(1, asyn(1))
            .node(2, asyn(2))
            .node(3, ExecNode::None)
            .node(4, ExecNode::None)
            .node(5, ExecNode::None)
            .node(6, asyn(6))
            .edge(1, 3)
            .edge(1, 4)
            .edge(2, 5)
            .edge(3, 6)
            .edge(5, 6)
            .build()
            .unwrap();

        let ag = Arc::new(graph);

        let _ = rt0.spawn(rt0.alloc(), async move {
            let _ = async_graph(AsyncRuntime::Multi(rt1), ag).await;
            
            debug!("ok");
        });

        std::thread::sleep(Duration::from_millis(2000));
    }

    #[test]
    fn test_graph() {
        use futures::FutureExt;
        use graph::NGraphBuilder;
        use log::debug;
        use r#async::rt::{multi_thread::MultiTaskRuntimeBuilder, AsyncRuntime};
        use std::time::Duration;

        struct A(usize);

        // A 是可执行节点
        impl Runner for A {
            fn run(self) {
                debug!("A id:{}", self.0);
            }
        }

        struct B(usize);

        // B 是 执行节点 工厂
        // 负责 生成 可执行节点
        impl RunFactory for B {
            type R = A;

            fn create(&self) -> A {
                A(self.0)
            }
        }

        // 同步调用
        fn syn(id: usize) -> ExecNode<A, B> {
            ExecNode::Sync(B(id))
        }

        // 异步调用
        fn asyn(id: usize) -> ExecNode<A, B> {
            let f = move || -> BoxFuture<'static, Result<()>> {
                async move {
                    debug!("async id:{}", id);
                    Ok(())
                }
                .boxed()
            };
            ExecNode::Async(Box::new(f))
        }

        setup_logger();

        let pool = MultiTaskRuntimeBuilder::default();
        let rt0 = pool.build();
        let rt1 = rt0.clone();
        let graph = NGraphBuilder::new()
            .node(1, asyn(1))
            .node(2, asyn(2))
            .node(3, syn(3))
            .node(4, asyn(4))
            .node(5, asyn(5))
            .node(6, asyn(6))
            .node(7, asyn(7))
            .node(8, asyn(8))
            .node(9, asyn(9))
            .node(10, ExecNode::None)
            .node(11, syn(11))
            .edge(1, 4)
            .edge(2, 4)
            .edge(2, 5)
            .edge(3, 5)
            .edge(4, 6)
            .edge(4, 7)
            .edge(5, 8)
            .edge(9, 10)
            .edge(10, 11)
            .build()
            .unwrap();

        let ag = Arc::new(graph);

        let _ = rt0.spawn(rt0.alloc(), async move {
            let _: _ = async_graph(AsyncRuntime::Multi(rt1), ag).await;
            debug!("ok");
        });

        std::thread::sleep(Duration::from_millis(2000));
    }

    #[test]
    fn test() {
        use log::debug;
        use r#async::rt::{multi_thread::MultiTaskRuntimeBuilder, AsyncRuntime};
        use std::time::Duration;

        setup_logger();

        let pool = MultiTaskRuntimeBuilder::default();
        let rt0 = pool.build();
        let rt1 = rt0.clone();

        let _ = rt0.spawn(rt0.alloc(), async move {
            let mut map_reduce = rt1.map_reduce(10);
            let rt2 = rt1.clone();
            let rt3 = rt1.clone();
            let _ = map_reduce.map(AsyncRuntime::Multi(rt1.clone()), async move {
                rt1.wait_timeout(300).await;
                debug!("1111");
                Ok(1)
            });

            let _ = map_reduce.map(AsyncRuntime::Multi(rt2.clone()), async move {
                rt2.wait_timeout(1000).await;
                debug!("2222");
                Ok(2)
            });

            let _ = map_reduce.map(AsyncRuntime::Multi(rt3.clone()), async move {
                rt3.wait_timeout(600).await;
                debug!("3333");
                Ok(3)
            });

            for r in map_reduce.reduce(true).await.unwrap() {
                debug!("r: {:?}", r);
            }
        });

        std::thread::sleep(Duration::from_millis(2000));
    }
}

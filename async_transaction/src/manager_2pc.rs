use std::any::Any;
use std::fmt::Debug;
use std::io::{Error, Result as IOResult, ErrorKind};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use bytes::BufMut;
use dashmap::DashMap;

use r#async::rt::{AsyncRuntime, multi_thread::MultiTaskRuntime};
use guid::{GuidGen, Guid};
use atom::Atom;

use super::{ErrorLevel,
            TransactionError,
            AsyncTransaction,
            Transaction2Pc,
            TransactionTree,
            AsyncCommitLog};
use futures::future::{FutureExt, BoxFuture};

/*
* 默认的事务唯一id的CtrlId
*/
const DEFAULT_TRANSACTION_CTRL_ID: u16 = 0;

/*
* 默认的事务预提交唯一id的CtrlId
*/
const DEFAULT_TRANSACTION_PREPARE_CTRL_ID: u16 = 1;

/*
* 默认的事务提交唯一id的CtrlId
*/
const DEFAULT_TRANSACTION_COMMIT_CTRL_ID: u16 = 2;

/*
* 默认的指定事务源的最大同时处理事务数
*/
const DEFAULT_MAX_PARALLEL_TRANSACTION_LIMIT: usize = usize::MAX;

///
/// 两阶段提交事务的状态
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transaction2PcStatus {
    Start = 0,          //开始事务
    Initing,            //正在初始化
    Inited,             //已初始化
    InitFailed,         //初始化失败
    Actioning,          //正在操作
    Actioned,           //已操作
    ActionFailed,       //操作失败
    Prepareing,         //正在预提交
    Prepared,           //已预提交
    PrepareFailed,      //预提交失败
    LogCommiting,       //正在提交日志
    LogCommited,        //已提交日志
    LogCommitFailed,    //提交日志失败
    Commiting,          //正在提交
    Commited,           //已提交
    CommitFailed,       //提交失败，错误级别为严重
    Rollbacking,        //正在回滚
    Rollbacked,         //已回滚
    RollbackFailed,     //回滚失败，错误级别为严重
}

impl Default for Transaction2PcStatus {
    fn default() -> Self {
        Transaction2PcStatus::Start
    }
}

///
/// 事务两阶段提交管理器
/// 负责执行和管理所有两阶段事务
/// 为所有两阶段事务提供原子的持久化保证
///
pub struct Transaction2PcManager<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
>(Arc<Inner2PcManager<C, Log>>);

unsafe impl<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
> Send for Transaction2PcManager<C, Log> {}
unsafe impl<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
> Sync for Transaction2PcManager<C, Log> {}

impl<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
> Clone for Transaction2PcManager<C, Log> {
    fn clone(&self) -> Self {
        Transaction2PcManager(self.0.clone())
    }
}

/*
* 事务两阶段提交管理器同步方法
*/
impl<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
> Transaction2PcManager<C, Log> {
    /// 构建一个事务两阶段提交管理器
    pub fn new(rt: MultiTaskRuntime<()>,
               uid_gen: GuidGen,
               commit_logger: Log) -> Self {
        let inner = Inner2PcManager {
            rt,
            uid_gen,
            commit_logger,
            trans_table: DashMap::default(),
            source_counter: DashMap::default(),
            prepare_produced: AtomicUsize::new(0),
            prepare_consumed: AtomicUsize::new(0),
            commit_produced: AtomicUsize::new(0),
            commit_consumed: AtomicUsize::new(0),
            produced_total: AtomicUsize::new(0),
            consumed_total: AtomicUsize::new(0),
        };

        Transaction2PcManager(Arc::new(inner))
    }

    /// 获取事务两阶段提交管理器的提交日志记录器
    pub fn commit_logger(&self) -> Log {
        self.0.commit_logger.clone()
    }

    /// 获取当前已开始，且未结束的事务数量
    pub fn transaction_len(&self) -> usize {
        self.0.trans_table.len()
    }

    /// 获取指定事务源，当前已开始，且未结束的事务数量
    pub fn source_len(&self, source: &Atom) -> Option<usize> {
        if let Some(counter) = self.0.source_counter.get(source) {
            let (add, sub, _parallel_limit) = counter.value();
            Some(add
                .load(Ordering::Relaxed)
                .checked_sub(sub.load(Ordering::Relaxed))
                .unwrap_or(0))
        } else {
            None
        }
    }

    /// 获取指定事务源，同时允许处理的最大事务数量限制
    pub fn get_max_source_parallel_limit(&self, source: &Atom) -> Option<usize> {
        if let Some(counter) = self.0.source_counter.get(source) {
            let (_add, _sub, parallel_limit) = counter.value();
            Some(parallel_limit.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    /// 设置指定事务源，同时允许处理的最大事务数量限制
    pub fn set_max_source_parallel_limit(&self, source: &Atom, limit: usize) {
        if let Some(counter) = self.0.source_counter.get(source) {
            //指定事务源已存在
            let (_add, _sub, parallel_limit) = counter.value();
            parallel_limit.store(limit, Ordering::SeqCst);
        } else {
            //指定事务源不存在
            self.0.source_counter.insert(source.clone(),
                                         (AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(limit)));
        }
    }

    /// 获取当前正在预提交的事务数量
    pub fn prepare_len(&self) -> usize {
        self
            .0
            .prepare_produced
            .load(Ordering::Relaxed)
            .checked_sub(self.0.prepare_consumed.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// 获取当前正在提交的事务数量
    pub fn commit_len(&self) -> usize {
        self.0
            .commit_produced
            .load(Ordering::Relaxed)
            .checked_sub(self.0.commit_consumed.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// 获取开始事务的总数量
    pub fn produced_transaction_total(&self) -> usize {
        self.0.produced_total.load(Ordering::Relaxed)
    }

    /// 获取结束事务的总数量
    pub fn consumed_transaction_total(&self) -> usize {
        self.0.consumed_total.load(Ordering::Relaxed)
    }

    /// 获取指定唯一id的事务状态
    pub fn get_transaction_status<T>(&self, uid: &Guid) -> Option<Transaction2PcStatus>
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
        if let Some(shared_tr) = self.0.trans_table.get(uid) {
            if let Some(tr) = <dyn Any>::downcast_ref::<T>(shared_tr.value()) {
                return Some(tr.get_status());
            }
        }

        None
    }

    /// 同步结束指定的事务
    pub fn finish<T>(&self, tr: T)
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
        let status = tr.get_status();
        if status == Transaction2PcStatus::Start
            || status == Transaction2PcStatus::Initing
            || status == Transaction2PcStatus::Actioning
            || status == Transaction2PcStatus::Prepareing
            || status == Transaction2PcStatus::LogCommiting
            || status == Transaction2PcStatus::Commiting
            || status == Transaction2PcStatus::Rollbacking
            || status == Transaction2PcStatus::CommitFailed
            || status == Transaction2PcStatus::RollbackFailed {
            //事务正在执行中，或事务出现严重错误，则不允许完成事务，并保持事务当前状态
            return;
        }

        if let Some(transaction_uid) = tr.get_transaction_uid() {
            //注销已注册的事务
            if let Some(counter) = self.0.source_counter.get(&tr.get_source()) {
                //指定事务源已存在，则减少计数
                counter.value().1.fetch_add(1, Ordering::Relaxed);
            }
            let _ = self.0.trans_table.remove(&transaction_uid);
        }
        self.0.consumed_total.fetch_add(1, Ordering::Relaxed); //增加结束事务的总数量
    }
}

/*
* 事务两阶段提交管理器异步方法
*/
impl<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
> Transaction2PcManager<C, Log> {
    /// 异步开始指定的事务
    pub async fn start<T>(&self, tr: T)
        -> Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
        let current_tr_status = tr.get_status();
        if current_tr_status != Transaction2PcStatus::Start {
            //事务未开始，则不允许初始化
            tr.set_status(Transaction2PcStatus::InitFailed); //更新事务状态为初始化失败
            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Init root failed, type: unit, status: {:?}, reason: invalid transaction status", current_tr_status)));
        }

        tr.set_transaction_uid(alloc_transaction_uid(&self.0.uid_gen, &tr)); //设置根事务的唯一id
        if let Err(current_len) = register_transcation(&self, &tr) {
            //注册根事务失败，指定事务所在事务源的同时处理事务数已达限制
            tr.set_status(Transaction2PcStatus::InitFailed); //更新事务状态为初始化失败
            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Init root failed, type: unit, current: {}, status: {:?}, reason: same source transaction excessive", current_len, tr.get_status())));
        }
        tr.set_status(Transaction2PcStatus::Initing); //更新事务状态为正在初始化

        let result = self.init_childrens(tr.clone()).await;

        if result.is_err() {
            //初始化事务失败
            tr.set_status(Transaction2PcStatus::InitFailed); //更新事务状态为初始化失败
        } else {
            //初始化事务成功
            tr.set_status(Transaction2PcStatus::Inited); //更新事务状态为已初始化
        }

        result
    }

    /// 初始化子事务，只允许在本地初始化事务
    fn init_childrens<T>(&self, tr: T)
                          -> BoxFuture<Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>>
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
        let mgr = self.clone();

        async move {
            if tr.is_unit() {
                //初始化单元事务
                return tr.init().await;
            }

            //初始化事务树
            let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();

            for child in childs {
                if child.is_unit() {
                    //当前事务的子事务是单元事务，则初始化子单元事务
                    child.set_transaction_uid(alloc_transaction_uid(&mgr.0.uid_gen, &tr)); //设置子单元事务的唯一id
                    child.set_status(Transaction2PcStatus::Initing); //更新子单元事务状态为正在初始化

                    if let Err(e) = child.init().await {
                        //子事务初始化失败
                        child.set_status(Transaction2PcStatus::InitFailed); //更新子单元事务状态为初始化失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Init children failed, type: unit, reason: {:?}", e)));
                    }

                    child.set_status(Transaction2PcStatus::Inited); //更新子单元事务状态为已初始化
                } else if child.is_tree() {
                    //当前事务的子事务是事务树，则初始化子事务树
                    if let Err(e) = mgr.start(child).await {
                        //子事务初始化失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Init children failed, type: tree, reason: {:?}", e)));
                    }
                } else {
                    //当前事务的子事务是无效的事务
                    child.set_status(Transaction2PcStatus::InitFailed); //更新子事务状态为初始化失败
                    return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, "Init transaction failed, reason: invalid transaction type"));
                }
            }

            //所有子事务初始化已成功，则初始化根事务
            match tr.init().await {
                Err(e) => {
                    //根事务初始化失败
                    Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Init root failed, type: tree, reason: {:?}", e)))
                },
                Ok(output) => {
                    //根事务初始化成功
                    Ok(output)
                },
            }
        }.boxed()
    }

    /// 异步预提交，成功返回需要写入提交日志的数据
    pub async fn prepare<T>(&self, tr: T)
        -> Result<Option<<T as Transaction2Pc>::PrepareOutput>, <T as Transaction2Pc>::PrepareError>
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {

        let current_tr_status = tr.get_status();
        if current_tr_status != Transaction2PcStatus::Inited
            && current_tr_status != Transaction2PcStatus::Actioned
            && current_tr_status != Transaction2PcStatus::Rollbacked {
            //事务未初始化、完成操作或未回滚成功，则不允许预提交
            tr.set_status(Transaction2PcStatus::PrepareFailed); //更新事务状态为预提交失败
            return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare root failed, type: unit, transaction_uid: {:?}, prepare_uid: {:?}, status: {:?}, reason: invalid transaction status", tr.get_transaction_uid(), tr.get_prepare_uid(), current_tr_status)));
        }

        if !tr.is_writable() {
            //只读事务不需要预提交，则立即返回预提交成功
            tr.set_status(Transaction2PcStatus::Prepared); //更新事务状态为已预提交
            return Ok(None);
        }

        self.0.prepare_produced.fetch_add(1, Ordering::Relaxed); //增加开始预提交数量
        tr.set_status(Transaction2PcStatus::Prepareing); //更新事务状态为正在预提交

        let result = if tr.is_unit() {
            //预提交单元事务
            tr.prepare().await
        } else {
            //预提交事务树
            self.prepare_childrens(tr.clone()).await
        };

        if result.is_err() {
            //预提交事务失败
            tr.set_status(Transaction2PcStatus::PrepareFailed); //更新事务状态为预提交失败
        } else {
            //预提交事务成功
            tr.set_status(Transaction2PcStatus::Prepared); //更新事务状态为已预提交
        }
        self.0.prepare_consumed.fetch_add(1, Ordering::Relaxed); //增加结束预提交数量

        result
    }

    // 可写子事务的异步预提交
    fn prepare_childrens<T>(&self, tr: T)
        -> BoxFuture<Result<Option<<T as Transaction2Pc>::PrepareOutput>, <T as Transaction2Pc>::PrepareError>>
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
        let mgr = self.clone();

        async move {
            if tr.is_require_persistence() {
                //只为需要持久化的根事务，分配提交唯一id
                tr.set_commit_uid(alloc_commit_uid(&mgr.0.uid_gen, &tr)); //设置根事务的提交唯一id
            }

            if tr.is_concurrent_prepare() {
                //需要并发预提交，一般用于远端预提交
                let mut map_reduce = mgr.0.rt.map_reduce(tr.children_len());
                let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();

                //映射子事务的预提交
                for child in childs {
                    let child_uid = child.get_transaction_uid();
                    let prepare_uid = child.get_prepare_uid();

                    if child.is_unit() {
                        //当前事务的子事务是单元事务
                        if child.is_require_persistence() {
                            //只为需要持久化的子事务，分配提交唯一id
                            child.set_commit_uid(alloc_commit_uid(&mgr.0.uid_gen, &tr)); //设置子单元事务的提交唯一id
                        }
                        let child_copy = child.clone();

                        if let Err(e) = map_reduce.map(AsyncRuntime::Multi(mgr.0.rt.clone()), async move {
                            //执行子单元事务的预提交
                            child_copy.set_status(Transaction2PcStatus::Prepareing); //更新子单元事务状态为正在预提交

                            match child_copy.prepare().await {
                                Err(e) => {
                                    //子事务预提交失败
                                    child_copy.set_status(Transaction2PcStatus::PrepareFailed); //更新子单元事务状态为预提交失败
                                    Err(Error::new(ErrorKind::Other, format!("Prepare children failed, type: unit, child_uid: {:?}, prepare_uid: {:?}, reason: {:?}", child_copy.get_transaction_uid(), child_copy.get_prepare_uid(), e)))
                                },
                                Ok(output) => {
                                    //子事务预提交成功
                                    child_copy.set_status(Transaction2PcStatus::Prepared); //更新子单元事务状态为已预提交
                                    Ok(output)
                                },
                            }
                        }) {
                            //映射子事务的预提交操作失败
                            child.set_status(Transaction2PcStatus::PrepareFailed); //更新子单元事务状态为预提交失败
                            return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Map children prepare failed, type: unit, child_uid: {:?}, prepare_uid: {:?}, reason: {:?}", child_uid, prepare_uid, e)));
                        };
                    } else if child.is_tree() {
                        //当前事务的子事务是事务树
                        let mgr_copy = mgr.clone();

                        if let Err(e) = map_reduce.map(AsyncRuntime::Multi(mgr.0.rt.clone()), async move {
                            //执行子事务树的预提交
                            let child_uid = child.get_transaction_uid();
                            let prepare_uid = child.get_prepare_uid();

                            match mgr_copy.prepare(child).await {
                                Err(e) => {
                                    //子事务预提交失败
                                    Err(Error::new(ErrorKind::Other, format!("Prepare children failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, reason: {:?}", child_uid, prepare_uid, e)))
                                },
                                Ok(output) => {
                                    //子事务预提交成功
                                    Ok(output)
                                },
                            }
                        }) {
                            //映射子事务的预提交操作失败
                            return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Map children prepare failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, reason: {:?}", child_uid, prepare_uid, e)));
                        };
                    } else {
                        //当前事务的子事务是无效的事务
                        child.set_status(Transaction2PcStatus::PrepareFailed); //更新子事务状态为预提交失败
                        return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare transaction failed, child_uid: {:?}, prepare_uid: {:?}, reason: invalid transaction type", child_uid, prepare_uid)));
                    }
                }

                //归并子事务的预提交
                match map_reduce.reduce(true).await {
                    Err(e) => {
                        //归并子事务的预提交操作失败
                        Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Reduce children prepare failed, type: tree, transaction_uid: {:?}, prepare_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), e)))
                    },
                    Ok(results) => {
                        //归并子事务的预提交操作成功
                        let mut childs_output: Vec<u8> = Vec::new();
                        for result in results {
                            match result {
                                Err(e) => {
                                    //子事务预提交失败
                                    return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, e));
                                },
                                Ok(child_output) => {
                                    //子事务预提交成功，则将返回的预提交输出写入二进制缓冲区
                                    if let Some(child_output) = child_output {
                                        //子事务是可写事务
                                        let buf = child_output.as_ref();
                                        if buf.len() > 0 {
                                            //预提交输出长度大于0
                                            childs_output.put_slice(buf);
                                        }
                                    }
                                },
                            }
                        }

                        //所有子事务预提交已成功，则执行根事务预提交
                        match tr.prepare().await {
                            Err(e) => {
                                //根事务提交失败
                                Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare root failed, type: tree, transaction_uid: {:?}, prepare_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), e)))
                            },
                            Ok(output) => {
                                //根事务预提交成功，则将返回的预提交输出写入二进制缓冲区
                                if let Some(mut output) = output {
                                    //根事务是可写事务
                                    if childs_output.len() > 0 {
                                        //所有子事务的预提交输出长度大于0
                                        output.put_slice(childs_output.as_ref());
                                    }

                                    Ok(Some(output))
                                } else {
                                    //根事务是只读事务
                                    Ok(None)
                                }
                            },
                        }
                    }
                }
            } else {
                //不需要并发预提交，一般用于本地预提交
                let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();
                let mut childs_output: Vec<u8> = Vec::new();

                for child in childs {
                    if child.is_unit() {
                        //当前事务的子事务是单元事务，则执行子单元事务的预提交
                        if child.is_require_persistence() {
                            //只为需要持久化的子事务，分配提交唯一id
                            child.set_commit_uid(alloc_commit_uid(&mgr.0.uid_gen, &tr)); //设置子单元事务的提交唯一id
                        }
                        child.set_status(Transaction2PcStatus::Prepareing); //更新子单元事务状态为正在预提交

                        match child.prepare().await {
                            Err(e) => {
                                //子事务预提交失败
                                child.set_status(Transaction2PcStatus::PrepareFailed); //更新子单元事务状态为预提交失败
                                return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare children failed, type: unit, child_uid: {:?}, prepare_uid: {:?}, reason: {:?}", child.get_transaction_uid(), child.get_prepare_uid(), e)));
                            },
                            Ok(child_output) => {
                                //子事务预提交成功，则将返回的预提交输出写入二进制缓冲区
                                child.set_status(Transaction2PcStatus::Prepared); //更新子单元事务状态为已预提交

                                if let Some(child_output) = child_output {
                                    //子事务是可写事务
                                    let buf = child_output.as_ref();
                                    if buf.len() > 0 {
                                        //预提交输出长度大于0
                                        childs_output.put_slice(buf);
                                    }
                                }
                            },
                        }
                    } else if child.is_tree() {
                        //当前事务的子事务是事务树，则执行子事务树的预提交
                        let child_uid = child.get_transaction_uid();
                        let prepare_uid = child.get_prepare_uid();

                        match mgr.prepare(child).await {
                            Err(e) => {
                                //子事务预提交失败
                                return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare children failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, reason: {:?}", child_uid, prepare_uid, e)));
                            },
                            Ok(child_output) => {
                                //子事务预提交成功，则将返回的预提交输出写入二进制缓冲区
                                if let Some(child_output) = child_output {
                                    //子事务是可写事务
                                    let buf = child_output.as_ref();
                                    if buf.len() > 0 {
                                        //预提交输出长度大于0
                                        childs_output.put_slice(buf);
                                    }
                                }
                            },
                        }
                    } else {
                        //当前事务的子事务是无效的事务
                        child.set_status(Transaction2PcStatus::PrepareFailed); //更新子事务状态为预提交失败
                        return Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare transaction failed, child_uid: {:?}, prepare_uid: {:?}, reason: invalid transaction type", child.get_transaction_uid(), child.get_prepare_uid())));
                    }
                }

                //所有子事务预提交已成功，则执行根事务预提交
                match tr.prepare().await {
                    Err(e) => {
                        //根事务提交失败
                        Err(<T as Transaction2Pc>::PrepareError::new_transaction_error(ErrorLevel::Normal, format!("Prepare root failed, type: tree, transaction_uid: {:?}, prepare_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), e)))
                    },
                    Ok(output) => {
                        //根事务预提交成功，则将返回的预提交输出写入二进制缓冲区
                        if let Some(mut output) = output {
                            //根事务是可写事务
                            if childs_output.len() > 0 {
                                //所有子事务的预提交输出长度大于0
                                output.put_slice(childs_output.as_ref());
                            }

                            Ok(Some(output))
                        } else {
                            //根事务是只读事务
                            Ok(None)
                        }
                    },
                }
            }
        }.boxed()
    }

    /// 异步提交，需要将当前事务预提交成功后返回的数据写入提交日志
    pub async fn commit<T>(&self,
                           tr: T,
                           input: <T as Transaction2Pc>::PrepareOutput,
                           confirm: <T as Transaction2Pc>::CommitConfirm)
                           -> Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>
        where T: TransactionTree<Cid = Guid, Node = T, Status = Transaction2PcStatus> {

        let current_tr_status = tr.get_status();
        if current_tr_status != Transaction2PcStatus::Prepared {
            //事务未完成预提交，则不允许提交日志
            tr.set_status(Transaction2PcStatus::LogCommitFailed); //更新事务状态为提交日志失败
            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Commit root failed, type: unit, transaction_uid: {:?}, commit_uid: {:?}, status: {:?}, reason: invalid transaction status", tr.get_transaction_uid(), tr.get_commit_uid(), current_tr_status)));
        }

        if tr.is_writable() && tr.is_require_persistence() {
            //可写且需要持久化的事务，必须首先写入提交日志
            self.0.commit_produced.fetch_add(1, Ordering::Relaxed); //增加开始提交数量
            tr.set_status(Transaction2PcStatus::LogCommiting); //更新事务状态为正在提交日志

            match self.0.commit_logger.append(tr.get_commit_uid().unwrap(),
                                              input).await {
                Err(e) => {
                    //同步追加提交日志失败
                    tr.set_status(Transaction2PcStatus::LogCommitFailed); //更新事务状态为提交日志失败
                    return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Commit transaction failed, type: tree, transaction_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_commit_uid(), e)));
                },
                Ok(log_handle) => {
                    //同步追加提交日志成功，则立即异步刷新提交日志
                    if let Err(e) = self.0.commit_logger.flush(log_handle).await {
                        //异步刷新提交日志失败
                        tr.set_status(Transaction2PcStatus::LogCommitFailed); //更新事务状态为提交日志失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Commit transaction failed, type: tree, transaction_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_commit_uid(), e)));
                    }

                    tr.set_status(Transaction2PcStatus::LogCommited); //更新事务状态为已提交日志
                },
            }
        } else if !tr.is_require_persistence() {
            //可写且不需要持久化的事务，不需要提交日志和提交确认，但仍然需要提交
            tr.set_status(Transaction2PcStatus::LogCommited); //更新事务状态为已提交日志
        } else if !tr.is_writable() {
            //只读事务，不需要提交日志，提交和确认，则立即返回提交成功
            tr.set_status(Transaction2PcStatus::Commited); //更新事务状态为已提交
            return Ok(<T as AsyncTransaction>::Output::default());
        }

        let current_tr_status = tr.get_status();
        if current_tr_status != Transaction2PcStatus::LogCommited {
            //事务未完成提交日志，则不允许提交
            tr.set_status(Transaction2PcStatus::CommitFailed); //更新事务状态为提交失败
            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Commit root failed, type: unit, transaction_uid: {:?}, commit_uid: {:?}, status: {:?}, reason: invalid transaction status", tr.get_transaction_uid(), tr.get_commit_uid(), current_tr_status)));
        }

        let result = self.commit_confirm(tr.clone(), confirm).await;

        if result.is_err() {
            //提交事务失败
            tr.set_status(Transaction2PcStatus::CommitFailed); //更新事务状态为提交失败
        } else {
            //提交事务成功
            tr.set_status(Transaction2PcStatus::Commited); //更新事务状态为已提交
        }
        self.0.commit_consumed.fetch_add(1, Ordering::Relaxed); //增加结束提交数量

        result
    }

    // 事务的提交和确认
    fn commit_confirm<T>(&self,
                         tr: T,
                         confirm: <T as Transaction2Pc>::CommitConfirm)
      -> BoxFuture<Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>>
        where T: TransactionTree<Node = T, Status = Transaction2PcStatus> {
        let mgr = self.clone();

        async move {
            let current_tr_status = tr.get_status();
            if current_tr_status != Transaction2PcStatus::LogCommited {
                //事务未完成提交日志，则不允许提交
                tr.set_status(Transaction2PcStatus::CommitFailed); //更新事务状态为提交失败
                return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Commit root failed, type: unit, transaction_uid: {:?}, commit_uid: {:?}, status: {:?}, reason: invalid transaction status", tr.get_transaction_uid(), tr.get_commit_uid(), current_tr_status)));
            }

            tr.set_status(Transaction2PcStatus::LogCommiting); //更新事务状态为正在提交

            if tr.is_unit() {
                //延迟提交单元事务
                return tr.commit(confirm).await;
            }

            //延迟提交事务树
            if tr.is_concurrent_commit() {
                //需要并发延迟提交，一般用于远端延迟提交
                let mut map_reduce = mgr.0.rt.map_reduce(tr.children_len());
                let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();

                for child in childs {
                    let child_uid = child.get_transaction_uid();
                    let commit_uid = child.get_commit_uid();

                    if child.is_unit() {
                        //当前事务的子事务是单元事务
                        let child_copy = child.clone();
                        let confirm_copy = confirm.clone();

                        if let Err(e) = map_reduce.map(AsyncRuntime::Multi(mgr.0.rt.clone()), async move {
                            //执行子单元事务的延迟提交
                            child_copy.set_status(Transaction2PcStatus::Commiting); //更新子单元事务状态为正在提交

                            match child_copy.commit(confirm_copy).await {
                                Err(e) => {
                                    //子事务延迟提交失败
                                    child_copy.set_status(Transaction2PcStatus::Commiting); //更新子单元事务状态为提交失败
                                    Err(Error::new(ErrorKind::Other, format!("Commit children failed, type: unit, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_copy.get_transaction_uid(), child_copy.get_commit_uid(), e)))
                                },
                                Ok(output) => {
                                    //子事务延迟提交成功
                                    child_copy.set_status(Transaction2PcStatus::Commited); //更新子单元事务状态为已提交
                                    Ok(output)
                                },
                            }
                        }) {
                            //映射子事务的延迟提交操作失败
                            child.set_status(Transaction2PcStatus::CommitFailed); //更新子单元事务状态为提交失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Map children commit failed, type: unit, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, commit_uid, e)));
                        };
                    } else if child.is_tree() {
                        //当前事务的子事务是事务树
                        let mgr_copy = mgr.clone();
                        let child_copy = child.clone();
                        let confirm_copy = confirm.clone();

                        if let Err(e) = map_reduce.map(AsyncRuntime::Multi(mgr.0.rt.clone()), async move {
                            //执行子事务树的延迟提交
                            let child_uid = child.get_transaction_uid();
                            let commit_uid = child.get_commit_uid();
                            child_copy.set_status(Transaction2PcStatus::Commiting); //更新子事务树状态为正在提交

                            match mgr_copy.commit_confirm(child, confirm_copy).await {
                                Err(e) => {
                                    //子事务延迟提交失败
                                    child_copy.set_status(Transaction2PcStatus::CommitFailed); //更新子事务树状态为提交失败
                                    Err(Error::new(ErrorKind::Other, format!("Commit children failed, type: tree, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, commit_uid, e)))
                                },
                                Ok(output) => {
                                    //子事务延迟务提交成功
                                    child_copy.set_status(Transaction2PcStatus::Commited); //更新子事务树状态为已提交
                                    Ok(output)
                                },
                            }
                        }) {
                            //映射子事务的延迟提交操作失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Map children commit failed, type: tree, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, commit_uid, e)));
                        };
                    } else {
                        //当前事务的子事务是无效的事务
                        child.set_status(Transaction2PcStatus::CommitFailed); //更新子事务状态为提交失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit transaction failed, child_uid: {:?}, commit_uid: {:?}, reason: invalid transaction type", child_uid, commit_uid)));
                    }
                }

                //归并子事务的延迟提交
                match map_reduce.reduce(true).await {
                    Err(e) => {
                        //归并子事务的延迟提交操作失败
                        Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Reduce children commit failed, type: tree, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_commit_uid(), e)))
                    },
                    Ok(results) => {
                        //归并子事务的延迟提交操作成功
                        for result in results {
                            if let Err(e) = result {
                                //子事务延迟提交失败
                                return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit children failed, type: tree, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_commit_uid(), e)));
                            }
                        }

                        //所有子事务延迟提交已成功，则执行根事务延迟提交
                        match tr.commit(confirm).await {
                            Err(e) => {
                                //根事务延迟提交失败
                                Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit root failed, type: tree, transaction_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_commit_uid(), e)))
                            },
                            Ok(output) => {
                                //根事务延迟提交成功
                                Ok(output)
                            },
                        }
                    }
                }
            } else {
                //不需要并发延迟提交，一般用于本地延迟提交
                let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();

                for child in childs {
                    if child.is_unit() {
                        //当前事务的子事务是单元事务，则执行子单元事务的延迟提交
                        child.set_status(Transaction2PcStatus::Commiting); //更新子单元事务状态为正在提交

                        if let Err(e) = child.commit(confirm.clone()).await {
                            //子事务延迟提交失败
                            child.set_status(Transaction2PcStatus::CommitFailed); //更新子单元事务状态为提交失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit children failed, type: unit, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", child.get_transaction_uid(), child.get_commit_uid(), e)));
                        }

                        child.set_status(Transaction2PcStatus::Commited); //更新子单元事务状态为已提交
                    } else if child.is_tree() {
                        //当前事务的子事务是事务树，则执行子事务树的延迟提交
                        let child_uid = child.get_transaction_uid();
                        let commit_uid = child.get_commit_uid();
                        child.set_status(Transaction2PcStatus::Commiting); //更新子事务树状态为正在提交

                        if let Err(e) = mgr.commit_confirm(child.clone(), confirm.clone()).await {
                            //子事务延迟提交失败
                            child.set_status(Transaction2PcStatus::CommitFailed); //更新子事务树状态为提交失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit children failed, type: tree, child_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, commit_uid, e)));
                        }

                        child.set_status(Transaction2PcStatus::Commited); //更新子事务树状态为已提交
                    } else {
                        //当前事务的子事务是无效的事务
                        child.set_status(Transaction2PcStatus::CommitFailed); //更新子事务状态为提交失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit transaction failed, child_uid: {:?}, commit_uid: {:?}, reason: invalid transaction type", child.get_transaction_uid(), child.get_commit_uid())));
                    }
                }

                //所有子事务延迟提交已成功，则执行根事务延迟提交
                match tr.commit(confirm).await {
                    Err(e) => {
                        //根事务延迟提交失败
                        Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Commit root failed, type: tree, transaction_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_commit_uid(), e)))
                    },
                    Ok(output) => {
                        //根事务延迟提交成功
                        Ok(output)
                    },
                }
            }
        }.boxed()
    }

    /// 异步回滚，严重错误无法回滚
    pub async fn rollback<T>(&self, tr: T)
        -> Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>
        where T: TransactionTree<Node = T, Status = Transaction2PcStatus> {

        let current_tr_status = tr.get_status();
        if current_tr_status != Transaction2PcStatus::ActionFailed
            && current_tr_status != Transaction2PcStatus::PrepareFailed
            && current_tr_status != Transaction2PcStatus::LogCommitFailed {
            //事务不是操作失败，预提交失败和提交日志失败，则不允许回滚
            tr.set_status(Transaction2PcStatus::RollbackFailed); //更新事务状态为回滚失败
            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Rollback root failed, type: unit, transaction_uid: {:?}, commit_uid: {:?}, status: {:?}, reason: invalid transaction status", tr.get_transaction_uid(), tr.get_commit_uid(), current_tr_status)));
        }

        if !tr.is_writable() {
            //只读事务不需要回滚，则立即返回回滚成功
            tr.set_status(Transaction2PcStatus::Rollbacked); //更新事务状态为已回滚
            return Ok(<T as AsyncTransaction>::Output::default());
        }

        tr.set_status(Transaction2PcStatus::Rollbacking); //更新事务状态为正在回滚

        let result = if tr.is_unit() {
            //回滚单元事务
            tr.rollback().await
        } else {
            tr.rollback().await
            //回滚事务树
            // self.rollback_childrens(tr).await
        };

        if result.is_err() {
            //回滚事务失败
            tr.set_status(Transaction2PcStatus::RollbackFailed); //更新事务状态为回滚失败
        } else {
            //回滚事务成功
            tr.set_status(Transaction2PcStatus::Rollbacked); //更新事务状态为已回滚
        }

        result
    }

    // 子事务的回滚
    fn rollback_childrens<T>(&self, tr: T)
        -> BoxFuture<Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>>
        where T: TransactionTree<Node = T, Status = Transaction2PcStatus> {
        let mgr = self.clone();

        async move {
            if tr.is_unit() {
                //回滚单元事务
                return tr.rollback().await;
            }

            //延迟提交事务树
            if tr.is_concurrent_rollback() {
                //需要并发回滚，一般用于远端回滚
                let mut map_reduce = mgr.0.rt.map_reduce(tr.children_len());
                let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();

                for child in childs {
                    let child_uid = child.get_transaction_uid();
                    let prepare_uid = child.get_prepare_uid();
                    let commit_uid = child.get_commit_uid();

                    if child.is_unit() {
                        //当前事务的子事务是单元事务
                        let child_copy = child.clone();

                        if let Err(e) = map_reduce.map(AsyncRuntime::Multi(mgr.0.rt.clone()), async move {
                            //执行子单元事务的回滚
                            child_copy.set_status(Transaction2PcStatus::Rollbacking); //更新子单元事务状态为正在回滚

                            match child_copy.rollback().await {
                                Err(e) => {
                                    //子事务回滚失败
                                    child_copy.set_status(Transaction2PcStatus::RollbackFailed); //更新子单元事务状态为回滚失败
                                    Err(Error::new(ErrorKind::Other, format!("Rollback children failed, type: unit, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_copy.get_transaction_uid(), child_copy.get_prepare_uid(), child_copy.get_commit_uid(), e)))
                                },
                                Ok(output) => {
                                    //子事务回滚成功
                                    child_copy.set_status(Transaction2PcStatus::Rollbacked); //更新子单元事务状态为已回滚
                                    Ok(output)
                                },
                            }
                        }) {
                            //映射子事务的回滚操作失败
                            child.set_status(Transaction2PcStatus::RollbackFailed); //更新子单元事务状态为回滚失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Map children rollback failed, type: unit, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, prepare_uid, commit_uid, e)));
                        };
                    } else if child.is_tree() {
                        //当前事务的子事务是事务树
                        let mgr_copy = mgr.clone();

                        if let Err(e) = map_reduce.map(AsyncRuntime::Multi(mgr.0.rt.clone()), async move {
                            //执行子事务树的回滚
                            let child_uid = child.get_transaction_uid();
                            let prepare_uid = child.get_prepare_uid();
                            let commit_uid = child.get_commit_uid();

                            match mgr_copy.rollback(child).await {
                                Err(e) => {
                                    //子事务回滚失败
                                    Err(Error::new(ErrorKind::Other, format!("Rollback children failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, prepare_uid, commit_uid, e)))
                                },
                                Ok(output) => {
                                    //子事务回滚成功
                                    Ok(output)
                                },
                            }
                        }) {
                            //映射子事务的回滚操作失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Map children rollback failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, prepare_uid, commit_uid, e)));
                        };
                    } else {
                        //当前事务的子事务是无效的事务
                        child.set_status(Transaction2PcStatus::RollbackFailed); //更新子事务状态为回滚失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback transaction failed, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: invalid transaction type", child_uid, prepare_uid, commit_uid)));
                    }
                }

                //归并子事务的回滚
                match map_reduce.reduce(true).await {
                    Err(e) => {
                        //归并子事务的回滚操作失败
                        Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Reduce children rollback failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), tr.get_commit_uid(), e)))
                    },
                    Ok(results) => {
                        //归并子事务的回滚操作成功
                        for result in results {
                            if let Err(e) = result {
                                //子事务回滚失败
                                return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback children failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), tr.get_commit_uid(), e)));
                            }
                        }

                        //所有子事务回滚已成功，则执行根事务回滚
                        match tr.rollback().await {
                            Err(e) => {
                                //根事务回滚失败
                                Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback root failed, type: tree, transaction_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), tr.get_commit_uid(), e)))
                            },
                            Ok(output) => {
                                //根事务回滚成功
                                Ok(output)
                            },
                        }
                    }
                }
            } else {
                //不需要并发回滚，一般用于本地回滚
                let childs: Vec<<T as TransactionTree>::Node> = tr.to_children().collect();

                for child in childs {
                    if child.is_unit() {
                        //当前事务的子事务是单元事务，则执行子单元事务的回滚
                        child.set_status(Transaction2PcStatus::Rollbacking); //更新子单元事务状态为正在回滚

                        if let Err(e) = child.rollback().await {
                            //子事务回滚失败
                            child.set_status(Transaction2PcStatus::RollbackFailed); //更新子单元事务状态为回滚失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback children failed, type: unit, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", child.get_transaction_uid(), child.get_prepare_uid(), child.get_commit_uid(), e)));
                        }

                        child.set_status(Transaction2PcStatus::Rollbacked); //更新子单元事务状态为已回滚
                    } else if child.is_tree() {
                        //当前事务的子事务是事务树，则执行子事务树的回滚
                        let child_uid = child.get_transaction_uid();
                        let prepare_uid = child.get_prepare_uid();
                        let commit_uid = child.get_commit_uid();

                        if let Err(e) = mgr.rollback(child).await {
                            //子事务回滚失败
                            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback children failed, type: tree, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", child_uid, prepare_uid, commit_uid, e)));
                        }
                    } else {
                        //当前事务的子事务是无效的事务
                        child.set_status(Transaction2PcStatus::CommitFailed); //更新子事务状态为回滚失败
                        return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback transaction failed, child_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: invalid transaction type", child.get_transaction_uid(), child.get_prepare_uid(), child.get_commit_uid())));
                    }
                }

                //所有子事务回滚已成功，则执行根事务回滚
                match tr.rollback().await {
                    Err(e) => {
                        //根事务回滚失败
                        Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Fatal, format!("Rollback root failed, type: tree, transaction_uid: {:?}, prepare_uid: {:?}, commit_uid: {:?}, reason: {:?}", tr.get_transaction_uid(), tr.get_prepare_uid(), tr.get_commit_uid(), e)))
                    },
                    Ok(output) => {
                        //根事务回滚成功
                        Ok(output)
                    },
                }
            }
        }.boxed()
    }

    /// 重播提交日志，以恢复因异常关闭导致的未确认的提交日志
    pub async fn replay_commit_log<B>(&self,
                                      callback: impl FnMut(Guid, B) -> IOResult<()> + Send + 'static)
                                      -> IOResult<(usize, usize)>
        where B: BufMut + AsRef<[u8]> + From<Vec<u8>> + Send + Sized + 'static {
        self.0.commit_logger.replay(callback).await
    }

    /// 重播提交，提交重播未确认的提交日志的事务
    /// 为事务设置指定的事务唯一id和提交唯一id
    pub async fn replay_commit<T>(&self,
                                  tr: T,
                                  transaction_uid: <T as Transaction2Pc>::Tid,
                                  commit_uid: <T as Transaction2Pc>::Cid,
                                  input: <T as Transaction2Pc>::PrepareOutput,
                                  confirm: <T as Transaction2Pc>::CommitConfirm)
        -> Result<<T as AsyncTransaction>::Output, <T as AsyncTransaction>::Error>
        where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
        //初始化重播事务
        reset_transaction_uid::<C, Log, T>(tr.clone(), transaction_uid); //递归重置重播事务的事务唯一id
        if let Err(current_len) = register_transcation(&self, &tr) {
            //注册根事务失败，指定事务所在事务源的同时处理事务数已达限制
            tr.set_status(Transaction2PcStatus::InitFailed); //更新事务状态为初始化失败
            return Err(<T as AsyncTransaction>::Error::new_transaction_error(ErrorLevel::Normal, format!("Init root failed, type: unit, current: {}, status: {:?}, reason: same source transaction excessive", current_len, tr.get_status())));
        }

        //因为是重播的是未确认且已提交的提交日志的事务，则忽略重播事务的预提交过程
        reset_commit_uid::<C, Log, T>(tr.clone(), commit_uid); //递归重置重播事务的提交唯一id
        tr.set_status(Transaction2PcStatus::Prepared); //更新事务状态为已预提交

        //提交重播事务
        self.commit(tr, input, confirm).await
    }
}

// 分配一个唯一的事务id
#[inline(always)]
fn alloc_transaction_uid<T>(uid_gen: &GuidGen, tr: &T) -> Guid
    where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
    if tr.is_enable_inherit_uid() {
        //允许子事务继承指定事务的事务唯一id
        if let Some(uid) = tr.get_transaction_uid() {
            //指定事务当前已分配事务唯一id，则立即返回
            return uid;
        }
    }

    //分配唯一的事务id
    uid_gen.gen(DEFAULT_TRANSACTION_CTRL_ID)
}

// 分配一个唯一的提交id
#[inline(always)]
fn alloc_commit_uid<T>(uid_gen: &GuidGen, tr: &T) -> Guid
    where T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus> {
    if tr.is_enable_inherit_uid() {
        //允许子事务继承指定事务的提交唯一id
        if let Some(uid) = tr.get_commit_uid() {
            //指定事务当前已分配提交唯一id，则立即返回
            return uid;
        }
    }

    //分配唯一的提交id
    uid_gen.gen(DEFAULT_TRANSACTION_COMMIT_CTRL_ID)
}

//注册指定事务到事务表中，注册失败则返回指定事务源的当前事务数量
fn register_transcation<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
    T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus>,
>(mgr: &Transaction2PcManager<C, Log>, tr: &T) -> Result<(), usize> {
    let source = tr.get_source();
    if let (Some(current_len), Some(limit)) = (mgr.source_len(&source), mgr.get_max_source_parallel_limit(&source)) {
        if current_len >= limit {
            //如果指定事务源的当前事务数量达到限制，则不允许注册
            return Err(current_len)
        }
    }

    if let Some(transaction_uid) = tr.get_transaction_uid() {
        //指定事务已分配事务唯一id，则注册
        let shared_tr = Arc::new(tr.clone()) as Arc<dyn Any + Send + Sync + 'static>;
        mgr.0.trans_table.insert(transaction_uid.clone(), shared_tr.clone()); //注册根事务
        if let Some(counter) = mgr.0.source_counter.get(&tr.get_source()) {
            //指定事务源已存在，则增加计数
            counter.value().0.fetch_add(1, Ordering::Relaxed);
        } else {
            //指定事务源不存在，则创建事务源，并增加计数
            mgr.0.source_counter.insert(tr.get_source(),
                                        (AtomicUsize::new(1), AtomicUsize::new(0), AtomicUsize::new(DEFAULT_MAX_PARALLEL_TRANSACTION_LIMIT)));
        }
        mgr.0.produced_total.fetch_add(1, Ordering::Relaxed); //增加开始事务的总数量
    }

    Ok(())
}

//递归设置事务的事务唯一id
fn reset_transaction_uid<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
    T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus>,
>(tr: T,
  transaction_uid: <T as Transaction2Pc>::Tid) {
    tr.set_transaction_uid(transaction_uid.clone()); //重置重播事务的事务唯一id

    if tr.children_len() > 0 {
        //当前事务有子事务，则递归的设置子事务的事务唯一id
        let mut childs = tr.to_children();

        while let Some(child) = childs.next() {
            reset_transaction_uid::<C, Log, T>(child, transaction_uid.clone());
        }
    }
}

//递归设置事务的提交唯一id
fn reset_commit_uid<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
    T: TransactionTree<Tid = Guid, Pid = Guid, Cid = Guid, Node = T, Status = Transaction2PcStatus>,
>(tr: T,
  commit_uid: <T as Transaction2Pc>::Cid) {
    tr.set_commit_uid(commit_uid.clone()); //重置重播事务的事务唯一id

    if tr.children_len() > 0 {
        //当前事务有子事务，则递归的设置子事务的事务唯一id
        let mut childs = tr.to_children();

        while let Some(child) = childs.next() {
            reset_commit_uid::<C, Log, T>(child, commit_uid.clone());
        }
    }
}

// 内部事务两阶段提交管理器
struct Inner2PcManager<
    C: Send + 'static,
    Log: AsyncCommitLog<C = C, Cid = Guid>,
> {
    rt:                 MultiTaskRuntime<()>,                                   //异步运行时
    uid_gen:            GuidGen,                                                //事务唯一id生成器
    commit_logger:      Log,                                                    //提交日志记录器
    trans_table:        DashMap<Guid, Arc<dyn Any + Send + Sync + 'static>>,    //事务表
    source_counter:     DashMap<Atom, (AtomicUsize, AtomicUsize, AtomicUsize)>, //事务源计数器
    prepare_produced:   AtomicUsize,                                            //事务开始预提交计数器
    prepare_consumed:   AtomicUsize,                                            //事务结束预提交计数器
    commit_produced:    AtomicUsize,                                            //事务开始提交计数器
    commit_consumed:    AtomicUsize,                                            //事务结束提交计数器
    produced_total:     AtomicUsize,                                            //开始事务的总数计数器
    consumed_total:     AtomicUsize,                                            //结束事务的总数计数器
}


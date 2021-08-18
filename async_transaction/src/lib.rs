#![allow(dead_code)]

///
/// 抽象了事务的执行，一个事务允许由其它事务通过序列或树方式组合而成，事务组合和事务初始化的过程可能完全不同，所以必须是具象的
/// 但事务的执行过程在事务组合和事务初始化后，是可以统一的，所以可以抽象事务的执行过程
///
use std::fmt::Debug;
use std::io::Result as IOResult;

use futures::future::BoxFuture;
use bytes::BufMut;

use atom::Atom;

pub mod manager_2pc;

///
/// 事务错误级别
///
#[derive(Debug, Clone)]
pub enum ErrorLevel {
    Normal, //一般事务错误，可以回滚或重试
    Fatal,  //严重事务错误，无法回滚或重试
}

///
/// 抽象的事务错误
///
pub trait TransactionError: Debug + Sized + 'static {
    /// 构建一个事务错误
    fn new_transaction_error<E>(level: ErrorLevel, reason: E) -> Self
        where E: Debug + Sized + 'static;
}

///
/// 抽象的事务日志
///
pub trait TransactionLog: BufMut + AsRef<[u8]> + Send + Sized + 'static {}

///
/// 抽象的异步事务
///
pub trait AsyncTransaction: Send + Sync + 'static {
    type Output: Default + Send + 'static;
    type Error: TransactionError;

    /// 是否是可写事务
    fn is_writable(&self) -> bool;

    /// 是否并发提交
    fn is_concurrent_commit(&self) -> bool;

    /// 是否并发回滚
    fn is_concurrent_rollback(&self) -> bool;

    /// 获取事务源
    fn get_source(&self) -> Atom;

    /// 异步初始化事务
    fn init(&self) -> BoxFuture<Result<<Self as AsyncTransaction>::Output, <Self as AsyncTransaction>::Error>>;

    /// 异步回滚事务
    fn rollback(&self) -> BoxFuture<Result<<Self as AsyncTransaction>::Output, <Self as AsyncTransaction>::Error>>;
}

///
/// 抽象的2阶段事务，由实现提供一致性和隔离性和保证
///
pub trait Transaction2Pc: AsyncTransaction + Clone {
    type Tid: Debug + Clone + Send + PartialEq + Eq + 'static;
    type Pid: Debug + Clone + Send + PartialEq + Eq + 'static;
    type Cid: Debug + Clone + Send + PartialEq + Eq + 'static;
    type PrepareOutput: BufMut + AsRef<[u8]> + Send + Sized + 'static;
    type PrepareError: TransactionError;
    type ConfirmOutput: Send + 'static;
    type ConfirmError: TransactionError;
    type CommitConfirm: Fn(
        <Self as Transaction2Pc>::Tid,
        <Self as Transaction2Pc>::Cid,
        Result<<Self as Transaction2Pc>::ConfirmOutput, <Self as Transaction2Pc>::ConfirmError>
    ) -> Result<(), <Self as Transaction2Pc>::ConfirmError> + Clone + Send + Sync + 'static;

    /// 是否需要持久化
    fn is_require_persistence(&self) -> bool;

    /// 是否并发预提交
    fn is_concurrent_prepare(&self) -> bool;

    /// 是否允许子事务继承指定事务的指定唯一id
    fn is_enable_inherit_uid(&self) -> bool;

    /// 获取事务唯一id
    fn get_transaction_uid(&self) -> Option<<Self as Transaction2Pc>::Tid>;

    /// 设置事务唯一id
    fn set_transaction_uid(&self, uid: <Self as Transaction2Pc>::Tid);

    /// 获取事务预提交唯一id
    fn get_prepare_uid(&self) -> Option<<Self as Transaction2Pc>::Pid>;

    /// 设置事务预提交唯一id
    fn set_prepare_uid(&self, uid: <Self as Transaction2Pc>::Pid);

    /// 获取事务提交唯一id
    fn get_commit_uid(&self) -> Option<<Self as Transaction2Pc>::Cid>;

    /// 设置事务提交唯一id
    fn set_commit_uid(&self, uid: <Self as Transaction2Pc>::Cid);

    /// 获取事务预提交超时时间，单位毫秒
    fn get_prepare_timeout(&self) -> u64;

    /// 获取事务提交超时时间，单位毫秒
    fn get_commit_timeout(&self) -> u64;

    /// 异步预提交，只读事务预提交成功不会返回输出数据，可写事务预提交成功会返回输出数据
    fn prepare(&self) -> BoxFuture<Result<Option<<Self as Transaction2Pc>::PrepareOutput>, <Self as Transaction2Pc>::PrepareError>>;

    /// 异步延迟提交
    fn commit(&self, confirm: <Self as Transaction2Pc>::CommitConfirm)
        -> BoxFuture<Result<<Self as AsyncTransaction>::Output, <Self as AsyncTransaction>::Error>>;
}

///
/// 抽象的异步单元事务，描述了事务的上下文、状态和服务质量
/// 单元事务表示一个独立的最小事务单位，单元事务执行成功，表示这个独立的事务执行成功
///
pub trait UnitTransaction: Transaction2Pc {
    type Status: Debug + Clone + PartialEq + Eq + 'static;
    type Qos: Debug + Default + Clone + PartialEq + Eq + 'static;

    /// 是否是单元事务
    fn is_unit(&self) -> bool;

    /// 获取事务状态
    fn get_status(&self) -> <Self as UnitTransaction>::Status;

    /// 设置事务状态
    fn set_status(&self, status: <Self as UnitTransaction>::Status);

    /// 获取事务服务质量
    fn qos(&self) -> <Self as UnitTransaction>::Qos;
}

///
/// 抽象的异步顺序事务，描述了一系列事务执行的先后顺序
/// 顺序事务表示一系列有顺序的事务中的前后关系，这一系列事务必须按照从前往后或从后往前的顺序全部执行成功，才表示这一系列事务执行成功
///
pub trait SequenceTransaction: UnitTransaction {
    type Item: Transaction2Pc;

    /// 是否是顺序事务
    fn is_sequence(&self) -> bool;

    /// 获取前一个事务
    fn prev_item(&self) -> Option<<Self as SequenceTransaction>::Item>;

    /// 获取后一个事务
    fn next_item(&self) -> Option<<Self as SequenceTransaction>::Item>;
}

///
/// 抽象的异步事务树，描述了事务的组合依赖和并发关系
/// 一个事务树，描述了一棵独立的事务树，它包括一个根节点和可能存在的多个子节点
/// 一个事务树，可以是另一棵事务树的子树，或是序列中的一个顺序事务
/// 一个事务树，可以有子节点，如果有子节点，描述了这个事务由子节点事务组合而成，或者说它依赖了子节点数量的其它事务，
/// 一个事务树，可以有子节点，子节点如果是另一个事务树，则另一个事务树是当前事务树的子树
/// 一个事务树，可以有子节点，子节点如果是一个单元事务或顺序事务，则这个子节点是叶节点
/// 一个事务树，所有的子节点事务之间没有依赖关系，可以并发执行
/// 一个事务树，在初始化时，是从根节点递推到子节点，执行时是从子节点回归到叶节点
/// 一个事务树，必须等待所有的子节点事务，并发的递归执行完成后，才可以认为一个事务树执行完成
/// 一个事务树，在所有子节点递归执行成功后，则可以认为一个状态事务执行成功
/// 一个事务树，在任意子节点的事务执行失败后，则可以认为一个事务树执行失败
///
pub trait TransactionTree: UnitTransaction {
    type Node: TransactionTree + SequenceTransaction;
    type NodeInterator: Iterator<Item = <Self as TransactionTree>::Node> + 'static;

    /// 是否是事务树
    fn is_tree(&self) -> bool;

    /// 获取所有子节点的数量
    fn children_len(&self) -> usize;

    /// 获取所有子节点的复制
    fn to_children(&self) -> Self::NodeInterator;
}

///
/// 抽象的异步提交日志
///
pub trait AsyncCommitLog: Clone + Send + Sync + 'static {
    type C: Clone + Send + 'static;
    type Cid: Debug + Clone + Send + PartialEq + Eq + 'static;

    /// 异步追加提交日志
    fn append<B>(&self, commit_uid: Self::Cid, log: B) -> BoxFuture<IOResult<Self::C>>
        where B: BufMut + AsRef<[u8]> + Send + Sized + 'static;

    /// 异步刷新提交日志
    fn flush(&self, log_handle: Self::C) -> BoxFuture<IOResult<()>>;

    /// 异步确认提交日志
    fn confirm(&self, commit_uid: Self::Cid) -> BoxFuture<IOResult<()>>;

    /// 重播提交日志
    fn replay<B, F>(&self, callback: F) -> BoxFuture<IOResult<(usize, usize)>>
        where B: BufMut + AsRef<[u8]> + From<Vec<u8>> + Send + Sized + 'static,
              F: FnMut(Self::Cid, B) -> IOResult<()> + Send + 'static;
}








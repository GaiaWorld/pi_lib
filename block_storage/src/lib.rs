use std::io::{Read, Write, Result as IOResult};

use futures::future::{FutureExt, BoxFuture};

///
/// 可访问的二进制块
///
pub trait BinaryBlock: Drop + Send + 'static {
    /// 获取块的容量
    fn capacity(&self) -> usize;

    /// 获取块的当前长度
    fn len(&self) -> usize;

    /// 从块的指定位置开始，读指定长度的数据，成功返回读取的数据
    fn read(&self, pos: usize, len: usize) -> BoxFuture<IOResult<&[u8]>>;

    /// 从块的指定位置开始，写入指定的数据，成功返回写入数据的长度
    fn write<'a, Buf>(&mut self, pos: usize, buf: Buf) -> BoxFuture<IOResult<usize>>
        where Buf: AsRef<[u8]> + 'a;

    /// 异步提交块的数据
    fn commit(&self) -> BoxFuture<IOResult<()>>;
}

///
/// 二进制块分配器
///
pub trait BinaryBlockAllocator: Clone + Send + Sync + 'static {
    type Block: BinaryBlock;    //已分配的块

    /// 异步分配指定容量的块，返回空表示无法分配指定容量的块
    fn alloc(&self, capacity: usize) -> BoxFuture<Option<Self::Block>>;

    /// 异步解除指定的已分配的块，返回指定的已分配的块表示无法解除指定的已分配的块
    fn dealloc(&self, block: Self::Block) -> BoxFuture<Result<(), Self::Block>>;

    /// 异步重新分配指定容量的块，并将指定的已分配的块内容移动到新的分配的块，然后解除旧的分配的块，成功返回重新分配的块，失败返回指定的已分配的块
    fn realloc(&self, block: Self::Block, new_capacity: usize) -> BoxFuture<Result<Self::Block, Self::Block>>;
}
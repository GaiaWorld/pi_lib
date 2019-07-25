/// 版本索引， 给slab vecmap 使用
use std::fmt::Debug;

pub mod bit;
pub mod ver;

pub trait VerIndex {
    type ID: Copy + Debug + PartialEq + Default + Send + Sync;
    // 将参数id 分解成 version 和 index
    fn split(&self, id: Self::ID) -> (usize, usize);
    // 将参数 version 和 index, 合成成id
    fn merge(&self, version: usize, index: usize) -> Self::ID;

    fn capacity(&self) -> usize;

    fn reserve(&mut self, additional: usize);

    fn shrink_to_fit(&mut self);

    fn clear(&mut self);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
    // 从开始位置向后查找，返回version 和 index
    fn first_true(&self) -> (usize, usize);
    // 从指定的位置向后查找，返回version 和 index
    fn next_true(&self, index: usize) -> (usize, usize);

    // 从结束位置向前查找，返回version 和 index
    fn last_true(&self) -> (usize, usize);
    // 从指定的位置向前查找，返回version 和 index
    fn prev_true(&self, index: usize) -> (usize, usize);

    fn set_true(&mut self, index: usize)-> usize;

    fn set_false(&mut self, index: usize, v: usize)-> bool;

    fn version(&self, index: usize) -> usize;
}
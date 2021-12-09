//! 该库用于对树形结构数据标记脏
//! 和普通的脏编辑不同，LayerDirty将同一层的节点索引放入同一个Vec中，并将这些Vec按照层的大小，由小到大放置在一个更外层的Vec中。
//! 例如[[1, 2, 3], [], [4, 5, 6], [7, 8]]，表示第0层中的节点1、节点2、节点3被标记为脏，
//! 第1层没有脏节点，第2层中节点4、节点4、节点6被标记为脏，第3层中节点7、节点8被标记为脏。
//! LayerDirty特别适合对树形结构中，子节点的数据计算依赖于父节点的数据计算。
//!
//! 一个常用的例子是：
//! ui中的节点被组织成树形结构，每个节点都存在transfrom属性，可以对当前节点进行变换（是将本节点和其子节点作为整体进行变化）
//! 根据公式，一个节点的最终变换，实际上是父变换*子变换, 如果有更多的父节，则为...祖*父*子。
//! 因此， 以三层节点为例，先从祖节点开始计算祖节点的最终变换（祖最终变换 = 祖变换），
//! 根据结果计算父节点的最终变换（父最终变换 = 祖最终变换* 父变化），
//! 再根据父的计算结果计算子的最终变换（子最终变换 = 父最终变换* 子变化）.
//! 如果我们将计算顺序反过来，势必会造成多重重复计算： 
//!    子最终变换 = 子变化；
//!    父最终变换 = 父变换；
//!    子最终变换= 父最终变换 * 子变化（前一步计算的子最终变换由于缺少父的影响，并不是真正的结果，需要重新计算）
//!       ...
//! 可以看到，同一个节点可能被反复计算多次
//!
//! LayerDirty就是为了解决这种父子存在计算顺序的问题。
//! 利用LayerDirty，你可以非常轻松的先迭代父脏，再迭代子脏（其中的脏对每一层进行了划分，并按照层从小到大的顺序排列，
//! 我们只需要从最外层数组的第一个开始迭代就可以了）
extern  crate log;
use std::slice::Iter;

/// # Examples
/// ```
///    use dirty::LayerDirty;
///    let mut dirtys = LayerDirty::default();
///
///    dirtys.mark(7, 3);// 将第3层的节点7标记为脏
///    dirtys.mark(8, 3);// 将第3层的节点8标记为脏
///
///    dirtys.mark(4, 2);// 将第2层的节点4标记为脏
///    dirtys.mark(5, 2);// 将第2层的节点5标记为脏
///    dirtys.mark(6, 2);// 将第6层的节点4标记为脏
///
///    dirtys.mark(1, 0);// 将第0层的节点1标记为脏
///    dirtys.mark(2, 0);// 将第0层的节点2标记为脏
///    dirtys.mark(3, 0);// 将第0层的节点3标记为脏
///
///    let mut iter = dirtys.iter();
///
///    // 迭代时，会从第0层开始迭代
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 1);
///    assert_eq!(di.1, 0);
///
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 2);
///    assert_eq!(di.1, 0);
///
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 3);
///    assert_eq!(di.1, 0);
///
///    // 由于第一层没有脏，继续迭代第2层元素
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 4);
///    assert_eq!(di.1, 2);
///
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 5);
///    assert_eq!(di.1, 2);
///
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 6);
///    assert_eq!(di.1, 2);
///
///    // 最后迭代第三层元素
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 7);
///    assert_eq!(di.1, 3);
///
///    let di = iter.next().unwrap();
///    assert_eq!(*(di.0), 8);
///    assert_eq!(di.1, 3);
/// ```
#[derive(Debug)]
pub struct LayerDirty<T> {
    dirtys: Vec<Vec<T>>, // 按层放置的脏节点
    count: usize,            // 脏节点数量
    start: usize,            // 脏节点的起始层
	end: usize,            // 脏节点的结束层
}

impl<T: Eq> Default for LayerDirty<T> {
    fn default() -> LayerDirty<T> {
        LayerDirty {
            dirtys: vec![Vec::new()],
            count: 0,
            start: 0,
			end: 0,
        }
    }
}

impl<T: Eq> LayerDirty<T> {
    /// 脏数量
    pub fn count(&self) -> usize {
        self.count
    }
    /// 标记脏，标记对象总是一个数字（id），如果标记对象不是一个数字，你应该使用其它数据结构将其映射为一个数字，才能使用LayerDirty
    ///
    /// # Examples
    /// 
    /// ```
    /// use dirty::LayerDirty;
    /// let mut dirtys = LayerDirty::default();
    /// dirtys.mark(7, 3);// 将第3层的节点7标记为脏
    /// ```
    pub fn mark(&mut self, id: T, layer: usize) {
        self.count += 1;
        if self.start > layer {
            self.start = layer;
        }
		if self.end <= layer {
			self.end = layer + 1;
		}
        if self.dirtys.len() <= layer {
            for _ in self.dirtys.len()..layer + 1 {
                self.dirtys.push(Vec::new())
            }
        }
        let vec = unsafe { self.dirtys.get_unchecked_mut(layer) };
        vec.push(id);
    }

    /// 删除脏标记
    ///
    /// # Examples
    ///
    /// ```
    /// use dirty::LayerDirty;
    /// let mut dirtys = LayerDirty::default();
    /// dirtys.mark(7, 3);// 将第3层的节点7标记为脏
    /// dirtys.delete(7, 3);// 将第3层的节点7标记为脏
    /// assert_eq!(dirtys.count(), 0);
    /// ```
    pub fn delete(&mut self, id: T, layer: usize) {
        let vec = unsafe { self.dirtys.get_unchecked_mut(layer) };
        for i in 0..vec.len() {
            if vec[i] == id {
                vec.swap_remove(i);
                self.count -= 1;
                break;
            }
        }
    }

    /// 取到LayerDirty的迭代器
    pub fn iter(&self) -> DirtyIterator<T> {
        if self.count == 0 {
            DirtyIterator {
                inner: self,
                layer: self.start,
                iter: self.dirtys[0].iter(),
            }
        } else {
             DirtyIterator {
                inner: self,
                layer: self.start + 1,
                iter: self.dirtys[self.start].iter(),
            }
        }
    }

	 /// 取到LayerDirty的迭代器
	 pub fn iter_reverse(&self) -> ReverseDirtyIterator<T> {
        if self.count == 0 {
            ReverseDirtyIterator {
                inner: self,
                layer: self.start,
                iter: self.dirtys[0].iter(),
            }
        } else {
			ReverseDirtyIterator {
                inner: self,
                layer: self.end - 1,
                iter: self.dirtys[self.end - 1].iter(),
            }
        }
    }

    /// 清空脏标记
    pub fn clear(&mut self) {
        let len = self.dirtys.len();
        while self.start < len {
            let vec = unsafe { self.dirtys.get_unchecked_mut(self.start) };
            let c = vec.len();
            self.start += 1;
            if c == 0 {
                continue;
            }
            self.count -= c;
            vec.clear();
        }
		self.end = 0;
    }
}


/// 迭代器
pub struct DirtyIterator<'a, T> {
    inner: &'a LayerDirty<T>,
    layer: usize,
    iter: Iter<'a, T>,
}

// 迭代器实现
impl<'a, T: Eq> Iterator for DirtyIterator<'a, T> {
    type Item = (&'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut r = self.iter.next();
        if r == None {
            let len = self.inner.dirtys.len();
            while self.layer < len {
                let vec = unsafe { self.inner.dirtys.get_unchecked(self.layer) };
                self.layer += 1;
                if vec.len() > 0 {
                    self.iter = vec.iter();
                    r = self.iter.next();
                    break;
                }
            }
			
        }
		match r {
			Some(r) => Some((r, self.layer - 1)),
			None => None,
		}
    }
}


/// 迭代器
pub struct ReverseDirtyIterator<'a, T> {
    inner: &'a LayerDirty<T>,
    layer: usize,
    iter: Iter<'a, T>,
}

/// 迭代器实现
impl<'a, T: Eq> Iterator for ReverseDirtyIterator<'a, T> {
    type Item = (&'a T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let mut r = self.iter.next();
        if r == None {
            while self.layer > 0 {
                let vec = unsafe { self.inner.dirtys.get_unchecked(self.layer - 1) };
                self.layer -= 1;
                if vec.len() > 0 {
                    self.iter = vec.iter();
                    r = self.iter.next();
                    break;
                }
            }
        }
		match r {
			Some(r) => Some((r, self.layer)),
			None => None,
		}
    }
}
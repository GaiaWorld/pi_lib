/// 基于位图的版本索引

use std::mem::{size_of};

use ::VerIndex;

#[derive(Default, Debug)]
pub struct BitIndex(Vec<usize>);

impl BitIndex {

    pub fn with_capacity(capacity: usize) -> Self {
        let i = if capacity%usize_size() == 0 { 0 }else{ 1 };
        BitIndex(Vec::with_capacity(capacity / usize_size() + i))
    }
}

impl VerIndex for BitIndex {
    type ID = usize;
    // 将参数id 分解成 version 和 index
    #[inline(always)]
    fn split(&self, id: Self::ID) -> (usize, usize) {
        (1, id)
    }
    // 将参数 version 和 index, 合成成id
    #[inline(always)]
    fn merge(&self, _version: usize, index: usize) -> Self::ID {
        index
    }
    fn capacity(&self) -> usize {
        self.0.capacity() * usize_size()
    }

    fn reserve(&mut self, additional: usize) {
        let i = if additional%usize_size() == 0 { 0 }else{ 1 };
        self.0.reserve(additional/usize_size() + i);
    }

    fn shrink_to_fit(&mut self) {
        match self.last_true() {
            (0, _) => self.0.clear(),
            (_, i) => self.0.truncate(i/usize_size() + 1)
        };
        self.0.shrink_to_fit();
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
    // 从开始位置向后查找，返回version 和 index
    fn first_true(&self) -> (usize, usize) {
        for i in 0..self.0.len() {
            let value = *unsafe{self.0.get_unchecked(i)};
            if value != 0 {
                return (1, i * usize_size() + value.trailing_zeros() as usize)
            }
        }
        (0, 0)
    }
    // 从指定的位置向后查找，返回version 和 index
    fn next_true(&self, mut index: usize) -> (usize, usize) {
        index += 1;
        let sign_index = index as usize/usize_size();
        let mut shift = index as usize%usize_size();
        for i in sign_index..self.0.len() {
            let value = *unsafe{self.0.get_unchecked(i)} >> shift;
            if value != 0 {
                return (1, index + value.trailing_zeros() as usize)
            }else if shift == 0 {
                index += usize_size();
            }else {
                index += usize_size() - shift;
                shift = 0;
            }
        }
        (0, 0)
    }
    // 从结束位置向前查找，返回version 和 index
    fn last_true(&self) -> (usize, usize) {
        let len = self.0.len();
        for i in 0..len {
            let value = *unsafe{self.0.get_unchecked(len - i - 1)};
            if value != 0 {
                return (1, (len - i) * usize_size() - value.leading_zeros() as usize - 1)
            }
        }
        (0, 0)
    }
    // 从指定的位置向前查找，返回version 和 index
    fn prev_true(&self, mut index: usize) -> (usize, usize) {
        if index == 0 {
            return (0, 0)
        }
        index -= 1;
        let sign_index = index as usize/usize_size();
        let mut shift = index as usize%usize_size();
        for i in 0..sign_index + 1 {
            let value = *unsafe{self.0.get_unchecked(sign_index - i)} << usize_size() - 1 - shift;
            if value != 0 {
                return (1, index - value.leading_zeros() as usize)
            }else if index == shift {
                return (0, 0)
            }else if shift == usize_size() - 1 {
                index -= usize_size();
            }else {
                index -= shift + 1;
                shift = usize_size() - 1;
            }
        }
        (0, 0)
    }

    #[inline(always)]
    fn set_true(&mut self, index: usize) -> usize {
        let i = index/usize_size();
        if i >= self.0.len() {
            self.0.resize(i + 1, 0);
        }
        set_true(unsafe {self.0.get_unchecked_mut(i)}, index%usize_size());
        1
    }

    #[inline(always)]
    fn set_false(&mut self, index: usize, version: usize) -> bool {
        let i = index/usize_size();
        if i < self.0.len() {
            set_false(unsafe {self.0.get_unchecked_mut(i)}, index%usize_size());
        }
        true
    }

    #[inline(always)]
    fn version(&self, index: usize) -> usize {
        let i = index/usize_size();
        if i < self.0.len() {
            let i = unsafe {self.0.get_unchecked(i)};
            is_true(i, index%usize_size())
        }else{
            0
        }
    }
}

impl Clone for BitIndex {
    fn clone(&self) -> Self {
        BitIndex(self.0.to_vec())
    }
}

#[derive(Debug)]
pub struct BitIndexIter<'a> {
    sign: &'a Vec<usize>,
    cur_index: usize,
    cur_len: usize,
    len: usize,
}


#[inline(always)]
fn is_true(i: &usize, index: usize) -> usize {
    (i >> index) & 1
}

#[inline(always)]
fn set_true(i: &mut usize, index: usize){
    (*i) = *i | (1 << index);
}

#[inline(always)]
fn set_false(i: &mut usize, index: usize){
    (*i) = *i - (1 << index);
}

#[inline(always)]
fn usize_size() -> usize{
    size_of::<usize>() * 8
}


#[test]
fn test(){
    let mut slab: BitIndex = BitIndex::default();
    for i in 0..72{
        slab.set_true(i);
        println!("slab------{:?}", slab);
    }

    slab.set_false(30, 1);
    println!("r 30------{:?}", slab);

    slab.set_false(31, 1);
    println!("r 31------{:?}", slab);

    slab.set_false(69, 1);
    println!("r 69------{:?}", slab);

    slab.set_false(70, 1);
    println!("r 70------{:?}", slab);


    slab.set_true(70);
    println!("i 70------{:?}", slab);

    assert_eq!(slab.version(0), 1);
    assert_eq!(slab.version(1), 1);
    assert_eq!(slab.version(71), 1);
    assert_eq!(slab.version(72), 0);
    let mut cur = slab.last_true();
    while cur.0 > 0 {
        println!("vvv------{:?}", cur.1);
        cur = slab.prev_true(cur.1);
    }

}

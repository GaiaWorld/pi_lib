/// u32版本索引

use std::mem::{size_of};

use ::VerIndex;

#[derive(Default, Debug)]
pub struct U32Index(Vec<i32>);

impl U32Index {

    pub fn with_capacity(capacity: usize) -> Self {
        U32Index(Vec::with_capacity(capacity))
    }
}

impl VerIndex for U32Index {
    type ID = u64;
    // 将参数id 分解成 version 和 index
    #[inline(always)]
    fn split(&self, id: Self::ID) -> (usize, usize) {
        ((id >> 32) as usize, id as u32 as usize)
    }
    // 将参数 version 和 index, 合成成id
    #[inline(always)]
    fn merge(&self, version: usize, index: usize) -> Self::ID {
        (version as u64) << 32 | index as u64
    }
    fn capacity(&self) -> usize {
        self.0.capacity()
    }

    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    fn shrink_to_fit(&mut self) {
        match self.last_true() {
            (0, _) => self.0.clear(),
            (_, i) => self.0.truncate(i + 1)
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
            let value = unsafe{*self.0.get_unchecked(i)} as usize;
            if value > 0 {
                return (value, i)
            }
        }
        (0, 0)
    }
    // 从指定的位置向后查找，返回version 和 index
    fn next_true(&self, mut index: usize) -> (usize, usize) {
        index += 1;
        for i in index..self.0.len() {
            let value = unsafe{*self.0.get_unchecked(i)} as usize;
            if value > 0 {
                return (value, i)
            }
        }
        (0, 0)
    }
    // 从结束位置向前查找，返回version 和 index
    fn last_true(&self) -> (usize, usize) {
        let len = self.0.len();
        for i in 1..len + 1 {
            let value = unsafe{*self.0.get_unchecked(len - i)} as usize;
            if value != 0 {
                return (value, len - i)
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
        for i in 0..index + 1 {
            let value = unsafe{*self.0.get_unchecked(index - i)} as usize;
            if value != 0 {
                return (value, index - i)
            }
        }
        (0, 0)
    }

    #[inline(always)]
    fn set_true(&mut self, index: usize) -> usize {
        if index >= self.0.len() {
            self.0.resize(index + 1, 0);
        }
        let i = unsafe {self.0.get_unchecked_mut(index)};
        *i = -*i + 1;
        *i as usize
    }

    #[inline(always)]
    fn set_false(&mut self, index: usize, version: usize) -> bool {
        if index < self.0.len() {
            let i = unsafe {self.0.get_unchecked_mut(index)};
            if *i as usize != version {
                return false
            }
            *i = -*i;
        }
        true
    }

    #[inline(always)]
    fn version(&self, index: usize) -> usize {
        if index < self.0.len() {
            let i = unsafe {*self.0.get_unchecked(index)};
            if i > 0 {i as usize}else{0}
        }else{
            0
        }
    }
}

impl Clone for U32Index {
    fn clone(&self) -> Self {
        U32Index(self.0.to_vec())
    }
}

#[derive(Debug)]
pub struct U32IndexIter<'a> {
    sign: &'a Vec<usize>,
    cur_index: usize,
    cur_len: usize,
    len: usize,
}


#[test]
fn test(){
    let mut slab: U32Index = U32Index::default();
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
        println!("vvvvv------{:?}", cur.1);
        cur = slab.prev_true(cur.1);
    }

}

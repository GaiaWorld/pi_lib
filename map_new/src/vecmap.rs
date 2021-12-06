//! 实现数据结构`VecMap`, 并为`VecMap`实现了`Map<K=usize,V=T>`
//! 就像其名字描述的一样，`VecMap`以Vec作为数据结构，实现索引到值得映射。
//! 与HashMap的区别是：
//! * HashMap的key可以是任何实现了`Hash`的类型，`VecMap`的key一定是一个`usize`
//! * 在性能方面，HashMap将key做hash运算，最后将值存储在该hash对应的一个数组或链表上；VecMap将key作为`Vec`的偏移,直接存放在数组的对应位置。VecMap具有更高的性能。
//! * 正因为VecMap将key作为偏移，存储值到数组的对应位置，`VecMap`可能浪费更多内存空间；如当前仅在VecMap中存储两个值，key分别是`1`, `100`, 那么，数组的第1、100的位置将存储一个值，而2..99的位置将是浪费的空间
//!
//! 与Vec的区别是：
//! * Vec不提供类似`Map`的接口来操作数据，如`insert`。
//! 
//! VecMap通常用于，存放的数据的key总是与某种数据结构的偏移相对应的情况。
//! 比如在ecs系统中，实体的数据接口可以是slab，我们可以创建和销毁实体，对应在slab中的操作是，分配一个位置（偏移）和释放一个位置
//! 而组件的数据接口，使用VecMap再合适不过，表示实体的数字，可以作为key，来映射一个组件的值。
//! 同时，VecMap的数据也是基本连续的，十分符合ecs的设计思想
//!
//! 再决定使用VecMap前，你应该综合考虑这几个问题：访问性能、数据连续性、内存的浪费情况。
//!
use std::mem::{replace};
use std::fmt::{Debug, Formatter, Result as FResult};
use std::ops::{Index, IndexMut};
use std::slice::Iter;
// use std::ops::Drop;
// use std::ptr::write;

use crate::Map;
// TODO 改成类似slab的写法，用单独的vec<usize>的位记录是否为空。现在这种写法太费内存了
/// 数据结构VecMap
pub struct VecMap<T> {
    entries: Vec<Option<T>>,// Chunk of memory
    len: usize,// Number of Filled elements currently in the slab
}

impl<T> Default for VecMap<T> {
    fn default() -> Self {
        VecMap::new()
    }
}
impl<T: Clone> Clone for VecMap<T> {
    fn clone(&self) -> Self {
        VecMap {
            entries: self.entries.to_vec(),
            len: self.len,
        }
    }
}
impl<T> VecMap<T> {

    /// 创建一个VecMap实例
    pub fn new() -> Self {
        VecMap::with_capacity(0)
    }

    /// 创建一个VecMap实例, 并指定初始化容量
    pub fn with_capacity(capacity: usize) -> VecMap<T> {
        VecMap {
            entries: Vec::with_capacity(capacity),
            len: 0,
        }
    }

    /// 获取VecMap当前的容量
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// 扩充容量
    pub fn reserve(&mut self, additional: usize) {
        if self.capacity() - self.len >= additional {
            return;
        }
        let need_add = self.len + additional - self.entries.len();
        self.entries.reserve(need_add);
    }

    /// 扩充容量
    pub fn reserve_exact(&mut self, additional: usize) {
        if self.capacity() - self.len >= additional {
            return;
        }
        let need_add = self.len + additional - self.entries.len();
        self.entries.reserve_exact(need_add);
    }

    // pub fn shrink_to_fit(&mut self) {
    //     self.entries.shrink_to_fit();
    //     self.vacancy_sign.shrink_to_fit();
    // }
    
    /// 清空数据
    pub fn clear(&mut self) {
        self.entries.clear();
        self.len = 0;
    }

    /// 片段当前是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 获取一个只读迭代器
    pub fn iter(&self) -> Iter<Option<T>> {
        self.entries.iter()
    }

    // pub fn iter_mut(&mut self) -> SlabIterMut<T> {
    //     SlabIterMut {
    //         entries: &mut self.entries as *mut Vec<T>,
    //         signs: &mut self.vacancy_sign,
    //         curr_index: 0,
    //         len: self.len,
    //         curr_len: 0,
    //     }
    // }
    
    /// 替换指定位置的值, 并返回旧值
    /// 你应该确认，旧值一定存在，否则将会panic
    pub unsafe fn replace(&mut self, index: usize, value: T) -> T {
        replace(&mut self.entries[index], Some(value)).unwrap()
    }


    /// 取到某个偏移位置的只读值
    pub fn get(&self, index: usize) -> Option<&T> {
        if index == usize::max_value() || index >= self.entries.len(){
            return None;
        }
        match &self.entries[index] {
            Some(v) => Some(v),
            None => None,
        }
    }

    /// 取到某个偏移位置的可变值
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index == usize::max_value() || index >= self.entries.len(){
            return None;
        }
        match &mut self.entries[index] {
            Some(v) => Some(v),
            None => None,
        }
    }

    /// 取到某个偏移位置的只读值
    /// 如果该位置不存在值，将panic
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.entries[index].as_ref().unwrap()
    }

    /// 取到某个偏移位置的可变值
    /// 如果该位置不存在值，将panic
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.entries[index].as_mut().unwrap()
    }

    /// 在指定位置插入一个值，并返回旧值，如果不存在旧值，返回None
    pub fn insert(&mut self, index:usize, val: T) -> Option<T>{
		let len = self.entries.len();
		if len == index {
			self.entries.push(Some(val));
			self.len += 1;
            None
		} else if index >= len {
			self.entries.extend((0..index - len + 1).map(|_| None));
            self.len += 1;
            replace(&mut self.entries[index], Some(val))
        }else {
            let r = replace(&mut self.entries[index], Some(val));
            if r.is_none(){
                self.len += 1;
            }
            r
        }
    }

    /// 移除指定位置的值，返回被移除的值，如果该位置不存在一个值，返回None
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index > self.entries.len() {
            return None;
        }
        match replace(&mut self.entries[index], None) {
            Some(v) => {
                self.len -= 1;
                Some(v)
            },
            None => None,
        }
    }

    /// 移除指定位置的值，返回被移除的值，如果该位置不存在一个值将panic
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        self.len -= 1;
        replace(&mut self.entries[index], None).unwrap()
    }

    /// 判断指定位置是否存在一个值
    pub fn contains(&self, index: usize) -> bool {
        if index == usize::max_value() || index >= self.entries.len(){
            return false;
        }
        match &self.entries[index] {
            Some(_v) => true,
            None => false,
        }
    }

    /// 取到VecMap的长度
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

/// 为VecMap实现Map
impl<T> Map for VecMap<T> {
	type Key = usize;
	type Val = T;
    #[inline]
    fn get(&self, key: &usize) -> Option<&T> {
        self.get(*key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut T> {
        self.get_mut(*key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &usize) -> &T {
        self.get_unchecked(*key)
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &usize) -> &mut T {
        self.get_unchecked_mut(*key)
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &usize) -> T {
        self.remove_unchecked(*key)
    }

    #[inline]
    fn insert(&mut self, key: usize, val: T) -> Option<T> {
        self.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<T> {
        self.remove(*key)
    }

    #[inline]
    fn contains(&self, key: &usize) -> bool {
        self.contains(*key)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }
    #[inline]
    fn capacity(&self) -> usize {
        self.entries.capacity()
    }
    #[inline]
    fn mem_size(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
	}
	
	fn with_capacity(capacity: usize) -> Self {
		VecMap::with_capacity(capacity)
	}
}

impl<T> Index<usize> for VecMap<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.entries[index].as_ref().unwrap()
    }
}

impl<T> IndexMut<usize> for VecMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.entries[index].as_mut().unwrap()
    }
}

// impl<'a, T> IntoIterator for &'a Slab<T> {
//     type Item = (usize, &'a T);
//     type IntoIter = SlabIter<'a, T>;

//     fn into_iter(self) -> SlabIter<'a, T> {
//         self.iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a mut Slab<T> {
//     type Item = (usize, &'a mut T);
//     type IntoIter = SlabIterMut<'a, T>;

//     fn into_iter(self) -> SlabIterMut<'a, T> {
//         self.iter_mut()
//     }
// }

impl<T> Debug for VecMap<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        write!(fmt,
               "Slab {{ len: {}, entries:{:?} }}",
               self.len,
               self.entries)
    }
}

// pub struct SlabIter<'a, T: 'a> {
//     signs: &'a Vec<usize>,
//     entries: &'a Vec<T>,
//     curr_index: usize,
//     curr_len: usize,
//     len: usize,
// }

// pub struct SlabIterMut<'a, T: 'a> {
//     signs: &'a Vec<usize>,
//     entries: *mut Vec<T>,
//     curr_index: usize,
//     curr_len: usize,
//     len: usize,
// }

// impl<'a, T: 'a> Debug for SlabIter<'a, T> where T: Debug {
//     fn fmt(&self, fmt: &mut Formatter) -> FResult {
//         fmt.debug_struct("Iter")
//             .field("curr", &self.curr_index)
//             .field("remaining", &self.len)
//             .finish()
//     }
// }

// impl<'a, T: 'a> Debug for SlabIterMut<'a, T> where T: Debug {
//     fn fmt(&self, fmt: &mut Formatter) -> FResult {
//         fmt.debug_struct("IterMut")
//             .field("curr", &self.curr_index)
//             .field("remaining", &self.len)
//             .finish()
//     }
// }


// ===== Iter =====

// impl<'a, T> Iterator for SlabIter<'a, T> {
//     type Item = (usize, &'a T);

//     fn next(&mut self) -> Option<(usize, &'a T)> {
//         if self.curr_len == self.len {
//             return None;
//         }
//         let sign_index = self.curr_index/usize_size();
//         let mut sign_index1 = self.curr_index%usize_size();
//         for i in sign_index..self.signs.len(){
//             let sign = self.signs[i].clone() >> sign_index1;
//             if sign != usize::max_value() >> sign_index1{
//                 let first_zero = find_zero(sign);
//                 let curr = self.curr_index + first_zero;
//                 self.curr_index = curr + 1;
//                 self.curr_len += 1;
//                 return Some((curr + 1, &self.entries[curr]));
//             }else {
//                 self.curr_index += 8 - sign_index1;
//                 sign_index1 = 0;
//             }
//         }
//         None
//     }
// }

// // ===== IterMut =====

// impl<'a, T> Iterator for SlabIterMut<'a, T> {
//     type Item = (usize, &'a mut T);

//     fn next(&mut self) -> Option<(usize, &'a mut T)> {
//         if self.curr_len == self.len {
//             return None;
//         }
//         let sign_index = self.curr_index/usize_size();
//         let mut sign_index1 = self.curr_index%usize_size();
//         for i in sign_index..self.signs.len(){
//             let sign = self.signs[i].clone() >> sign_index1;
//             if sign != usize::max_value() >> sign_index1{
//                 let first_zero = find_zero(sign);
//                 let curr = self.curr_index + first_zero;
//                 self.curr_index = curr + 1;
//                 self.curr_len += 1;
//                 return Some((curr + 1, unsafe{&mut (*self.entries)[curr]} ));
//             }else {
//                 self.curr_index += 8 - sign_index1;
//                 sign_index1 = 0;
//             }
//         }
//         None
//     }
// }

#[cfg(test)]
extern crate time;
// #[cfg(test)]
// use time::now_microsecond;
#[cfg(test)]
use std::time::Instant;
#[test]
fn test_time(){
    let mut map: VecMap<[f32; 16]> = VecMap::new();
    map.entries = Vec::with_capacity(0);

    let mut arr = Vec::with_capacity(100000);
    let time = Instant::now();
    for _i in 0..10000 {
        arr.push(Some([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]));
    }
    println!("insert vec time: {:?}", Instant::now() - time);

    let time = Instant::now();
    for i in 1..10001 {
        map.insert(i, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]);
    }
    println!("insert vecmap time: {:?}", Instant::now() - time);


    let mut map: VecMap<f32> = VecMap::new();
    map.entries = Vec::with_capacity(100000);

    let time = Instant::now();
    for i in 1..10001 {
        map.insert(i, 1.0);
    }
    println!("insert vecmap time: {:?}", Instant::now() - time);

}
#[test]
fn test(){
    let mut map: VecMap<u64> = VecMap::new();
    for i in 1..71{
        map.insert(i as usize, i);
        println!("map------{:?}", map);
    }

    map.remove(30);
    println!("r 30------{:?}", map);

    map.remove(31);
    println!("r 31------{:?}", map);

    map.remove(69);
    println!("r 69------{:?}", map);

    map.remove(70);
    println!("r 70------{:?}", map);

    assert_eq!(map.contains(0), false);
    assert_eq!(map.contains(1), true);
    assert_eq!(map.contains(71), false);
    assert_eq!(map.contains(72), false);

    assert_eq!(map.get(0), None);
    assert_eq!(map.get(1), Some(&1));
    assert_eq!(map.get(50), Some(&50));
    assert_eq!(map.get(70), None);
    assert_eq!(map.get(72), None);


    assert_eq!(map.get_mut(0), None);
    assert_eq!(map.get_mut(64), Some(&mut 64));
    assert_eq!(map.get_mut(30), None);
    assert_eq!(map.get_mut(20), Some(&mut 20));
    assert_eq!(map.get_mut(75), None);

    assert_eq!(unsafe{map.get_unchecked(2)}, &2);
    assert_eq!(unsafe{map.get_unchecked(9)}, &9);
    assert_eq!(unsafe{map.get_unchecked(55)}, &55);
    assert_eq!(unsafe{map.get_unchecked(60)}, &60);

    assert_eq!(unsafe{map.get_unchecked_mut(44)}, &mut 44);
    assert_eq!(unsafe{map.get_unchecked_mut(33)}, &mut 33);
    assert_eq!(unsafe{map.get_unchecked_mut(7)}, &mut 7);
}

// #[test]
// fn test_eff(){
    
//     let mut map: VecMap<u64> = VecMap::new();
//     let time = now_millis();
//     for i in 1..1000001{
//         map.insert(i as usize, i);
//     }
//     let time1 = now_millis();
//     println!("insert time-----------------------------------------------{}", time1 - time);

//     for i in 1..1000001{
//         unsafe { map.remove(i) };
//     }
//     let time2 = now_millis();
//     println!("remove time-----------------------------------------------{}", time2 - time1);

//     let mut v = Vec::new();

//     let time3 = now_millis();
//     for i in 1..1000001{
//         v.push(i);
//     }

//     let time4 = now_millis();
//     println!("insert vec time-----------------------------------------------{}", time4 - time3);
// }

// #[test]
// fn m(){
//     //let a: usize = (usize::max_value() - 1) << 1;
//     println!("xxxxxxxxxxxxxxxxxxxxxx");
// }
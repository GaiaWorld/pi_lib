
#![allow(warnings)]

use std::mem::{size_of, transmute_copy, needs_drop, replace};
use std::fmt::{Debug, Formatter, Result as FResult};
use std::ops::{Index, IndexMut};
use std::iter::IntoIterator;
use std::ops::Drop;
use std::ptr::write;

pub trait IndexMap<T>: Index<usize> + IndexMut<usize>{
    fn len(&self) -> usize;
    fn get(&self, key: usize) -> Option<&T>;
    fn get_mut(&mut self, key: usize) -> Option<&mut T>;
    fn contains(&self, key: usize) -> bool;
    unsafe fn get_unchecked(&self, key: usize) -> &T;
    unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut T;
    fn insert(&mut self, val: T) -> usize;
    fn remove(&mut self, key: usize) -> T;
}

pub struct Slab<T> {
    entries: Vec<T>,// Chunk of memory
    vacancy_sign: Vec<usize>,// sign for vacancy
    len: usize,// Number of Filled elements currently in the slab
    next: usize, //Offset of the next vacancy
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Slab::new()
    }
}
impl<T: Clone> Clone for Slab<T> {
    fn clone(&self) -> Self {
        Slab {
            entries: self.entries.to_vec(),
            vacancy_sign: self.vacancy_sign.to_vec(),
            len: self.len,
            next: self.next,
        }
    }
}

impl<T> Slab<T> {

    pub fn new() -> Slab<T> {
        Slab::with_capacity(0)
    }

    pub fn mem_size(&self) -> usize {
        self.entries.capacity() * std::mem::size_of::<T>() + self.vacancy_sign.capacity() * std::mem::size_of::<usize>()
    }

    pub fn with_capacity(capacity: usize) -> Slab<T> {
        Slab {
            entries: Vec::with_capacity(capacity),
            next: 0,
            len: 0,
            vacancy_sign: Vec::with_capacity(capacity/usize_size()),
        }
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.capacity() - self.len >= additional {
            return;
        }
        let need_add = self.len + additional - self.entries.len();
        self.entries.reserve(need_add);
        self.vacancy_sign.reserve(need_add/usize_size());
    }

    
    pub fn reserve_exact(&mut self, additional: usize) {
        if self.capacity() - self.len >= additional {
            return;
        }
        let need_add = self.len + additional - self.entries.len();
        self.entries.reserve_exact(need_add);
        self.vacancy_sign.reserve(need_add/usize_size());
    }

    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.vacancy_sign.shrink_to_fit();
    }

    pub fn clear(&mut self) {
        self.clear_entries();
        // 如果T实现了drop，则clear_entries负责销毁T，但是，如果T没有实现drop，则clear_entries什么都没做，人需要手动设置entries的长度为0
        unsafe {self.entries.set_len(0)}
        self.vacancy_sign.clear();
        self.len = 0;
        self.next = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> SlabIter<T> {
        SlabIter {
            entries: &self.entries,
            signs: &self.vacancy_sign,
            curr_index: 0,
            len: self.len,
            curr_len: 0,
        }
    }

    pub fn iter_mut(&mut self) -> SlabIterMut<T> {
        SlabIterMut {
            entries: &mut self.entries as *mut Vec<T>,
            signs: &mut self.vacancy_sign,
            curr_index: 0,
            len: self.len,
            curr_len: 0,
        }
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        if key == usize::max_value() || key > self.entries.len() || self.is_one(key){
            return None;
        }
        return Some(&self.entries[key]);
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if key == usize::max_value() || key > self.entries.len() || self.is_one(key){
            return None;
        }
        return Some(&mut self.entries[key])
        
    }

    pub unsafe fn get_unchecked(&self, key: usize) -> &T {
        return &self.entries[key];
    }

    pub unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut T {
        return &mut self.entries[key];
    }

    pub fn alloc_with_is_first(&mut self) -> (usize, &mut T, bool){
        let key = self.next;
        let r = self.alloc_with_is_first_at(key);
        (key, r.0, r.1)
    }

    pub fn alloc_with_is_first_at(&mut self, key: usize) -> (&mut T, bool) {
        let len = self.entries.len();
        self.len += 1;
        let t = if key == len {
            if len == self.capacity() {
                self.entries.reserve(1);
            }
            unsafe{self.entries.set_len(len + 1)};
            self.next = key + 1;
            let s_index = key%usize_size();
            if s_index == 0{
                self.vacancy_sign.push(usize::max_value());
            }else {
                one2zero(&mut self.vacancy_sign[key/usize_size()], s_index);
            }
            (&mut self.entries[key], true)
        } else {
            self.next = unsafe{*(&self.entries[key] as *const T as usize as *const usize)}.clone();
            self.one2zero(key);
            (&mut self.entries[key], false)
        };
        t
    }

    pub fn alloc(&mut self) -> (usize, &mut T){
        let key = self.next;
        (key, self.alloc_at(key))
    }

    pub fn alloc_at(&mut self, key: usize) -> &mut T {
        let len = self.entries.len();
        self.len += 1;
        let t = if key == len {
            if len == self.capacity() {
                self.entries.reserve(1);
            }
            unsafe{self.entries.set_len(len + 1)};
            self.next = key + 1;
            let s_index = key%usize_size();
            if s_index == 0{
                self.vacancy_sign.push(usize::max_value() - 1);
            }else {
                one2zero(&mut self.vacancy_sign[key/usize_size()], s_index);
            }
            &mut self.entries[key]
        } else {
            self.next = unsafe{*(&self.entries[key] as *const T as usize as *const usize)}.clone();
            self.one2zero(key);
            &mut self.entries[key]
        };
        t
    }

    pub fn insert(&mut self, val: T) -> usize {
        let key = self.next;
        self.insert_at(key, val);
        key
    }

    pub fn insert_at(&mut self, key: usize, val: T) {
        if key == self.entries.len() {
            self.entries.push(val);
            self.next = key + 1;
            let s_index = key%usize_size();
            if s_index == 0{
                self.vacancy_sign.push(usize::max_value() - 1);
            }else {
                one2zero(&mut self.vacancy_sign[key/usize_size()], s_index);
            }
        } else {
            self.next = unsafe{*(&mut self.entries[key] as *mut T as usize as *mut usize)};
            unsafe{write(&mut self.entries[key] as *mut T, val)};
            self.one2zero(key);
           
        }
        self.len += 1;
    }

    pub fn remove(&mut self, key: usize) -> T {
        let r: T = unsafe{ transmute_copy(&self.entries[key]) };
        unsafe{*(&mut self.entries[key] as *mut T as usize as *mut usize) = self.next };
        self.next = key;
        self.zero2one(key);
        self.len -= 1;
        r
    }

    pub unsafe fn replace(&mut self, key: usize, value: T) -> T {
        replace(&mut self.entries[key], value)
    }

    pub fn contains(&self, key: usize) -> bool {
        if key == usize::max_value() || key >= self.entries.len() || self.is_one(key) {
            false
        } else{
            true
        }
    }

    // pub fn retain<F>(&mut self, mut f: F) where F: FnMut(usize, &mut T) -> bool {
    //     for i in 0..self.entries.len() {
    //         let keep = match self.entries[i] {
    //             Entry::Occupied(ref mut v) => f(i, v),
    //             _ => true,
    //         };

    //         if !keep {
    //             self.remove(i);
    //         }
    //     }
    // }

    #[inline]
    fn zero2one(&mut self, index: usize){
        zero2one(&mut self.vacancy_sign[index/usize_size()], index%usize_size());
    }

    #[inline]
    fn one2zero(&mut self, index: usize){
        one2zero(&mut self.vacancy_sign[index/usize_size()], index%usize_size());
    }

    #[inline]
    fn is_one(&self, key: usize) -> bool{
        let signs = unsafe {self.vacancy_sign.get_unchecked(key/usize_size())};
        is_one(signs, key%usize_size())
    }

    fn clear_entries(&mut self){
        if needs_drop::<T>(){
            let count = 0;
            if self.entries.len() == 0 {
                return;
            }
            let index = self.entries.len() - 1;
            let mut index1 = index/usize_size();
            let mut index2 = index%usize_size();
            loop {
                let signs = self.vacancy_sign[index1].clone();
                let diff = usize_size() - index2 - 1;
                let signs =  signs<<diff;
                let len = self.entries.len();
                if signs == usize::max_value()<<diff {  //如果全是空位， 不需要drop
                    unsafe{ self.entries.set_len(len - (index2 + 1))};
                }else{ //否则找到容器尾部的非空位， 移除元素（移除即drop）
                    let i = (!signs).leading_zeros() as usize;
                    unsafe{ self.entries.set_len(len - i)};
                    self.entries.pop();
                    if index2 - i != 0{
                        index2 = index2 - i - 1;
                        continue;
                    }
                }
                if index1 == 0 {
                    return;
                }
                index1 -= 1;
                index2 = usize_size() - 1;
                if count == self.len(){
                    break;
                }
            }
        }
    }
}

impl<T> IndexMap<T> for Slab<T>{
    #[inline]
    fn len(&self) -> usize{
        self.len()
    }
    #[inline]
    fn get(&self, key: usize) -> Option<&T> {
        self.get(key)
    }
    #[inline]
    fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        self.get_mut(key)
    }
    #[inline]
    fn contains(&self, key: usize) -> bool{
        self.contains(key)
    }
    #[inline]
    unsafe fn get_unchecked(&self, key: usize) -> &T{
        self.get_unchecked(key)
    }
    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut T{
        self.get_unchecked_mut(key)
    }
    #[inline]
    fn insert(&mut self, val: T) -> usize{
        self.insert(val)
    }
    #[inline]
    fn remove(&mut self, key: usize) -> T{
        self.remove(key)
    }
}

impl<T> Drop for Slab<T>{
    fn drop(&mut self) {
        self.clear_entries();
    }
}

impl<T> Index<usize> for Slab<T> {
    type Output = T;

    fn index(&self, key: usize) -> &T {
        &self.entries[key]
    }
}

impl<T> IndexMut<usize> for Slab<T> {
    fn index_mut(&mut self, key: usize) -> &mut T {
        &mut self.entries[key]
    }
}

impl<'a, T> IntoIterator for &'a Slab<T> {
    type Item = (usize, &'a T);
    type IntoIter = SlabIter<'a, T>;

    fn into_iter(self) -> SlabIter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Slab<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = SlabIterMut<'a, T>;

    fn into_iter(self) -> SlabIterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> Debug for Slab<T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        //let vacancy_sign = Vec<()>;
        let mut s = String::from("[");
        for v in self.vacancy_sign.iter(){
            s += &format!("{:b}", v);
            s += ",";
        }
        s += "]";
        write!(fmt,
               "Slab {{ len: {}, cap: {}, next: {}, vacancy_sign: {}, entries:{:?} }}",
               self.len,
               self.capacity(),
               self.next,
               s,
               self.entries)
    }
}

pub struct SlabIter<'a, T: 'a> {
    signs: &'a Vec<usize>,
    entries: &'a Vec<T>,
    curr_index: usize,
    curr_len: usize,
    len: usize,
}

pub struct SlabIterMut<'a, T: 'a> {
    signs: &'a Vec<usize>,
    entries: *mut Vec<T>,
    curr_index: usize,
    curr_len: usize,
    len: usize,
}

impl<'a, T: 'a> Debug for SlabIter<'a, T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        fmt.debug_struct("Iter")
            .field("curr", &self.curr_index)
            .field("remaining", &self.len)
            .finish()
    }
}

impl<'a, T: 'a> Debug for SlabIterMut<'a, T> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        fmt.debug_struct("IterMut")
            .field("curr", &self.curr_index)
            .field("remaining", &self.len)
            .finish()
    }
}


// ===== Iter =====

impl<'a, T> Iterator for SlabIter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<(usize, &'a T)> {
        if self.curr_len == self.len {
            return None;
        }
        let sign_index = self.curr_index/usize_size();
        let mut sign_index1 = self.curr_index%usize_size();
        for i in sign_index..self.signs.len(){
            let sign = self.signs[i].clone() >> sign_index1;
            if sign != usize::max_value() >> sign_index1{
                let first_zero = find_zero(sign);
                let curr = self.curr_index + first_zero;
                self.curr_index = curr + 1;
                self.curr_len += 1;
                return Some((curr, &self.entries[curr]));
            }else {
                self.curr_index += usize_size() - sign_index1;
                sign_index1 = 0;
            }
        }
        None
    }
}

// ===== IterMut =====

impl<'a, T> Iterator for SlabIterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<(usize, &'a mut T)> {
        if self.curr_len == self.len {
            return None;
        }
        let sign_index = self.curr_index/usize_size();
        let mut sign_index1 = self.curr_index%usize_size();
        for i in sign_index..self.signs.len(){
            let sign = self.signs[i].clone() >> sign_index1;
            if sign != usize::max_value() >> sign_index1{
                let first_zero = find_zero(sign);
                let curr = self.curr_index + first_zero;
                self.curr_index = curr + 1;
                self.curr_len += 1;
                return Some((curr, unsafe{&mut (*self.entries)[curr]} ));
            }else {
                self.curr_index += usize_size() - sign_index1;
                sign_index1 = 0;
            }
        }
        None
    }
}

#[inline]
fn is_one(i: &usize, index: usize) -> bool {
    ((i >> index) & 1 == 1)
}

#[inline]
fn zero2one(i: &mut usize, index: usize){
    (*i) = *i | (1 << index);
}

#[inline]
fn one2zero(i: &mut usize, index: usize){
    (*i) = *i - (1 << index);
}

#[inline]
fn usize_size() -> usize{
    size_of::<usize>() * 8
}

// 返回指定的数字中从最低位开始第一个0的位置
#[inline]
fn find_zero(i:usize) -> usize {
	let a = !i;
    a.trailing_zeros() as usize
}

#[cfg(test)]
extern crate time;

#[test]
fn test(){
    let mut slab: Slab<u64> = Slab::new();
    for i in 1..71{
        slab.insert(i);
        println!("slab------{:?}", slab);
    }

    slab.remove(30);
    println!("r 30------{:?}", slab);

    slab.remove(31);
    println!("r 31------{:?}", slab);

    slab.remove(69);
    println!("r 69------{:?}", slab);

    slab.remove(70);
    println!("r 70------{:?}", slab);

    {
        let mut it = slab.iter_mut();
        println!("itermut start-----------------------------------------------");
        loop {
            match it.next() {
                Some(n) => {
                    print!("{:?},", n);
                },
                None => break,
            };
        }
        println!("itermut end-----------------------------------------------");
    }

    slab.insert(70);
    println!("i 70------{:?}", slab);

    slab.insert(60);
    println!("i 60------{:?}", slab);

    slab.insert(31);
    println!("i 31------{:?}", slab);

    slab.insert(30);
    println!("i 31------{:?}", slab);

    slab.insert(71);
    println!("i 71------{:?}", slab);

    assert_eq!(slab.contains(0), false);
    assert_eq!(slab.contains(1), true);
    assert_eq!(slab.contains(71), true);
    assert_eq!(slab.contains(72), false);

    assert_eq!(slab.get(0), None);
    assert_eq!(slab.get(1), Some(&1));
    assert_eq!(slab.get(50), Some(&50));
    assert_eq!(slab.get(70), Some(&70));
    assert_eq!(slab.get(72), None);


    assert_eq!(slab.get_mut(0), None);
    assert_eq!(slab.get_mut(64), Some(&mut 64));
    assert_eq!(slab.get_mut(30), Some(&mut 30));
    assert_eq!(slab.get_mut(20), Some(&mut 20));
    assert_eq!(slab.get_mut(75), None);

    assert_eq!(unsafe{slab.get_unchecked(2)}, &2);
    assert_eq!(unsafe{slab.get_unchecked(9)}, &9);
    assert_eq!(unsafe{slab.get_unchecked(55)}, &55);
    assert_eq!(unsafe{slab.get_unchecked(60)}, &60);

    assert_eq!(unsafe{slab.get_unchecked_mut(31)}, &mut 31);
    assert_eq!(unsafe{slab.get_unchecked_mut(44)}, &mut 44);
    assert_eq!(unsafe{slab.get_unchecked_mut(33)}, &mut 33);
    assert_eq!(unsafe{slab.get_unchecked_mut(7)}, &mut 7);

    let mut it = slab.iter();
    println!("iter start-----------------------------------------------");
    loop {
        match it.next() {
            Some(n) => {
                print!("{:?},", n);
            },
            None => break,
        };
    }
    println!("iter end-----------------------------------------------");
}

#[test]
fn test_alloc(){
    let mut slab: Slab<u64> = Slab::new();
    for i in 1..71{
        let r = slab.alloc();
        *r.1 = i;
    }
    println!("slab ------{:?}", slab);
}



#[test]
fn test_eff(){
    use time::now_millisecond;
    let mut slab: Slab<u64> = Slab::new();
    let time = now_millisecond();
    for i in 0..1000000{
        let r = slab.alloc();
        *r.1 = i;
    }
    println!("alloc time-----------------------------------------------{}", now_millisecond() - time);
    let mut slab: Slab<u64> = Slab::new();
    let time = now_millisecond();
    for i in 0..1000000{
        slab.insert(i);
    }
    let time1 = now_millisecond();
    println!("insert time-----------------------------------------------{}", time1 - time);

    for i in 1..1000001{
        slab.remove(i);
    }
    let time2 = now_millisecond();
    println!("remove time-----------------------------------------------{}", time2 - time1);

    let time = now_millisecond();
    for i in 0..1000000{
        slab.insert(i);
    }
    let time1 = now_millisecond();
    println!("insert1 time-----------------------------------------------{}", time1 - time);

    let mut v = Vec::new();

    let time3 = now_millisecond();
    for i in 1..1000001{
        v.push(i);
    }

    let time4 = now_millisecond();
    println!("insert vec time-----------------------------------------------{}", time4 - time3);
}

// #[test]
// fn m(){
//     //let a: usize = (usize::max_value() - 1) << 1;
//     println!("xxxxxxxxxxxxxxxxxxxxxx");
// }
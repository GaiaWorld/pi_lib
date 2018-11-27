
use std::mem::{size_of, replace, transmute_copy, needs_drop, drop};
use std::fmt::{Debug, Formatter, Result as FResult};
use std::ops::{Index, IndexMut};
use std::iter::IntoIterator;
use std::ops::Drop;

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

impl<T> Slab<T> {

    pub fn new() -> Slab<T> {
        Slab::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Slab<T> {
        Slab {
            entries: Vec::with_capacity(capacity),
            next: 0,
            len: 0,
            vacancy_sign: Vec::with_capacity(usize_size() << 1),
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
        self.entries.clear();
        self.len = 0;
        self.next = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

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
        if key == 0 || key > self.entries.len() || self.is_one(key - 1){
            return None;
        }
        return Some(&self.entries[key - 1]);
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if key == 0 || key > self.entries.len() || self.is_one(key - 1){
            return None;
        }
        return Some(&mut self.entries[key - 1])
        
    }

    pub unsafe fn get_unchecked(&self, key: usize) -> &T {
        return &self.entries[key - 1];
    }

    pub unsafe fn get_unchecked_mut(&mut self, key: usize) -> &mut T {
        return &mut self.entries[key - 1];
    }
    pub fn alloc(&mut self) -> (usize, &mut T){
        let key = self.next;
        (key +1, self.alloc_at(key))
    }

    #[inline]
    fn alloc_at(&mut self, key: usize) -> &mut T {
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
        key + 1
    }

    #[inline]
    fn insert_at(&mut self, key: usize, val: T) {
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
            let mut prev = replace(&mut self.entries[key], val);
            self.next = unsafe{*(&mut prev as *mut T as usize as *mut usize)};
            self.one2zero(key);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, key: usize) -> T {
        let key1 = key - 1;
        let r: T = unsafe{ transmute_copy(&self.entries[key1]) };
        unsafe{*(&mut self.entries[key1] as *mut T as usize as *mut usize) = self.next };
        self.next = key1;
        self.zero2one(key1);
        self.len -= 1;
        r
    }

    pub fn contains(&self, key: usize) -> bool {
        if key == 0 || key > self.entries.len() || self.is_one(key - 1) {
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
}

impl<T> Drop for Slab<T>{
    fn drop(&mut self) {
        if needs_drop::<T>(){
            let index = self.entries.len() - 1;
            let mut index1 = index/usize_size();
            let mut index2 = index%usize_size();
            loop {
                let signs = self.vacancy_sign[index1].clone();
                let diff = usize_size() - index2 + 1;
                let signs =  signs<<diff;
                if signs == usize_size()<<diff {  //如果全是空位， 不需要drop
                    unsafe{ self.entries.set_len(self.entries.len() - (index2 + 1))};
                }else{ //否则找到容器尾部的非空位， 移除元素（移除即drop）
                    let i = signs.leading_zeros() as usize;
                    unsafe{ self.entries.set_len(self.entries.len() - i)};
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
                index2 = 0;
            }
        }
    }
}

impl<T> Index<usize> for Slab<T> {
    type Output = T;

    fn index(&self, key: usize) -> &T {
        unsafe{ self.get_unchecked(key) }
    }
}

impl<T> IndexMut<usize> for Slab<T> {
    fn index_mut(&mut self, key: usize) -> &mut T {
        unsafe{ self.get_unchecked_mut(key) }
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
                return Some((curr + 1, &self.entries[curr]));
            }else {
                self.curr_index += 8 - sign_index1;
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
                return Some((curr + 1, unsafe{&mut (*self.entries)[curr]} ));
            }else {
                self.curr_index += 8 - sign_index1;
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

// 返回指定的数字中低位第一个0的位置
#[inline]
fn find_zero(i:usize) -> usize {
	let a = !i;
    a.trailing_zeros() as usize
}

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
    use time::now_millis;
    let mut slab: Slab<u64> = Slab::new();
    let time = now_millis();
    for i in 0..1000000{
        let r = slab.alloc();
        *r.1 = i;
    }
    println!("alloc time-----------------------------------------------{}", now_millis() - time);
    let mut slab: Slab<u64> = Slab::new();
    let time = now_millis();
    for i in 0..1000000{
        slab.insert(i);
    }
    let time1 = now_millis();
    println!("insert time-----------------------------------------------{}", time1 - time);

    for i in 1..1000001{
        slab.remove(i);
    }
    let time2 = now_millis();
    println!("remove time-----------------------------------------------{}", time2 - time1);

    let time = now_millis();
    for i in 0..1000000{
        slab.insert(i);
    }
    let time1 = now_millis();
    println!("insert1 time-----------------------------------------------{}", time1 - time);

    let mut v = Vec::new();

    let time3 = now_millis();
    for i in 1..1000001{
        v.push(i);
    }

    let time4 = now_millis();
    println!("insert vec time-----------------------------------------------{}", time4 - time3);
}
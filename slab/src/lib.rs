
extern crate map;
extern crate ver_index;

use std::mem::{transmute_copy, needs_drop, replace};
use std::fmt::{Debug, Formatter, Result as FResult};
use std::iter::IntoIterator;
use std::ops::Drop;
use std::ptr::write;

use map::Map;
use ver_index::VerIndex;


pub trait IdAllocater<T> {
    type ID: Copy + Debug + PartialEq + Default + Send + Sync;
    fn alloc(&mut self, val: T) -> Self::ID;
}

pub struct Slab<T, I: VerIndex> {
    entries: Vec<T>,// Chunk of memory
    indexs: I,// version index
    len: usize,
    next: usize, //Offset of the next vacancy
}

impl<T, I: VerIndex + Default> Default for Slab<T, I> {
    fn default() -> Self {
        Slab {
            entries: Vec::new(),
            indexs: I::default(),
            len: 0,
            next: 0,
        }
    }
}
impl<T: Clone, I: VerIndex + Clone> Clone for Slab<T, I> {
    fn clone(&self) -> Self {
        Slab {
            entries: self.entries.to_vec(),
            indexs: self.indexs.clone(),
            len: self.len,
            next: self.next,
        }
    }
}

impl<T, I: VerIndex> Slab<T, I> {

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
        self.indexs.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.indexs.shrink_to_fit();
    }

    pub fn clear(&mut self) {
        self.clear_entries();
        self.indexs.clear();
        self.len = 0;
        self.next = 0;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> SlabIter<T, I> {
        SlabIter {
            entries: &self.entries,
            indexs: &self.indexs,
            len: self.len,
            cur_index: self.indexs.first_true(),
            cur_len: 0,
        }
    }

    pub fn iter_mut(&mut self) -> SlabIterMut<T, I> {
        SlabIterMut {
            entries: &mut self.entries,
            indexs: &self.indexs,
            len: self.len,
            cur_index: self.indexs.first_true(),
            cur_len: 0,
        }
    }

    #[inline(always)]
    pub fn get(&self, key: I::ID) -> Option<&T> {
        let (v, k) = self.indexs.split(key);
        if k == 0 || k > self.entries.len() || self.indexs.version(k - 1) != v{
            return None;
        }
        return Some(unsafe{self.entries.get_unchecked(k - 1)})
    }

    #[inline(always)]
    pub fn get_mut(&mut self, key: I::ID) -> Option<&mut T> {
        let (v, k) = self.indexs.split(key);
        if k == 0 || k > self.entries.len() || self.indexs.version(k - 1) != v{
            return None;
        }
        return Some(unsafe{self.entries.get_unchecked_mut(k - 1)})
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, key: I::ID) -> &T {
        self.entries.get_unchecked(self.indexs.split(key).1 - 1)
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, key: I::ID) -> &mut T {
        self.entries.get_unchecked_mut(self.indexs.split(key).1 - 1)
    }

    #[inline(always)]
    pub fn insert(&mut self, val: T) -> I::ID {
        let key = self.next;
        if key == self.entries.len() {
            if key == self.entries.capacity() {
                self.reserve(1);
            }
            self.next = key + 1;
            unsafe{
                write(self.entries.as_mut_ptr().add(key), val);
                self.entries.set_len(key + 1);
            };
        } else {
            self.next = unsafe{*(&mut self.entries[key] as *mut T as usize as *mut usize)};
            unsafe{write(self.entries.as_mut_ptr().add(key), val)};
        }
        self.len += 1;
        let version = self.indexs.set_true(key);
        self.indexs.merge(version, key + 1)
    }

    #[inline(always)]
    pub fn remove(&mut self, key: I::ID) -> Option<T> {
        let (v, k) = self.indexs.split(key);
        let key = k - 1;
        if key >= self.entries.len() || !self.indexs.set_false(key, v) {
            return None;
        }
        self.len -= 1;
        unsafe {
            let v = self.entries.get_unchecked_mut(key);
            let r = transmute_copy(v);
            *(v as *mut T as usize as *mut usize) = self.next;
            self.next = key;
            Some(r)
        }
    }

    #[inline(always)]
    pub unsafe fn replace(&mut self, key: I::ID, value: T) -> T {
        replace(self.entries.get_unchecked_mut(self.indexs.split(key).1 - 1), value)
    }

    pub fn contains(&self, key: I::ID) -> bool {
        let (v, k) = self.indexs.split(key);
        !(k == 0 || k > self.entries.len() || self.indexs.version(k - 1) != v)
    }

    fn clear_entries(&mut self){
        if needs_drop::<T>() {
            let mut count = self.len;
            if count > 0 {
                let mut cur = self.indexs.prev_true(self.entries.len());
                while count > 0 && cur.0 > 0 {
                    unsafe{ self.entries.set_len(cur.1 + 1)};
                    self.entries.pop();
                    count -= 1;
                    cur = self.indexs.prev_true(cur.1);
                }
            }
        }
        unsafe{ self.entries.set_len(0)};
    }
}

impl<T, I: VerIndex> Map for Slab<T, I> {
	type Key = I::ID;
	type Val = T;
    #[inline]
    fn len(&self) -> usize{
        self.len()
    }
    #[inline]
    fn get(&self, key: &Self::Key) -> Option<&T> {
        self.get(*key)
    }
    #[inline]
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut T> {
        self.get_mut(*key)
    }
    #[inline]
    fn contains(&self, key: &Self::Key) -> bool{
        self.contains(*key)
    }
    #[inline]
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &T{
        self.get_unchecked(*key)
    }
    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut T{
        self.get_unchecked_mut(*key)
    }
    #[inline]
    fn insert(&mut self, key: Self::Key, val: T) -> Option<T>{
        match self.get_mut(key) {
            Some(r) => Some(replace(r, val)),
            None => None
        }
    }
    #[inline]
    fn remove(&mut self, key: &Self::Key) -> Option<T> {
        self.remove(*key)
    }
    #[inline]
    fn clear(&mut self) {
        self.clear()
    }
}

impl<T, I: VerIndex> IdAllocater<T> for Slab<T, I> {
    type ID = I::ID;
    #[inline(always)]
    fn alloc(&mut self, val: T) -> Self::ID {
        self.insert(val)
    }
}

impl<T, I: VerIndex> Drop for Slab<T, I>{
    fn drop(&mut self) {
        self.clear_entries();
    }
}

impl<'a, T: 'a, I: VerIndex + 'a> IntoIterator for &'a Slab<T, I> {
    type Item = (I::ID, &'a T);
    type IntoIter = SlabIter<'a, T, I>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: 'a, I: VerIndex + 'a> IntoIterator for &'a mut Slab<T, I> {
    type Item = (I::ID, &'a mut T);
    type IntoIter = SlabIterMut<'a, T, I>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, I: VerIndex> Debug for Slab<T, I> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        let mut s = String::from("[");
        for v in self.iter(){
            s += &format!("({:?} {:?}),", v.0, v.1);
        }
        s += "]";
        write!(fmt,
               "Slab {{ len: {}, cap: {}, entries_len:{}, next: {}, entries: {} }}",
               self.len,
               self.entries.capacity(),
               self.entries.len(),
               self.next,
               s)
    }
}

pub struct SlabIter<'a, T: 'a, I: VerIndex + 'a> {
    entries: &'a Vec<T>,
    indexs: &'a I,
    len: usize,
    cur_index: (usize, usize),
    cur_len: usize,
}

pub struct SlabIterMut<'a, T: 'a, I: VerIndex + 'a> {
    entries: &'a mut Vec<T>,
    indexs: &'a I,
    len: usize,
    cur_index: (usize, usize),
    cur_len: usize,
}

impl<'a, T: 'a, I: VerIndex + 'a> Debug for SlabIter<'a, T, I> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        fmt.debug_struct("Iter")
            .field("curr", &self.cur_index)
            .field("remaining", &self.cur_len)
            .finish()
    }
}

impl<'a, T: 'a, I: VerIndex + 'a> Debug for SlabIterMut<'a, T, I> where T: Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FResult {
        fmt.debug_struct("IterMut")
            .field("curr", &self.cur_index)
            .field("remaining", &self.cur_len)
            .finish()
    }
}


// ===== Iter =====

impl<'a, T, I: VerIndex + 'a> Iterator for SlabIter<'a, T, I> {
    type Item = (I::ID, &'a T);

    fn next(&mut self) -> Option<(I::ID, &'a T)> {
        if self.cur_index.0 == 0 {
            return None;
        }
        let r = Some((self.indexs.merge(self.cur_index.0, self.cur_index.1 + 1), unsafe{self.entries.get_unchecked(self.cur_index.1)}));
        self.cur_len += 1;
        if self.cur_len < self.len {
            self.cur_index = self.indexs.next_true(self.cur_index.1);
        }else{
            self.cur_index.0 = 0;
        }
        r
    }
}

// ===== IterMut =====
impl<'a, T, I: VerIndex + 'a> Iterator for SlabIterMut<'a, T, I> {
    type Item = (I::ID, &'a mut T);

    fn next(&mut self) -> Option<(I::ID, &'a mut T)> {
        if self.cur_index.0 == 0 {
            return None;
        }
        let r = Some((self.indexs.merge(self.cur_index.0, self.cur_index.1 + 1), unsafe{&mut *(self.entries.get_unchecked_mut(self.cur_index.1) as *mut T)}));
        self.cur_len += 1;
        if self.cur_len < self.len {
            self.cur_index = self.indexs.next_true(self.cur_index.1);
        }else{
            self.cur_index.0 = 0;
        }
        r
    }
}

#[cfg(test)]
extern crate time;

#[test]
fn test(){
    use ver_index::bit::BitIndex;
    let mut slab: Slab<u64, BitIndex> = Slab::default();
    for i in 1..71{
        slab.insert(i);
    }
    println!("!!!------{:?}", slab);
    slab.remove(70);
    println!("r 70------{:?}", slab);

    slab.remove(30);
    println!("r 30------{:?}", slab);

    slab.remove(31);
    println!("r 31------{:?}", slab);

    slab.remove(69);
    println!("r 69------{:?}", slab);


    slab.insert(70);
    println!("i 70------{:?}", slab);

    slab.insert(60);
    println!("i 60------{:?}", slab);

    slab.insert(30);
    println!("i 31------{:?}", slab);

    slab.insert(31);
    println!("i 30------{:?}", slab);

    slab.insert(71);
    println!("i 71------{:?}", slab);

    assert_eq!(slab.contains(0), false);
    assert_eq!(slab.contains(1), true);
    assert_eq!(slab.contains(71), true);
    assert_eq!(slab.contains(72), false);

    assert_eq!(slab.get(0), None);
    assert_eq!(slab.get(1), Some(&1));
    assert_eq!(slab.get(50), Some(&50));
    assert_eq!(slab.get(70), Some(&31));
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

    assert_eq!(unsafe{slab.get_unchecked_mut(31)}, &mut 60);
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
    use ver_index::bit::BitIndex;
    let mut slab: Slab<u64, BitIndex> = Slab::default();
    for i in 1..71{
        slab.insert(i);
    }
    println!("slab ------{:?}", slab);
}

#[test]
fn test_eff(){
    extern crate time;
    use time::now_millisecond;
    use ver_index::bit::BitIndex;
    let mut slab: Slab<u64, BitIndex> = Slab::default();
    let time = now_millisecond();
    for i in 0..1000000{
        slab.insert(i);
    }
    println!("alloc time-----------------------------------------------{}", now_millisecond() - time);
    slab.clear();
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


use std::mem::{replace};
use std::fmt::{Debug, Formatter, Result as FResult};
use std::ops::{Index, IndexMut};
// use std::iter::IntoIterator;
// use std::ops::Drop;
// use std::ptr::write;

use ::Map;
// TODO 改成类似slab的写法，用单独的vec<usize>的位记录是否为空。现在这种写法太费内存了
pub struct VecMap<T> {
    entries: Vec<Option<T>>,// Chunk of memory
    len: usize,// Number of Filled elements currently in the slab
}

impl<T> Default for VecMap<T> {
    fn default() -> Self {
        VecMap::new()
    }
}

impl<T> VecMap<T> {

    pub fn new() -> Self {
        VecMap::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> VecMap<T> {
        VecMap {
            entries: Vec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    // pub fn reserve(&mut self, additional: usize) {
    //     if self.capacity() - self.len >= additional {
    //         return;
    //     }
    //     let need_add = self.len + additional - self.entries.len();
    //     self.entries.reserve(need_add);
    //     self.vacancy_sign.reserve(need_add/usize_size());
    // }

    
    // pub fn reserve_exact(&mut self, additional: usize) {
    //     if self.capacity() - self.len >= additional {
    //         return;
    //     }
    //     let need_add = self.len + additional - self.entries.len();
    //     self.entries.reserve_exact(need_add);
    //     self.vacancy_sign.reserve(need_add/usize_size());
    // }

    // pub fn shrink_to_fit(&mut self) {
    //     self.entries.shrink_to_fit();
    //     self.vacancy_sign.shrink_to_fit();
    // }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.len = 0;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // pub fn iter(&self) -> SlabIter<T> {
    //     SlabIter {
    //         entries: &self.entries,
    //         signs: &self.vacancy_sign,
    //         curr_index: 0,
    //         len: self.len,
    //         curr_len: 0,
    //     }
    // }

    // pub fn iter_mut(&mut self) -> SlabIterMut<T> {
    //     SlabIterMut {
    //         entries: &mut self.entries as *mut Vec<T>,
    //         signs: &mut self.vacancy_sign,
    //         curr_index: 0,
    //         len: self.len,
    //         curr_len: 0,
    //     }
    // }

    pub unsafe fn replace(&mut self, index: usize, value: T) -> T {
        replace(&mut self.entries[index - 1], Some(value)).unwrap()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index == 0 || index > self.entries.len(){
            return None;
        }
        match &self.entries[index - 1] {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index == 0 || index > self.entries.len(){
            return None;
        }
        match &mut self.entries[index - 1] {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        self.entries[index - 1].as_ref().unwrap()
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.entries[index - 1].as_mut().unwrap()
    }

    pub fn insert(&mut self, index:usize, val: T) -> Option<T>{
        let index = index - 1;
        let len = self.entries.len();
        if index >= len {
            for _ in 0..index - len  {
                self.entries.push(None);
            }
            self.entries.push(Some(val));
            self.len += 1;
            None
        }else {
            let r = replace(&mut self.entries[index], Some(val));
            if r.is_none(){
                self.len += 1;
            }
            r
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index > self.entries.len() {
            return None;
        }
        match replace(&mut self.entries[index - 1], None) {
            Some(v) => {
                self.len -= 1;
                Some(v)
            },
            None => None,
        }
    }

    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        self.len -= 1;
        replace(&mut self.entries[index - 1], None).unwrap()
    }

    pub fn contains(&self, index: usize) -> bool {
        if index == 0 || index > self.entries.len(){
            return false;
        }
        match &self.entries[index - 1] {
            Some(_v) => true,
            None => false,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

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
}

impl<T> Index<usize> for VecMap<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        unsafe{ self.get_unchecked(index) }
    }
}

impl<T> IndexMut<usize> for VecMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        unsafe{ self.get_unchecked_mut(index) }
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
// use time::now_millis;
#[test]
fn test(){
    let mut map: VecMap<u64> = VecMap::new();
    for i in 1..71{
        map.insert(i as usize, i);
        println!("map------{:?}", map);
    }

    unsafe {map.remove(30)};
    println!("r 30------{:?}", map);

    unsafe {map.remove(31)};
    println!("r 31------{:?}", map);

    unsafe {map.remove(69)};
    println!("r 69------{:?}", map);

    unsafe {map.remove(70)};
    println!("r 70------{:?}", map);

    assert_eq!(map.contains(0), false);
    assert_eq!(map.contains(1), true);
    assert_eq!(map.contains(71), true);
    assert_eq!(map.contains(72), false);

    assert_eq!(map.get(0), None);
    assert_eq!(map.get(1), Some(&1));
    assert_eq!(map.get(50), Some(&50));
    assert_eq!(map.get(70), Some(&70));
    assert_eq!(map.get(72), None);


    assert_eq!(map.get_mut(0), None);
    assert_eq!(map.get_mut(64), Some(&mut 64));
    assert_eq!(map.get_mut(30), Some(&mut 30));
    assert_eq!(map.get_mut(20), Some(&mut 20));
    assert_eq!(map.get_mut(75), None);

    assert_eq!(unsafe{map.get_unchecked(2)}, &2);
    assert_eq!(unsafe{map.get_unchecked(9)}, &9);
    assert_eq!(unsafe{map.get_unchecked(55)}, &55);
    assert_eq!(unsafe{map.get_unchecked(60)}, &60);

    assert_eq!(unsafe{map.get_unchecked_mut(31)}, &mut 31);
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
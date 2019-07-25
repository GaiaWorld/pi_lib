/// A double-ended queue implemented with a link and map.
///
/// Support for appending to the back and popping from the back.
/// Support for prepending to the front and popping from the front.
/// Supports quick deletion of specified elements
/// 
/// 
use std::fmt::{Debug, Formatter, Result as FResult};

use slab::Slab;
use ver_index::VerIndex;
use deque::{ Deque, Direction, Node, Iter as DIter };

pub struct SlabDeque<T, I:VerIndex<ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync>{
    slab: Slab<Node<T, ID>, I>,
    deque: Deque<T, Slab<Node<T, ID>, I>, ID>,
}

impl<T, I:VerIndex<ID=ID> + Default,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Default for SlabDeque<T, I, ID> {
    fn default() -> Self {
        Self {
            slab: Slab::default(),
            deque: Deque::default(),
        }
    }
}

impl<T, I:VerIndex<ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> SlabDeque<T, I, ID> {
    /// Append an element to the SlabDeque. return a index
    #[inline]
    pub fn push(&mut self, elem: T, direct: Direction) -> ID {
        self.deque.push(elem, direct, &mut self.slab)
    }
    #[inline]
    pub fn push_back(&mut self, elem: T) -> ID {
        self.deque.push_back(elem, &mut self.slab)
    }

    /// Prepend an element to the SlabDeque. return a index
    #[inline]
    pub fn push_front(&mut self, elem: T) -> ID{
        self.deque.push_front(elem, &mut self.slab)
    }

    /// Removes the first or last element from the SlabDeque and returns it, or None if it is empty.
    pub fn pop(&mut self, direct: Direction) -> Option<T> {
        self.deque.pop(direct, &mut self.slab)
    }

    /// Removes the first element from the SlabDeque and returns it, or None if it is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.deque.pop_front(&mut self.slab)
    }

    /// Removes the last element from the SlabDeque and returns it, or None if it is empty.
    pub fn pop_back(&mut self) -> Option<T> {
        self.deque.pop_back(&mut self.slab)
    }

    ///Removes and returns the element at index from the SlabDeque.
    pub fn remove(&mut self, index: ID) -> Option<T> {
        self.deque.remove(index, &mut self.slab)
    }

    //clear SlabDeque
    pub fn clear(&mut self) {
        self.slab.clear();
        self.deque = Deque::default();
    }

    //clear SlabDeque
    pub fn len(&self) -> usize {
        self.slab.len()
    }

    pub fn iter(&mut self) -> Iter<T, I, ID> {
        Iter{
            d_iter: self.deque.iter(&self.slab),
        }
    }
}

impl<T: Debug, I:VerIndex<ID=ID> + Default,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Debug for SlabDeque<T, I, ID> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        f.debug_struct("SlabDeque")
            .field("slab", &self.slab)
            .field("first", &self.deque)
            .finish()
    }
}

pub struct Iter<'a, T: 'a, I:VerIndex<ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> {
    d_iter: DIter<'a, T, Slab<Node<T, ID>, I>, ID>,
}


impl<'a, T, I:VerIndex<ID=ID>,  ID: Copy + Debug + PartialEq + Default + Send + Sync> Iterator for Iter<'a, T, I, ID> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.d_iter.next()
    }
}



// #[cfg(test)]
// use time::now_millisecond;

// #[cfg(test)]
// use std::collections::{VecDeque, HashMap};

// #[test]
// fn test(){
//     use ver_index::bit::BitIndex;
// 	let mut fast_deque: SlabDeque<u32, BitIndex, usize> = SlabDeque::default();
   
//     let i = fast_deque.push_back(1);
//     fast_deque.remove(i);
//     println!("-----{}", fast_deque.len());

// }


#[test]
fn test_effict(){
    use std::collections::VecDeque;
    use std::collections::HashMap;
    use ver_index::bit::BitIndex;
    use time::now_millisecond;
	let mut fast_deque: SlabDeque<u32, BitIndex, usize> = SlabDeque::default();
    let max = 10;

    let now = now_millisecond();
    for i in 0..max {
        fast_deque.push_back(i);
    }

    println!("push back time{}",  now_millisecond() - now);
    println!("1---{:?}",  fast_deque);

    let now = now_millisecond();
    let mut ii= 0;
    for i in 0..max {
        match fast_deque.pop_back() {
            Some(_) => (),
            _ => {ii = i; break}
        }
        //println!("i---------------------{}", i);
        // let index: usize = ((5- i)/2) as usize;
        // println!("index---------------------{}", index);
        // assert_eq!(fast_SlabDeque.remove(&(index + 1)).unwrap(), index as u32);
        //assert_eq!(fast_SlabDeque.pop_front().unwrap(), i);
    }
    println!("pop_back time{} {}",  now_millisecond() - now, ii);
    println!("2---{:?}",  fast_deque);

    let mut vec_deque = VecDeque::new();
    let now = now_millisecond();
    for i in 0..max {
        vec_deque.push_back(i);
    }
    println!("push vec front time{}",  now_millisecond() - now);

    let now = now_millisecond();
    for _ in 0..max{
        vec_deque.pop_back();
    }
    println!("pop vec front time{}",  now_millisecond() - now);

    let mut map = HashMap::new();
    let now = now_millisecond();
    for i in 0..max {
        map.insert(i, i);
    }
    println!("insert HashMap front time{}",  now_millisecond() - now);

    let now = now_millisecond();
    for i in 0..max {
        assert_eq!(map.remove(&i).unwrap(), i);
    }
    println!("remove HashMap front time{}",  now_millisecond() - now);

}
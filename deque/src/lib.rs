
//! 双端队列
//! 支持从队列头部添加或弹出
//! 支持从队列尾部添加或弹出
//! 与标准库的双端队列相比，本双端队列还支持根据索引快速从任意位置删除和查询，一些时候，可快速删除的双端队列十分有用（例如pi_lib中的task_pool）
//!
//! 特色： 将双端队列本身的逻辑和索引（删除就需要依赖索引）分离，因此，十分容易和其它需要索引的数据结构共享索引。
//! 关于共享索引的意义，请参考：https://github.com/GaiaWorld/pi_lib/tree/master/dyn_uint
//!
//! 选择:
//! - 当你需要使用双端队列，并且你不需要快速从任意位置删除和查询，标准库中的双端队列是一个不错的选择
//! - 当你的部分功能需要使用从任意位置删除和查询，部分功能不需要时，不太建议你同时依赖标准库与本库的双端队列，毕竟会增减应用程序的尺寸
//! 但如果你不在意，你可以这么做！这种情况下，
//! 建议的做法是，总是使用本库或其它的代替品,本库的双端队列性能仅比标准库略低（删除功能也需要一定成本）


#[cfg(test)]
extern crate time;
extern crate slab;

/// 双端队列的核心逻辑模块
pub mod deque;

/// deque模块通常不单独使用，slab_deque模块将deque，为其提供一个Slab实现的索引工厂
/// 当不需要与其它数据结构结合时，你可以使用slab_deque为你提供的双端队列默认实现
pub mod slab_deque;


// use std::fmt::{Debug, Formatter, Result as FResult};

// use slab::Slab;

// pub struct FastDeque<T>{
//     first : usize,
//     last :usize,
//     slab: Slab<Node<T>>,
// }

// impl<T> FastDeque<T> {
//     pub fn new() -> Self {
//         Self {
//             first: 0,
//             last: 0,
//             slab: Slab::new(),
//         }
//     }

//     /// Append an element to the FastDeque. return a index
//     pub fn push_back(&mut self, elem: T) -> usize {
//         if self.last == 0 {
//             let index = self.slab.insert(Node::new(elem, 0, 0));
//             self.last = index;
//             self.first = index;
//             index
//         }else {
//             let index = self.slab.insert(Node::new(elem, self.last, 0));
//             unsafe{self.slab.get_unchecked_mut(self.last).next = index;}
//             self.last = index;
//             index
//         }
//     }

//     /// Prepend an element to the FastDeque. return a index
//     pub fn push_front(&mut self, elem: T) -> usize{
//         if self.first == 0 {
//             let index = self.slab.insert(Node::new(elem, 0, 0));
//             self.last = index;
//             self.first = index;
//             index
//         }else {
//             let index = self.slab.insert(Node::new(elem, 0, self.first));
//             unsafe{self.slab.get_unchecked_mut(self.first).pre = index;}
//             self.first = index;
//             index
//         }
//     }

//     /// Removes the first element from the FastDeque and returns it, or None if it is empty.
//     pub fn pop_front(&mut self) -> Option<T> {
//         if self.first == 0{
//             None
//         } else {
//             let node = self.slab.remove(self.first);
//             self.first = node.next;
//             if self.first == 0 {
//                 self.last = 0;
//             }
//             Some(node.elem)
//         }
//     }

//     /// Removes the last element from the FastDeque and returns it, or None if it is empty.
//     pub fn pop_back(&mut self) -> Option<T> {
//         if self.last == 0{
//             None
//         } else {
//             let node = self.slab.remove(self.last);
//             self.last = node.pre;
//             if self.last == 0 {
//                 self.first = 0;
//             }
//             Some(node.elem)
//         }
//     }

//     ///Removes and returns the element at index from the FastDeque.
//     pub fn remove(&mut self, index: usize) -> T {
//         let node = self.slab.remove(index);
//         match (node.pre, node.next) {
//             (0, 0) => {
//                 //如果该元素既不存在上一个元素，也不存在下一个元素， 则设置队列的头部None， 则设置队列的尾部None
//                 self.first = 0;
//                 self.last = 0;
//             },
            
//             (_, 0) => {
//                 //如果该元素存在上一个元素，不存在下一个元素， 则将上一个元素的下一个元素设置为None, 并设置队列的尾部为该元素的上一个元素
//                 unsafe{ self.slab.get_unchecked_mut(node.pre).next = 0};
//                 self.last = node.pre;
//             },
//             (0, _) => {
//                 //如果该元素不存在上一个元素，但存在下一个元素， 则将下一个元素的上一个元素设置为None, 并设置队列的头部为该元素的下一个元素
//                 unsafe{ self.slab.get_unchecked_mut(node.next).pre = 0};
//                 self.first = node.next;
//             },
//             (_, _) => {
//                 //如果该元素既存在上一个元素，也存在下一个元素， 则将上一个元素的下一个元素设置为本元素的下一个元素, 下一个元素的上一个元素设置为本元素的上一个元素
//                 unsafe{ self.slab.get_unchecked_mut(node.pre).next = node.next};
//                 unsafe{ self.slab.get_unchecked_mut(node.next).pre = node.pre};
//             },
            
//         }
//         node.elem
//     }

//     ///Removes and returns the element at index from the FastDeque.
//     pub fn try_remove(&mut self, index: usize) -> Option<T> {
//         match self.slab.contains(index){
//             true => Some(self.remove(index)),
//             false => None,
//         }
//     }

//     //clear FastDeque
//     pub fn clear(&mut self) {
//         self.slab.clear();
//         self.first = 0;
//         self.last = 0;
//     }

//     //clear FastDeque
//     pub fn len(&self) -> usize {
//         self.slab.len()
//     }
// }

// impl<T: Debug> Debug for FastDeque<T> {
//     fn fmt(&self, f: &mut Formatter) -> FResult {
//         f.debug_struct("FastDeque")
//             .field("slab", &self.slab)
//             .field("first", &self.first)
//             .field("last", &self.last)
//             .finish()
//     }
// }

// struct Node<T>{
//     pub elem: T,
//     pub next: usize,
//     pub pre: usize,
// }

// impl<T> Node<T>{
//     fn new(elem: T, pre: usize, next: usize) -> Node<T>{
//         Node{
//             elem,
//             pre,
//             next,
//         }
//     }
// }

// impl<T: Debug> Debug for Node<T> {
//     fn fmt(&self, f: &mut Formatter) -> FResult {
//         f.debug_struct("Node")
//             .field("elem", &self.elem)
//             .field("pre", &self.pre)
//             .field("next", &self.next)
//             .finish()
//     }
// }


// #[cfg(test)]
// use time::now_millis;

// #[cfg(test)]
// use std::collections::{VecDeque, HashMap};

// #[test]
// fn test(){
// 	let mut fast_deque: FastDeque<u32> = FastDeque::new();
   
//     let i = fast_deque.push_back(1);
//     fast_deque.remove(i);
//     println!("-----{}", fast_deque.len());

// }


// #[test]
// fn test_effict(){
// 	let mut fast_deque: FastDeque<u32> = FastDeque::new();
//     let max = 100000;

//     let now = now_millis();
//     for i in 0..max {
//         fast_deque.push_back(i);
//     }

//     println!("push back time{}",  now_millis() - now);

//     let now = now_millis();
//     for i in 0..max {
//         fast_deque.pop_back().unwrap();
//         //println!("i---------------------{}", i);
//         // let index: usize = ((5- i)/2) as usize;
//         // println!("index---------------------{}", index);
//         // assert_eq!(fast_deque.remove(&(index + 1)).unwrap(), index as u32);
//         //assert_eq!(fast_deque.pop_front().unwrap(), i);
//     }
//     println!("pop_back time{}",  now_millis() - now);

//     let mut vec_deque = VecDeque::new();
//     let now = now_millis();
//     for i in 0..max {
//         vec_deque.push_back(i);
//     }
//     println!("push vec front time{}",  now_millis() - now);

//     let now = now_millis();
//     for _ in 0..max{
//         vec_deque.pop_back();
//     }
//     println!("pop vec front time{}",  now_millis() - now);

//     let mut map = HashMap::new();
//     let now = now_millis();
//     for i in 0..max {
//         map.insert(i, i);
//     }
//     println!("insert HashMap front time{}",  now_millis() - now);

//     let now = now_millis();
//     for i in 0..max {
//         assert_eq!(map.remove(&i).unwrap(), i);
//     }
//     println!("remove HashMap front time{}",  now_millis() - now);

// }
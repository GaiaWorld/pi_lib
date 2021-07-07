//! 功能：
//! * 定义了一个triat：`Map`
//! * 定义了一个数据结构：`VecMap`，并为`VecMap`实现了`Map<K=usize,V=T>`
//! * 定义了数据结构：`HashMap`，并为`HashMap`实现了`Map`

#![feature(rustc_private)]
#![feature(integer_atomics)]
#![feature(asm,box_syntax,box_patterns)]
#![feature(core_intrinsics)]
#![feature(generators, generator_trait)]
#![feature(associated_type_defaults)]
#![feature(exclusive_range_pattern)]
#![feature(trait_alias)]
#![feature(nll)]
#[warn(unreachable_patterns)]

#[allow(dead_code,unused_variables,non_snake_case,unused_parens,unused_assignments,unused_unsafe,unused_imports)]


extern crate hash;
#[cfg(test)]
extern crate time;

pub mod vecmap;
pub mod hashmap;

/// Map接口定义
pub trait Map{
	type Key;
	type Val;
	fn len(&self) -> usize;
	fn with_capacity(capacity: usize) -> Self;
    fn capacity(&self) -> usize;
    fn mem_size(&self) -> usize;
    fn contains(&self, key: &Self::Key) -> bool;
    fn get(&self, key: &Self::Key) -> Option<&Self::Val>;
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val>;
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val;
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val;
    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val;
    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val>;
}

#![feature(rustc_private)]
#![feature(const_vec_new)]
#![feature(integer_atomics)]
#![feature(asm,box_syntax,box_patterns)]
#![feature(core_intrinsics)]
#![feature(generators, generator_trait)]
#![feature(associated_type_defaults)]
#![feature(exclusive_range_pattern)]
#![feature(box_into_raw_non_null)]
#![feature(trait_alias)]
#![feature(const_fn)]
#![feature(nll)]
#[warn(unreachable_patterns)]

#[allow(dead_code,unused_variables,non_snake_case,unused_parens,unused_assignments,unused_unsafe,unused_imports)]

#[cfg(test)]
extern crate time;

pub mod vecmap;

use std::marker::PhantomData;

pub trait Map {
	type Key;
	type Val;
    fn len(&self) -> usize;
    fn contains(&self, key: &Self::Key) -> bool;
    fn get(&self, key: &Self::Key) -> Option<&Self::Val>;
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val>;
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val;
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val;
    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val>;
    fn clear(&mut self);
}

#[derive(Clone)]
pub struct Empty<K, T>(PhantomData<(K, T)>);
impl<K, T> Default for Empty<K, T> {
    fn default() -> Self {
        Empty(PhantomData)
    }
}

impl<K, T> Map for Empty<K, T> {
	type Key = K;
	type Val = T;
    #[inline]
    fn len(&self) -> usize {
        0
    }
    #[inline]
    fn contains(&self, _id: &Self::Key) -> bool {
        false
    }
    #[inline]
    fn get(&self, _id: &Self::Key) -> Option<&T> {
        None
    }
    #[inline]
    fn get_mut(&mut self, _id: &Self::Key) -> Option<&mut T> {
        None
    }
    #[inline]
    unsafe fn get_unchecked(&self, _id: &Self::Key) -> &T {
        panic!("Empty, invalid method");
    }
    #[inline]
    unsafe fn get_unchecked_mut(&mut self, _id: &Self::Key) -> &mut T {
        panic!("Empty, invalid method");
    }
    #[inline]
    fn insert(&mut self, _id: Self::Key, _value: T) -> Option<T> {
        None
    }
    #[inline]
    fn remove(&mut self, _id: &Self::Key) -> Option<T> {
        None
    }
    #[inline]
    fn clear(&mut self) {
    }
}

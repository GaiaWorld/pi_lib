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
#![feature(fnbox)]
#[warn(unreachable_patterns)]

#[allow(dead_code,unused_variables,non_snake_case,unused_parens,unused_assignments,unused_unsafe,unused_imports)]

pub mod vecmap;


pub trait Map{
	type Key;
	type Val;
    fn len(&self) -> usize;
    fn contains(&self, key: &Self::Key) -> bool;
    fn get(&self, key: &Self::Key) -> Option<&Self::Val>;
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val>;
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val;
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val;
    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val;
    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val>;
}

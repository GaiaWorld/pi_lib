//! 事件监听器及监听器列表， 为ECS系统服务的

extern crate share;

use std::{
    ops::{Deref, DerefMut},
    default::Default,
};
// use im::vector::Vector;
use share::Share;
// use std::vec::Vector;

/// 监听器定义
pub trait Listener<E> {
    /// 监听事件
    fn listen(&self, e: &E);
}

/// 闭包函数的监听器
pub struct FnListener<E>(pub Share<dyn Fn(&E)>);

unsafe impl<E> Sync for FnListener<E> {}
unsafe impl<E> Send for FnListener<E> {}

impl<E> Listener<E> for FnListener<E> {
    fn listen(&self, e: &E) {
        self.0(e)
    }
}
impl<E> PartialEq for FnListener<E> {
    fn eq(&self, other: &Self) -> bool {
        Share::ptr_eq(&self.0, &other.0)
    }
}
impl<E> Clone for FnListener<E> {
    fn clone(&self) -> Self {
        FnListener(self.0.clone())
    }
}

pub type FnListeners<E> = Listeners<FnListener<E>>;

/// 监听器列表
#[derive(Clone)]
pub struct Listeners<T: Clone> (Vec<T>);

impl<T: Clone + PartialEq> Listeners<T> {
    /// 获取监听器列表的内存大小
    pub fn mem_size(&self) -> usize {
        self.0.len() * std::mem::size_of::<T>()
    }
    /// 移除一个监听器， 要求该监听器实现PartialEq
    pub fn delete(&mut self, listener: &T) -> bool {
        for i in 0..self.0.len() {
            if &self.0[i] == listener {
                self.0.swap_remove(i);
                return true
            }
        }
        return false;
    }
}
impl<T: Clone + Listener<E>, E> Listener<E> for Listeners<T> {
    fn listen(&self, e: &E) {
		if self.0.len() > 0 {
			for l in self.0.iter() {
				// let time = std::time::Instant::now();
				l.listen(e);
				// println!("listen time----------{:?}", std::time::Instant::now() - time);
			}
		}
    }
}

impl<T: Clone> Default for Listeners<T> {
    fn default() -> Self{
        Listeners(Vec::new())
    }
}

impl<T: Clone> Deref for Listeners<T> {
    type Target=Vec<T>;
    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl<T: Clone> DerefMut for Listeners<T> {
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.0
    }
}

//! 写时复制的列表
//! TODO (最初是atom库需要，后面atom库内部实现了一个简单的cowlist，本库就此闲置，未完全实现)

#![feature(core_intrinsics)] 

use std::sync::Arc;
use std::marker::PhantomData;

pub struct CowList<T> {
	next:Option<Arc<CowList<T>>>,
	value:Arc<T>,
}

impl<T> Clone for CowList<T>{
	fn clone(&self) -> Self{
		CowList{
			next: self.next.clone(),
			value: self.value.clone(),
		}
	}
}

impl<T> CowList<T>{
	pub fn new(ele: T) -> Self {
		CowList{
			next: None,
			value: Arc::new(ele)
		}
	}


	pub fn push(&mut self, ele: T) -> CowList<T> {
		CowList{
			next: Some(Arc::new(self.clone())),
			value: Arc::new(ele),
		}
	}


	pub fn iter(&self) -> Iter<T>{
		Iter{
			head: Some(Arc::new(self.clone())),
			marker: PhantomData,
		}
	}
}

pub struct Iter<'a, T: 'a> {
    head: Option<Arc<CowList<T>>>,
	marker: PhantomData<&'a CowList<T>>,
    //tail: Node<T>,
}

impl<'a, T> Iterator for Iter<'a, T>{
	type Item = &'a T;
	fn next(&mut self) -> Option<&'a T>{
		let node = match self.head {
			Some(ref node) => unsafe{
				let node = &*Arc::into_raw(node.clone());
				node
			},
			None => return None,
		};
		self.head = node.next.clone();
		Some(node.value.as_ref())
	}
}

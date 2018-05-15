use std::sync::Arc;
use core::marker::PhantomData;


// struct Node<T> {
// 	next:Option<NonNull<Arc<Node<T>>>>,
// 	value:T,
// }

// impl<T> Node<T>{
// 	pub fn new(ele: T) -> Node<T>{
// 		Node{
// 			next: None,
// 			value: ele
// 		}
// 	}
// }

// pub struct CowList<T> {
// 	root: Option<NonNull<Arc<Node<T>>>>,
// 	len: usize,
// 	marker: PhantomData<Box<Node<T>>>
// }

// impl<T> CowList<T>{
// 	pub fn new() -> Self {
// 		CowList{
// 			root:None,
// 			len: 0,
// 			marker:PhantomData
// 		}
// 	}

// 	fn push_node(&mut self, mut node: Box<Arc<Node<T>>>) -> CowList<T> {
//         unsafe {
//             node.next = self.root;
//             let node = Some(Box::into_raw_non_null(node));
//             CowList{
// 				root: node,
// 				len: self.len + 1,
// 				marker:PhantomData
// 			}
//         }
//     }

// 	pub fn push(&mut self, elt: T) -> CowList<T> {
//         self.push_node(box Arc::new(Node::new(elt)))
//     }

// 	pub fn len(&self) -> usize {
//         self.len
//     }

// 	pub fn iter(&self) -> Iter<T>{
// 		Iter{
// 			head: self.root.clone(),
// 			marker: &PhantomData,
// 		}
// 	}

// 	// pub fn iter_mut(&mut self) -> IterMut<T>{
// 	// 	IterMut{
// 	// 		head:self.root.clone(),
// 	// 		marker: &mut PhantomData,
// 	// 	}
// 	// }
// }

// pub struct Iter<'a, T: 'a> {
//     head: Option<NonNull<Arc<Node<T>>>>,
// 	marker: &'a PhantomData<Node<T>>,
//     //tail: Node<T>,
// }

// impl<'a, T> Iterator for Iter<'a, T>{
// 	type Item = &'a T;
// 	fn next(&mut self) -> Option<&'a T>{
// 		if self.head.is_some(){
// 			self.head.map(|node|unsafe {
//                 // Need an unbound lifetime to get 'a
//                 let node = &mut *node.as_ptr();
//                 self.head = node.next;
//                 &node.value
//             })
// 		}else{
// 			None
// 		}
// 	}
// }

// pub struct IterMut<'a, T: 'a> {
//     head: Option<NonNull<Arc<Node<T>>>>,
// 	marker: &'a mut PhantomData<Node<T>>,
//     //tail: Node<T>,
// }

// impl<'a, T> Iterator for IterMut<'a, T>{
// 	type Item = &'a mut T;
// 	fn next(&mut self) -> Option<&'a mut T>{
// 		if self.head.is_some(){
// 			self.head.map(|node| unsafe {
//                 // Need an unbound lifetime to get 'a
// 				let node = &mut *node.as_ptr();
//                 self.head = node.next;
//                 &mut node.value
//             })
// 		}else{
// 			None
// 		}
// 	}
// }


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
		match self.head {
			Some(ref node) => unsafe{
				let node = &*Arc::into_raw(node.clone());
				self.head = node.next.clone();
				Some(node.value.as_ref())
			},
			None => {None},
		}
	}
}



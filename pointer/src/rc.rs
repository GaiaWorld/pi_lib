pub use slab::Slab;


// TODO 支持 Arc， 需要对slab加读写锁
#[macro_export]
macro_rules! rc{
    ($(#[$attr:meta])* $name: ident, $elem: ty, $container: ident) => {
        lazy_static! {
            pub static ref $container: usize = Box::into_raw(Box::new($crate::rc::Slab::<$elem>::new())) as usize;
        }

        $(#[$attr])*
        pub struct $name{
            id: usize,
        }

        impl $name {
            pub fn new(value: $elem) -> $name{
                $name{
                    id: <$crate::rc::Slab<$crate::rc::Inner<$elem>>>::insert(unsafe{&mut *(* $container as *mut $crate::rc::Slab<$crate::rc::Inner<$elem>>)}, $crate::rc::Inner{count: 1, value: value}),
                }
            }

            pub fn strong_count(&self) -> usize{
                unsafe{(&mut *(* $container as *mut $crate::rc::Slab<$crate::rc::Inner<$elem>>)).get_unchecked_mut(self.id).count}
            }
        }

        impl std::ops::Deref for $name {
            type Target = $elem;
            fn deref(&self) -> &Self::Target {
                unsafe{&(&*(* $container as *const $crate::rc::Slab<$crate::rc::Inner<$elem>>)).get_unchecked(self.id).value}
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe{&mut (&mut *(* $container as *mut $crate::rc::Slab<$crate::rc::Inner<$elem>>)).get_unchecked_mut(self.id).value}
            }
        }

        impl Clone for $name {
            fn clone(&self) -> $name {
                unsafe{(&mut *(* $container as *mut $crate::rc::Slab<$crate::rc::Inner<$elem>>)).get_unchecked_mut(self.id)}.count += 1;
                $name{
                    id: self.id
                }
            }
        }

        impl std::ops::Drop for $name {
            fn drop(&mut self){
                let count = {
                    let mut inner = unsafe{(&mut *(* $container as *mut $crate::rc::Slab<$crate::rc::Inner<$elem>>)).get_unchecked_mut(self.id)};
                    inner.count -= 1;
                    inner.count
                };
                
                if count == 0{
                    unsafe{(&mut *(* $container as *mut $crate::rc::Slab<$crate::rc::Inner<$elem>>)).remove(self.id)};
                }
            }
        }
    }
}

pub struct Inner<T>{
    pub count: usize,
    pub value: T,
}

#[macro_export]
macro_rules! id{
    ($name: ident, $elem: ty, $container: ident) => {
        lazy_static! {
            pub static ref $container: usize = Box::into_raw(Box::new($crate::rc::Slab::<$elem>::new())) as usize;
        }

        pub struct $name{
            id: usize,
        }

        impl $name {
            pub fn new(value: $elem) -> $name{
                $name{
                    id: <$crate::rc::Slab<$elem>>::insert(unsafe{&mut *(* $container as *mut $crate::rc::Slab<$elem>)}, value),
                }
            }
        }

        impl std::ops::Deref for $name {
            type Target = $elem;
            fn deref(&self) -> &Self::Target {
                unsafe{(&*(* $container as *const $crate::rc::Slab<$elem>)).get_unchecked(self.id)}
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe{(&mut *(* $container as *mut $crate::rc::Slab<$elem>)).get_unchecked_mut(self.id)}
            }
        }

        impl std::ops::Drop for $name {
            fn drop(&mut self){
                unsafe{(&mut *(* $container as *mut $crate::rc::Slab<$elem>)).remove(self.id)};
            }
        }
    }
}

#[cfg(test)]
pub struct A{
    pub x: usize,
    pub y: usize,
}

#[cfg(test)]
rc!(RCA, A, _rc_slab);

#[test]
fn test(){

    let a = RCA::new(A{x: 5, y: 5});
    let _x = a.clone();
    let _x = a.clone();
    let _x = a.clone();
    let _x = a.clone();
    assert_eq!(a.strong_count(), 5);
}

// extern crate slab;
// #[macro_use]
// extern crate lazy_static;

// use std::marker::PhantomData;
// use std::ops::{DerefMut, Deref};
// use std::ops::Drop;

// use slab::{Slab};

// lazy_static! {
// 	pub static ref CCC: usize = Box::into_raw(Box::new(Slab::<usize>::new())) as usize;
// }


// pub struct Rc<T>{
//     id: usize,
//     mark: PhantomData<T>,
// }

// impl Rc<usize> {
//     pub fn new(value: usize) -> Rc<usize>{
//         Rc{
//             id: <Slab<Inner<usize>>>::insert(unsafe{&mut *(*CCC as *mut Slab<Inner<usize>>)}, Inner{count: 1, value: value}),
//             mark: PhantomData,
//         }
//     }

//     pub fn strong_count(&self) -> usize{
//         unsafe{(&mut *(*CCC as *mut Slab<Inner<usize>>)).get_unchecked_mut(self.id).count}.clone()
//     }

//     pub fn free(&mut self){
//         let count = {
//             let mut inner = unsafe{(&mut *(*CCC as *mut Slab<Inner<usize>>)).get_unchecked_mut(self.id)};
//             inner.count -= 1;
//             inner.count
//         };
        
//         if count == 0{
//             unsafe{(&mut *(*CCC as *mut Slab<Inner<usize>>)).remove(self.id)};
//         }
//     }
// }

// impl Rc<usize> {
//     pub fn xx() -> usize{
//         5
//     }
// }

// impl Deref for Rc<usize> {
//     type Target = usize;
//     fn deref(&self) -> &Self::Target {
//         unsafe{&(&*(*CCC as *const Slab<Inner<usize>>)).get_unchecked(self.id).value}
//     }
// }

// impl DerefMut for Rc<usize> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe{&mut (&mut *(*CCC as *mut Slab<Inner<usize>>)).get_unchecked_mut(self.id).value}
//     }
// }

// impl Clone for Rc<usize> {
//     fn clone(&self) -> Rc<usize> {
//         unsafe{(&mut *(*CCC as *mut Slab<Inner<usize>>)).get_unchecked_mut(self.id)}.count += 1;
//         Rc{
//             id: self.id,
//             mark: PhantomData,
//         }
//     }
// }

// type A = Rc<usize>;
// impl Drop for A {
//     fn drop(&mut self){
//         // self.
//         // let count = {
//         //     let mut inner = unsafe{(&mut *(*CCC as *mut Slab<Inner<usize>>)).get_unchecked_mut(self.id)};
//         //     inner.count -= 1;
//         //     inner.count
//         // };
        
//         // if count == 0{
//         //     unsafe{(&mut *(*CCC as *mut Slab<Inner<usize>>)).remove(self.id)};
//         // }
//     }
// }

// struct Inner<T>{
//     count: usize,
//     value: T,
// }


// #[test]
// fn test(){
//     let a = Rc::new(1);
//     let x = a.clone();
//     let x = a.clone();
//     let x = a.clone();
//     let x = a.clone();
//     assert_eq!(a.strong_count(), 5);
// }
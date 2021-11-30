//!
//! Null主要用在其他数据结构中，让值本身支持判断是否空。可以提升内存性能，减少使用Option。 
//!

use std::{any::TypeId, mem};
pub trait Null {
    /// 判断当前值是否空
    fn null() -> Self;
    /// 判断当前值是否空
    fn is_null(&self) -> bool;
}

impl<T> Null for Option<T> {
    #[inline(always)]
    fn null() -> Self {
        None
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        match self {
            Some(_) => false,
            None => true,
        }
    }
}

impl Null for usize {
    #[inline(always)]
    fn null() -> Self {
        usize::max_value()
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == usize::max_value()
    }
}
impl Null for isize {
    #[inline(always)]
    fn null() -> Self {
        isize::min_value()
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == isize::min_value()
    }
}
impl Null for bool {
    #[inline(always)]
    fn null() -> Self {
        false
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self
    }
}
impl Null for u8 {
    #[inline(always)]
    fn null() -> Self {
        u8::MAX
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == u8::MAX
    }
}
impl Null for i8 {
    #[inline(always)]
    fn null() -> Self {
        i8::MIN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == i8::MIN
    }
}
impl Null for u16 {
    #[inline(always)]
    fn null() -> Self {
        u16::MAX
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == u16::MAX
    }
}
impl Null for i16 {
    #[inline(always)]
    fn null() -> Self {
        i16::MIN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == i16::MIN
    }
}
impl Null for u32 {
    #[inline(always)]
    fn null() -> Self {
        u32::MAX
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == u32::MAX
    }
}
impl Null for i32 {
    #[inline(always)]
    fn null() -> Self {
        i32::MIN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == i32::MIN
    }
}
impl Null for u64 {
    #[inline(always)]
    fn null() -> Self {
        u64::MAX
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == u64::MAX
    }
}
impl Null for i64 {
    #[inline(always)]
    fn null() -> Self {
        i64::MIN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == i64::MIN
    }
}
impl Null for u128 {
    #[inline(always)]
    fn null() -> Self {
        u128::MAX
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == u128::MAX
    }
}
impl Null for i128 {
    #[inline(always)]
    fn null() -> Self {
        i128::MIN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        *self == i128::MIN
    }
}
impl Null for f32 {
    #[inline(always)]
    fn null() -> Self {
        f32::NAN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        self.is_nan()
    }
}
impl Null for f64 {
    #[inline(always)]
    fn null() -> Self {
        f64::NAN
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        self.is_nan()
    }
}
impl Null for &str {
    #[inline(always)]
    fn null() -> Self {
        ""
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        self.is_empty()
    }
}
impl Null for String {
    #[inline(always)]
    fn null() -> Self {
        String::new()
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        self.is_empty()
    }
}
impl Null for TypeId {
    #[inline(always)]
    fn null() -> Self {
        unsafe {mem::transmute::<u64, TypeId>(u64::null()) }
    }
    #[inline(always)]
    fn is_null(&self) -> bool {
        unsafe {mem::transmute::<&TypeId, &u64>(self) }.is_null()
    }
}

#[test]
fn test() {
    let s = Some(1);
    assert_eq!(s.is_null(), false);
    assert_eq!(1.is_null(), false);
    assert_eq!(i8::MIN.is_null(), true);
    assert_eq!(2.0f32.is_null(), false);
    assert_eq!("".is_null(), true);
    assert_eq!("2".is_null(), false);
}
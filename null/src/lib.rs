//!
//! Null主要用在其他数据结构中，让值本身支持判断是否空。可以提升内存性能，减少使用Option。 
//!
pub trait Null {
    /// 判断当前值是否空
    fn null() -> Self;
    /// 判断当前值是否空
    fn is_null(&self) -> bool;
}

impl<T> Null for Option<T> {
    fn null() -> Self {
        None
    }
    fn is_null(&self) -> bool {
        match self {
            Some(_) => false,
            None => true,
        }
    }
}

impl Null for usize {
    fn null() -> Self {
        usize::max_value()
    }
    fn is_null(&self) -> bool {
        *self == usize::max_value()
    }
}
impl Null for isize {
    fn null() -> Self {
        isize::min_value()
    }
    fn is_null(&self) -> bool {
        *self == isize::min_value()
    }
}
impl Null for bool {
    fn null() -> Self {
        false
    }
    fn is_null(&self) -> bool {
        *self
    }
}
impl Null for u8 {
    fn null() -> Self {
        u8::MAX
    }
    fn is_null(&self) -> bool {
        *self == u8::MAX
    }
}
impl Null for i8 {
    fn null() -> Self {
        i8::MIN
    }
    fn is_null(&self) -> bool {
        *self == i8::MIN
    }
}
impl Null for u16 {
    fn null() -> Self {
        u16::MAX
    }
    fn is_null(&self) -> bool {
        *self == u16::MAX
    }
}
impl Null for i16 {
    fn null() -> Self {
        i16::MIN
    }
    fn is_null(&self) -> bool {
        *self == i16::MIN
    }
}
impl Null for u32 {
    fn null() -> Self {
        u32::MAX
    }
    fn is_null(&self) -> bool {
        *self == u32::MAX
    }
}
impl Null for i32 {
    fn null() -> Self {
        i32::MIN
    }
    fn is_null(&self) -> bool {
        *self == i32::MIN
    }
}
impl Null for u64 {
    fn null() -> Self {
        u64::MAX
    }
    fn is_null(&self) -> bool {
        *self == u64::MAX
    }
}
impl Null for i64 {
    fn null() -> Self {
        i64::MIN
    }
    fn is_null(&self) -> bool {
        *self == i64::MIN
    }
}
impl Null for u128 {
    fn null() -> Self {
        u128::MAX
    }
    fn is_null(&self) -> bool {
        *self == u128::MAX
    }
}
impl Null for i128 {
    fn null() -> Self {
        i128::MIN
    }
    fn is_null(&self) -> bool {
        *self == i128::MIN
    }
}
impl Null for f32 {
    fn null() -> Self {
        f32::NAN
    }
    fn is_null(&self) -> bool {
        self.is_nan()
    }
}
impl Null for f64 {
    fn null() -> Self {
        f64::NAN
    }
    fn is_null(&self) -> bool {
        self.is_nan()
    }
}
impl Null for &str {
    fn null() -> Self {
        ""
    }
    fn is_null(&self) -> bool {
        self.is_empty()
    }
}
impl Null for String {
    fn null() -> Self {
        String::new()
    }
    fn is_null(&self) -> bool {
        self.is_empty()
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
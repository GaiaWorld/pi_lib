//!
//! Invalid主要用在其他数据结构中，让值本身支持判断是否无效。可以提升内存性能，减少使用Option。 
//!
pub trait Invalid {
    /// 判断当前值是否无效
    fn is_invalid(&self) -> bool;
}

impl<T> Invalid for Option<T> {
    fn is_invalid(&self) -> bool {
        match self {
            Some(_) => false,
            None => true,
        }
    }
}

impl Invalid for usize {
    fn is_invalid(&self) -> bool {
        *self == usize::max_value()
    }
}
impl Invalid for isize {
    fn is_invalid(&self) -> bool {
        *self == isize::min_value()
    }
}
impl Invalid for bool {
    fn is_invalid(&self) -> bool {
        *self
    }
}
impl Invalid for u8 {
    fn is_invalid(&self) -> bool {
        *self == u8::MAX
    }
}
impl Invalid for i8 {
    fn is_invalid(&self) -> bool {
        *self == i8::MIN
    }
}
impl Invalid for u16 {
    fn is_invalid(&self) -> bool {
        *self == u16::MAX
    }
}
impl Invalid for i16 {
    fn is_invalid(&self) -> bool {
        *self == i16::MIN
    }
}
impl Invalid for u32 {
    fn is_invalid(&self) -> bool {
        *self == u32::MAX
    }
}
impl Invalid for i32 {
    fn is_invalid(&self) -> bool {
        *self == i32::MIN
    }
}
impl Invalid for u64 {
    fn is_invalid(&self) -> bool {
        *self == u64::MAX
    }
}
impl Invalid for i64 {
    fn is_invalid(&self) -> bool {
        *self == i64::MIN
    }
}
impl Invalid for u128 {
    fn is_invalid(&self) -> bool {
        *self == u128::MAX
    }
}
impl Invalid for i128 {
    fn is_invalid(&self) -> bool {
        *self == i128::MIN
    }
}
impl Invalid for f32 {
    fn is_invalid(&self) -> bool {
        self.is_nan()
    }
}
impl Invalid for f64 {
    fn is_invalid(&self) -> bool {
        self.is_nan()
    }
}
impl Invalid for str {
    fn is_invalid(&self) -> bool {
        self.is_empty()
    }
}
impl Invalid for String {
    fn is_invalid(&self) -> bool {
        self.is_empty()
    }
}
#[test]
fn test() {
    let s = Some(1);
    assert_eq!(s.is_invalid(), false);
    assert_eq!(1.is_invalid(), false);
    assert_eq!(i8::MIN.is_invalid(), true);
    assert_eq!(2.0f32.is_invalid(), false);
    assert_eq!("".is_invalid(), true);
    assert_eq!("2".is_invalid(), false);
}
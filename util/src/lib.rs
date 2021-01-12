/**
 * 通用函数库
 */

// 为Vec增加的新方法
pub trait VecIndex {
    type Item;
    fn index(&self, item: &Self::Item) -> Option<usize>;
}

impl<T: PartialEq> VecIndex for Vec<T> {
    type Item = T;
    #[inline]
    fn index(&self, item: &T) -> Option<usize> {
        for i in 0..self.len() {
            if unsafe { self.get_unchecked(i) } == item {
                return Some(i);
            }
        }
        None
    }
}

#[inline]
pub fn err_string<T, E: ToString>(err: Result<T, E>) -> Result<T, String> {
    match err {
        Ok(o) => Ok(o),
        Err(e) => Err(e.to_string()),
    }
}

// 为浮点数按精度取整， 0为取整，负数为保留多少位小数，正数为保留多少0
pub trait PrecisionRound {
    type Item;
    fn round(&self, precision: i32) -> Self::Item;
}

impl PrecisionRound for f32 {
    type Item = f32;
    #[inline]
    fn round(&self, precision: i32) -> f32 {
        let p = 10.0f32.powi(precision);
        (self * p + 0.5).floor() / p
    }
}
impl PrecisionRound for f64 {
    type Item = f64;
    #[inline]
    fn round(&self, precision: i32) -> f64 {
        let p = 10.0f64.powi(precision);
        (self * p + 0.5).floor() / p
    }
}

// 为Option增加的新方法
pub trait FetchDefault {
    type Item: Default;
    fn fetch_default(self) -> Self::Item;
}
impl<T: Default> FetchDefault for Option<T> {
    type Item = T;
    #[inline]
    fn fetch_default(self) -> Self::Item {
        match self {
            Some(t) => t,
            _ => Self::Item::default(),
        }
    }
}
// 为Option增加的新方法
pub trait FetchClone {
    type Item: Default + Clone;
    fn fetch_clone(self) -> Self::Item;
}
impl<T: Default + Clone> FetchClone for Option<T> {
    type Item = T;
    #[inline]
    fn fetch_clone(self) -> Self::Item {
        match self {
            Some(t) => t.clone(),
            _ => Self::Item::default(),
        }
    }
}

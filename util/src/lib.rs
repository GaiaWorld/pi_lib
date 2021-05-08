/**
 * 通用函数库
 */

/// 返回元素所在的偏移量 `Some(usize)` 如果这个slice 用给定的值找到了该元素.
///
/// # Examples
///
/// ```
/// let v = [10, 40, 30];
/// assert_eq!(v.index_of(&30), Some(2));
/// assert_eq!(!v.index_of(&50), None);
/// ```

pub trait IndexOf {
    type Item: PartialEq;
    fn index_of(&self, item: &Self::Item) -> Option<usize>;
}

impl<T: PartialEq> IndexOf for [T] {
    type Item = T;
    #[inline]
    fn index_of(&self, item: &T) -> Option<usize> {
        for i in 0..self.len() {
            if &self[i] == item {
                return Some(i);
            }
        }
        None
    }
}


/// 将Result<T, E: ToString>类型转成Result<T, String>类型
#[inline]
pub fn err_string<T, E: ToString>(err: Result<T, E>) -> Result<T, String> {
    match err {
        Ok(o) => Ok(o),
        Err(e) => Err(e.to_string()),
    }
}
/// 为浮点数(f32, f64)按精度取整， 0为取整，负数为保留多少位小数，正数为保留多少0
///
/// # Examples
///
/// ```
/// assert_eq!(2.51_f32.round(-1), 2.5_f32);
/// assert_eq!(278.2_f32.round(0), 278_f64);
/// assert_eq!(-278.2_f64.round(1), -270_f64);
/// ```
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
/// 为Option增加的取值方法，如果为None则返回缺省值
///
/// # Examples
///
/// ```
/// let mut x = 12;
/// let opt_x = Some(x);
/// assert_eq!(opt_x, Some(12));
/// let cloned = opt_x.fetch_default();
/// assert_eq!(cloned, 12);
/// assert_eq!(None.fetch_default(), 0);
/// ```
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
/// 为Option的引用增加的取值的方法，如果为None则返回缺省值
/// 相比先调用cloned(),然后调用fetch_default(), 少一次判断
///
/// # Examples
///
/// ```
/// let x = 12;
/// let opt_x = Some(x);
/// let cloned = opt_x.fetch_clone();
/// assert_eq!(cloned, 12);
/// assert_eq!(cloned, opt_x.cloned().fetch_default());
/// assert_eq!(None.fetch_clone(), 0);
/// ```
pub trait FetchCloneDefault {
    type Item: Default + Clone;
    fn fetch_clone_default(&self) -> Self::Item;
}
impl<T: Default + Clone> FetchCloneDefault for Option<T> {
    type Item = T;
    #[inline]
    fn fetch_clone_default(&self) -> Self::Item {
        match self {
            Some(t) => t.clone(),
            _ => Self::Item::default(),
        }
    }
}

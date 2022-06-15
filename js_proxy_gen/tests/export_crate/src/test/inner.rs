use std::fs::{self, File as SyncFile};
use std::sync::{Arc,
                atomic::{AtomicBool, Ordering}};
use std::collections::HashMap;

use parking_lot::{Mutex, RwLock};
use num_bigint::BigInt;

use js_proxy_gen_macro::pi_js_export;

///
/// 布尔值
///
#[pi_js_export]
pub const BOOL: bool = true;

///
/// 32位无符号整数
///
#[pi_js_export]
pub const UINT: u32 = 0xffffffff;

///
/// 32位有符号整数
///
#[pi_js_export]
pub const INT: i32 = -999_999_999;

///
/// 二进制
///
pub const BINARY: &'static [u8] = b"undefined";

///
/// 浮点数
///
#[pi_js_export]
pub const FLOAT: f64 = 1e-9;

///
/// 字符串
///
#[pi_js_export]
pub const STRING: &'static str = r#".\tests\test.rs"#;

///
/// 发送消息
///
#[pi_js_export]
pub const fn send(_: bool, _: usize, _: f64) -> Result<bool, f64> {
    Ok(true)
}

///
/// 解析大整数
///
#[pi_js_export]
pub fn parse_bigint(_: i8, _: u8, _: i16, _: u16, _: i32, _: u32, _: f32, _: f64, _: i64, _: u64, _: i128, _: u128, _: isize, _: usize, _: BigInt) -> usize {
    1
}

///
/// 解析大整数数组
///
#[pi_js_export]
pub fn parse_bigint_array(_: Vec<i8>, _: Vec<u8>, _: Vec<i16>, _: Vec<u16>, _: Vec<i32>, _: Vec<u32>, _: Vec<f32>, _: Vec<f64>, _: Vec<i64>, _: Vec<u64>, _: Vec<i128>, _: Vec<u128>, _: Vec<isize>, _: Vec<usize>, _: Vec<BigInt>) -> Vec<usize> {
    vec![1]
}

///
/// 获取数据
///
#[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String), Z = type(bool, usize, String))]
pub fn get_data<X, Y, Z>(x: X, y: Y, _z: Z) -> Result<Box<[u8]>, String> {
    Ok(Box::new(vec![]).into_boxed_slice())
}

///
/// 通知
///
#[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String), Z = type(bool, usize, String))]
pub async fn notify<'b, 'a: 'b, X, Y, Z>(x: &'a X, y: &'b Y, _z: &'a Z) -> Option<TestStruct<'b, 'a, X, Y>>
    where X: Default + Clone + Send + Sync + 'static,
          Y: Default + Clone + Send + Sync + 'static {
    Some(TestStruct::new(x, x, X::default(), Vec::new()))
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> Drop for TestStruct<'b, 'a, T, B> {
    ///
    /// 释放测试用结构体
    ///
    #[pi_js_export]
    fn drop(&mut self) {

    }
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> TestStruct<'b, 'a, T, B> {
    ///
    /// 构建测试用结构体
    ///
    #[pi_js_export(T = type(bool, usize, String), B = type(u8))]
    pub fn new<TT, BB>(x: &'a T, y: &'b T, z: T, vec: Vec<B>) -> Self {
        TestStruct {
            x,
            y,
            z,
            vec,
        }
    }

    ///
    /// 获取x的只读引用
    ///
    #[pi_js_export(T = type(bool, usize, String))]
    pub fn get_x(&self) -> &'a T {
        self.x
    }

    ///
    /// 设置x的只读引用
    ///
    #[pi_js_export(T = type(bool, usize, String))]
    pub fn set_x(&mut self, x: &'a T) {
        self.x = x;
    }

    ///
    /// 获取指定类型的值
    ///
    pub fn get<X: Clone>() -> Option<X> {
        None
    }

    ///
    /// 设置指定类型的值
    ///
    #[pi_js_export(T = type(bool, usize, String))]
    pub fn set<X: Clone>(x: X) -> Option<X> {
        Some(x)
    }

    ///
    /// 刷新指定类型的值
    ///
    #[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String))]
    pub fn flush<X, Y>(&'a mut self, x: X, y: &'a X, z: &'a mut Y, _c: &'a mut Vec<bool>, _: &HashMap<usize, String>) -> Result<&'a mut Y, String>
        where X: Clone, Y: Clone {
        Ok(z)
    }
}

///
/// 测试用结构体
///
#[pi_js_export(T = type(bool, usize, String), B = type(u8))]
pub struct TestStruct<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> {
    x:      &'a T,
    y:      &'b T,
    z:      T,
    vec:    Vec<B>,
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> Clone for TestStruct<'b, 'a, T, B> {
    ///
    /// 复制测试用结构体
    ///
    #[pi_js_export(T = type(bool, usize, String), B = type(u8))]
    fn clone(&self) -> Self {
        TestStruct {
            x: self.x,
            y: self.y,
            z: self.z.clone(),
            vec: self.vec.clone(),
        }
    }
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> TestStruct<'b, 'a, T, B> {
    ///
    /// 布尔值
    ///
    #[pi_js_export]
    pub const BOOL1: bool = true;

    ///
    /// 32位无符号整数
    ///
    #[pi_js_export]
    pub const UINT1: u32 = 0xffffffff;

    ///
    /// 32位有符号整数
    ///
    #[pi_js_export]
    pub const INT1: i32 = -999_999_999;

    ///
    /// 二进制
    ///
    pub const BINARY1: &'static [u8] = b"undefined";

    ///
    /// 浮点数
    ///
    #[pi_js_export]
    pub const FLOAT1: f64 = 1.1231658798;

    ///
    /// 字符串
    ///
    #[pi_js_export]
    pub const STRING1: &'static str = r#".\tests\test.rs"#;

    ///
    /// 同步指定类型的值
    ///
    #[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String))]
    pub async fn sync<X, Y>(&'a mut self, x: X, y: &'a X, z: &'a mut Y, _c: &'a mut Vec<Vec<bool>>, _r: &'a mut HashMap<Vec<usize>, Vec<String>>) -> Result<&'a mut HashMap<Vec<usize>, Vec<String>>, String>
        where X: Clone, Y: Clone {
        Ok(_r)
    }
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> Drop for TestEnum<'b, 'a, T, B> {
    ///
    /// 释放测试用枚举
    ///
    #[pi_js_export]
    fn drop(&mut self) {

    }
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> Clone for TestEnum<'b, 'a, T, B> {
    ///
    /// 复制测试用枚举
    ///
    #[pi_js_export]
    fn clone(&self) -> Self {
        match self {
            TestEnum::None => TestEnum::None,
            TestEnum::X(x) => TestEnum::X(x.clone()),
            TestEnum::Y(y) => TestEnum::Y(y.clone()),
            TestEnum::Z(z) => TestEnum::Z(z.clone()),
            TestEnum::Vec(vec) => TestEnum::Vec(vec.clone()),
        }
    }
}

///
/// 测试用枚举
///
#[pi_js_export(T = type(bool, usize, String), B = type(u8))]
pub enum TestEnum<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> {
    None,
    X(&'a T),
    Y(&'b T),
    Z(T),
    Vec(Vec<B>),
}

impl<'b, 'a: 'b, T: Clone + Send + Sync + 'static, B: Clone + Send + Sync + 'static> TestEnum<'b, 'a, T, B> {
    ///
    /// 同步指定类型的值
    ///
    #[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String))]
    pub async fn sync<X, Y>(&'a mut self, x: X, y: &'a X, z: &'a mut Y, _c: &'a Vec<Vec<bool>>, _r: &'a mut HashMap<Vec<usize>, Vec<String>>) -> Result<&'a mut HashMap<Vec<usize>, Vec<String>>, String>
        where X: Clone, Y: Clone {
        Ok(_r)
    }
}

///
/// 测试
///
pub fn test<'b, 'a: 'b, X, Y, Z>(x: &'a X, y: &'b Y, _z: &'a Z) -> Option<TestStruct<'b, 'a, X, Y>>
    where X: Default + Clone + Send + Sync + 'static,
          Y: Default + Clone + Send + Sync + 'static {
    Some(TestStruct::new(x, x, X::default(), Vec::new()))
}

///
/// 测试用简单结构体
///
#[pi_js_export(T = type(HashMap<usize, Arc<[u8]>>))]
pub struct TestSimpleStruct<T: Clone> {
    inner: T,
}

impl<T: Clone + Send + Sync + 'static> TestSimpleStruct<T> {
    ///
    /// 构造测试用简单结构体
    ///
    #[pi_js_export(X = type(HashMap<bool, Vec<u8>>))]
    pub fn new<X>(inner: &T, x: &X) -> Self {
        TestSimpleStruct {
            inner: inner.clone(),
        }
    }

    ///
    /// 异步构造测试用简单结构体
    ///
    #[pi_js_export(X = type(HashMap<String, Box<[u8]>>))]
    pub async fn async_new<X>(inner: &T, x: &mut X) -> Self {
        TestSimpleStruct {
            inner: inner.clone(),
        }
    }
}
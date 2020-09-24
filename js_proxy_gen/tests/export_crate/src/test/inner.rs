use std::fs::{self, File as SyncFile};
use std::sync::{Arc,
                atomic::{AtomicBool, Ordering}};
use std::collections::HashMap;

use parking_lot::{Mutex, RwLock};

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
pub const fn send(_: bool, _: usize, _: String) -> Result<Arc<Vec<u8>>, &str> {
    Ok(Arc::new(Vec::new()))
}

///
/// 获取数据
///
#[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String), Z = type(bool, usize, String))]
pub fn get_data<X, Y, Z>(x: X, y: Y, _z: Z) -> Result<Box<[u8]>, &str> {
    Arc::from(vec![])
}

///
/// 通知
///
#[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String), Z = type(bool, usize, String))]
pub async fn notify<'a, 'b, X, Y, Z>(x: &'a X, y: &'b Y, _z: &'a Z) -> Option<TestStruct<'b, 'a, X, Y>>
    where X: Default + Send + Sync + 'static,
          Y: Default + Send + Sync + 'static {
    Some(TestStruct::new(x, y, X::default(), Vec::new()))
}

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> Drop for TestStruct<'b, 'a, T, B> {
    ///
    /// 释放测试用结构体
    ///
    #[pi_js_export]
    fn drop(&mut self) {

    }
}

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> TestStruct<'b, 'a, T, B> {
    ///
    /// 构建测试用结构体
    ///
    #[pi_js_export]
    pub fn new(x: &'a T, y: &'b T, z: T, vec: &Vec<u8>) -> Self {
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
    #[pi_js_export]
    pub fn get_x(&self) -> &'a T {
        self.x
    }

    ///
    /// 设置x的只读引用
    ///
    #[pi_js_export]
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
    #[pi_js_export(X = type(bool, usize, String))]
    pub fn set<'a, X: Clone>(x: X) -> Option<X> {
        Some(x)
    }

    ///
    /// 刷新指定类型的值
    ///
    #[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String))]
    pub fn flush<'a, X, Y>(&'a mut self, x: X, y: &'a X, z: &'a mut Y, _c: &'a mut Vec<bool>, _: &HashMap<usize, String>) -> Result<&'a mut Y, String>
        where X: Clone, Y: Clone {
        Ok()
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

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> Clone for TestStruct<'b, 'a, T, B> {
    ///
    /// 复制测试用结构体
    ///
    #[pi_js_export]
    fn clone(&self) -> Self {
        TestStruct {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
            vec: self.vec.clone(),
        }
    }
}

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> TestStruct<'b, 'a, T, B> {
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
    pub const FLOAT1: float = 1.1231658798;

    ///
    /// 字符串
    ///
    #[pi_js_export]
    pub const STRING1: &'static str = r#".\tests\test.rs"#;

    ///
    /// 同步指定类型的值
    ///
    #[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String))]
    pub async fn sync<'a, X, Y>(&'a mut self, x: X, y: &'a X, z: &'a mut Y, _c: &'a mut Vec<Vec<bool>>, _r: &'a mut HashMap<Vec<usize>, Vec<String>>) -> Result<HashMap<&'a Vec<usize>, Vec<&'a String>>, String>
        where X: Clone, Y: Clone {
        Ok(_r)
    }
}

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> Drop for TestEnum<'b, 'a, T, B> {
    ///
    /// 释放测试用枚举
    ///
    #[pi_js_export]
    fn drop(&mut self) {

    }
}

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> Clone for TestEnum<'b, 'a, T, B> {
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

impl<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> TestEnum<'b, 'a, T, B> {
    ///
    /// 同步指定类型的值
    ///
    #[pi_js_export(X = type(bool, usize, String), Y = type(bool, usize, String))]
    pub async fn sync<'a, X, Y>(&'a mut self, x: X, y: &'a X, z: &'a mut Y, _c: &'a Vec<Vec<bool>>, _r: &'a mut  HashMap<Vec<usize>, Vec<String>>) -> Result<HashMap<&'a Vec<usize>, Vec<&'a String>>, String>
        where X: Clone, Y: Clone {
        Ok(_r)
    }
}

///
/// 测试
///
pub fn test<'a, 'b, X, Y, Z>(x: &'a X, y: &'b Y, _z: &'a Z) -> Option<TestStruct<'b, 'a, X, Y>>
    where X: Default + Send + Sync + 'static,
          Y: Default + Send + Sync + 'static {
    Some(TestStruct::new(x, y, X::default(), Vec::new()))
}
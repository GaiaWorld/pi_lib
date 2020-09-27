use std::sync::{Arc, Mutex};

///
/// 测试用结构体A
///
#[pi_js_export(T = type(u8))]
pub struct A<'a, T>(bool, usize, String, Arc<Mutex<usize>>, Vec<T>);
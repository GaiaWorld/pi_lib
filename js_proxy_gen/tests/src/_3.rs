use std::sync::{Arc, Mutex};

///
/// 测试用结构体A
///
#[pi_js_export]
pub struct A(Arc<Mutex<usize>>);
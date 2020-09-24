use std::time::Duration;

///
/// 测试用结构体A
///
#[pi_js_export]
pub struct A {
    x:  bool,
    y:  String,
    z:  Duration,
}
///
/// 测试用结构体A
///
#[pi_js_export]
pub struct A<'a> {
    x: &'a usize,
    y: &'a str,
    z: &'a String,
}
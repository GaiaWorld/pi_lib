///
/// 测试用结构体A
///
#[pi_js_export(T = type(bool, usize, String), B = type(u8))]
pub struct A<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> {
    x:      &'a T,
    y:      &'b T,
    z:      T,
    vec:    Vec<B>,
}
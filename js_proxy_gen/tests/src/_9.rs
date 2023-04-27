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

mod test {
    ///
    /// 测试用结构体B
    ///
    #[cfg(target_os = "windows")]
    #[cfg(feature = "pi_js_export")]
    pub struct B<'b, 'a: 'b, T: Send + Sync + 'static, B: Send + Sync + 'static> {
        x:      &'a T,
        y:      &'b T,
        z:      T,
        vec:    Vec<B>,
    }
}

#[pi_js_export]
pub fn test_callback(x: bool,
                     y: usize,
                     z: String,
                     func: Arc<dyn Fn(bool, u32, f64, String, &[u8], Option<Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>>) + Send + 'static>) {

}
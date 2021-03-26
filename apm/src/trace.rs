//! # 跟踪线程堆栈
//!

use backtrace::Backtrace;

///
/// 堆栈跟踪器
///
pub struct StackTracer {
    inner: Backtrace,
}

impl StackTracer {
    /// 构建一个堆栈跟踪器
    pub fn new() -> Self {
        StackTracer {
            inner: Backtrace::new_unresolved(),
        }
    }

    /// 打印当前线程堆栈
    pub fn print_stack(&mut self) {
        self.inner.resolve();
        println!("{:?}", self.inner);
    }
}
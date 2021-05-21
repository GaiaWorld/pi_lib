//! 本库最开始是从[adler32](https://github.com/remram44/adler32-rs)拷贝过来， 拷贝原因不记得了
//! 但此时再对比[adler32](https://github.com/remram44/adler32-rs),发现与它的代码并没有什么出入，如果你想使用本库的功能，请直接引用(https://github.com/remram44/adler32-rs)
//! 本库的代码也不再维护，目前是直接引用(https://github.com/remram44/adler32-rs)，以防止其它引用本库的地方出错。

extern crate adler32;
pub use adler32::*;

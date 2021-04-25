//! 提供两个宏，用于在输出内容到控制台上
//! `debug_println!`，与标准库中的`println!`具有相同的效果，但其受`feature`的控制，只有存在`print`这个`feature`时，才会输出
//! `debug_print!`，与标准库中的`print!`具有相同的效果，但其受`feature`的控制，只有存在`print`这个`feature`时，才会输出
//!
//! ##使用
//!
//! ### Cargo.toml
//! 
//!     ...
//!     [features]
//!     default = ["print"] // 默认features中添加print时，源码中调用debug_println!或debug_print!，才能在控制台输出，否则不输出
//!     print=[]
//!     ...
//!### example
//!
//! debug_print!("打印一个数字:{}", 5);

#[ macro_export ] 
macro_rules! debug_println {
    ($($ arg: tt)*)=>(if cfg!(feature = "print"){ println!($($ arg)*);})
}

#[ macro_export ] 
macro_rules! debug_print {
    ($($ arg: tt)*)=>(if cfg!(feature = "print"){ print!($($ arg)*);})
}
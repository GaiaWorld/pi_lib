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
// #![feature(repr128)]
// #![feature(format_args_nl)]


#[cfg(feature = "wasm-bindgen")]
pub extern crate web_sys;


#[cfg(all(feature = "print", not(feature = "wasm-bindgen")))]
#[ macro_export ] 
macro_rules! debug_println {
    ($($ arg: tt)*)=>{
        println!($($ arg)*);
    }
}

#[cfg(all(feature = "print", feature = "wasm-bindgen"))]
#[ macro_export ] 
macro_rules! debug_println {
    ($($ arg: tt)*) => {{
        let s = format!($($ arg)*);
        unsafe { $crate::web_sys::console::log_1( &s.into())};
    }}
}

#[cfg(not(feature = "print"))]
#[ macro_export ] 
macro_rules! debug_println {
    ($($ arg: tt)*)=>{{

    }}
}


#[cfg(all(feature = "print", not(feature = "wasm-bindgen")))]
#[ macro_export ] 
macro_rules! debug_print {
    ($($ arg: tt)*)=>{
        print!($($ arg)*);
    }
}

#[cfg(all(feature = "print", feature = "wasm-bindgen"))]
#[ macro_export ] 
macro_rules! debug_print {
    ($($ arg: tt)*)=>{{
        let s = format!($($ arg)*);
        unsafe { $crate::web_sys::console::log_1( &s.into())};
    }}
}


#[cfg(not(feature = "print"))]
#[ macro_export ] 
macro_rules! debug_print {
    ($($ arg: tt)*)=>{{

    }}
}
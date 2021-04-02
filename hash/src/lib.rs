//! 本模块提供`XHashMap`、`XHashSet`两种容器，来替代标准库的HashMap和HashSet。
//! 意在鼓励外部库大多数时候都使用本模块提供的XHashMap、XHashSet来替代标准库或其它库的HashMap和HashSet。
//! 使用本模块的优势在于，库的编写者，在使用这两种容器时，不用关心使用哪种hash算法
//! 而由具体的应用程序关心，应用程序可以设置不同的feature，来确定自己需要哪种hash算法。
//! 
//! 例如：
//! 一个名为`gui`的库，使用了本模块的XHashMap；
//!
//! 另一个库`gui_web`,是对`gui`的再次封装，意在编译为asm供web平台使用，考虑到asm中，64位整数的计算速度明显低于32位，因此
//! 希望使用一个32位的hash算法，另外，gui的hashMap，大部分的key的长度，
//! 使用xxhash对比其它hash算法会更快（不同的hash算法，在不同的场景中有着自身的优势和劣势）,因此，`gui_web`决定使用32位的xxhash
//! `gui_web`仅需在在Cargo.toml中添加`feature`位`xxhash`，就能控制`gui`库使用的HashMap的Hash算法位`xxhash`
//! 至于要使用**32**位的xxhash，本模块可以根据编译目标，自动选择字长。如要编译为wasm、asm，会自动选择32位的算法
//! 当然，也有可能，在应用的过程中，发现xxhash不时一个好的选择，你可以低成本的更换算法（只需要修改`feature`）
//! 
//! 目前，本库支持的hash算法有限，仅支持了xxhash、fxhash。



extern crate fxhash;
extern crate twox_hash;

use std::hash::{BuildHasherDefault};
use std::collections::{HashMap, HashSet};

// 32位平台下， not(feature = "xxhash")时，默认使用FxHasher32
#[cfg(all( not(feature = "xxhash") , target_pointer_width = "32"))]
pub type DefaultHasher = fxhash::FxHasher32;
// 64位平台下，not(feature = "xxhash")时，默认使用FxHasher64
#[cfg(all( not(feature = "xxhash") , target_pointer_width = "64"))]
pub type DefaultHasher = fxhash::FxHasher64;
// 32位平台下，feature = "xxhash"时，默认使用XxHash32
#[cfg(all(feature = "xxhash", target_pointer_width = "32"))]
pub type DefaultHasher = twox_hash::XxHash32;
// 64位平台下，feature = "xxhash"时，默认使用XxHash64
#[cfg(all(feature = "xxhash", target_pointer_width = "64"))]
pub type DefaultHasher = twox_hash::XxHash64;

// 当前默认的HashMap和HashSet（使用根据平台字长、和feature来决定的DefaultHasher）
pub type XHashMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;
pub type XHashSet<K> = HashSet<K, BuildHasherDefault<DefaultHasher>>;
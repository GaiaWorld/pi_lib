extern crate fxhash;
extern crate twox_hash;

use std::hash::{BuildHasherDefault};
use std::collections::{HashMap, HashSet};

#[cfg(not(feature = "xxhash"))]
use fxhash::FxHasher32;
#[cfg(not(feature = "xxhash"))]
pub type DefaultHasher32 = FxHasher32;
#[cfg(all(not(feature = "xxhash"), target_pointer_width = "32"))]
pub type DefaultHasher = FxHasher32;
#[cfg(all(not(feature = "xxhash"), target_pointer_width = "32"))]
pub type XHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;
#[cfg(all(not(feature = "xxhash"), target_pointer_width = "32"))]
pub type XHashSet<K> = HashSet<K, BuildHasherDefault<FxHasher32>>;

#[cfg(not(feature = "xxhash"))]
use fxhash::FxHasher64;
#[cfg(not(feature = "xxhash"))]
pub type DefaultHasher64 = FxHasher64;
#[cfg(all(not(feature = "xxhash"), target_pointer_width = "64"))]
pub type DefaultHasher = FxHasher64;
#[cfg(all(not(feature = "xxhash"), target_pointer_width = "64"))]
pub type XHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher64>>;
#[cfg(all(not(feature = "xxhash"), target_pointer_width = "64"))]
pub type XHashSet<K> = HashSet<K, BuildHasherDefault<FxHasher64>>;




#[cfg(feature = "xxhash")]
use twox_hash::XxHash32;
#[cfg(feature = "xxhash")]
pub type DefaultHasher32 = XxHash32;
#[cfg(all(feature = "xxhash", target_pointer_width = "32"))]
pub type DefaultHasher = XxHash32;
#[cfg(all(feature = "xxhash", target_pointer_width = "32"))]
pub type XHashMap<K, V> = HashMap<K, V, BuildHasherDefault<XxHash32>>;
#[cfg(all(feature = "xxhash", target_pointer_width = "32"))]
pub type XHashSet<K> = HashSet<K, BuildHasherDefault<XxHash32>>;

#[cfg(feature = "xxhash")]
use twox_hash::XxHash64;
#[cfg(feature = "xxhash")]
pub type DefaultHasher64 = XxHash64;
#[cfg(all(feature = "xxhash", target_pointer_width = "64"))]
pub type DefaultHasher = XxHash64;
#[cfg(all(feature = "xxhash", target_pointer_width = "64"))]
pub type XHashMap<K, V> = HashMap<K, V, BuildHasherDefault<XxHash64>>;
#[cfg(all(feature = "xxhash", target_pointer_width = "64"))]
pub type XHashSet<K> = HashSet<K, BuildHasherDefault<XxHash64>>;


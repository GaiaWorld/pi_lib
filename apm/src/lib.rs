//! # 应用性能监控器
//!

#![allow(warnings)]

#![feature(integer_atomics)]
#![feature(duration_as_u128)]

extern crate fnv;
extern crate sysinfo;
extern crate netstat2;
extern crate backtrace;
extern crate parking_lot;
extern crate crossbeam_queue;

#[macro_use]
extern crate lazy_static;

// TODO: 为安卓生成so时,不能编译部分代码,需要全局搜索"#[cfg(any(unix))]"替换成unix但不包括Android(类似但不是:#[cfg(all(unix, not(android)))]),以排除这些代码.
#[cfg(all(unix, not(target_os="android")))]
extern crate libc;
#[cfg(all(unix, not(target_os="android")))]
extern crate psutil;
#[cfg(all(unix, not(target_os="android")))]
extern crate walkdir;

extern crate pi_atom;

use std::path::PathBuf;
use std::collections::HashMap;

///
/// 系统特定平台状态
///
pub trait SysSpecialStat {
    /// 获取系统cpu占用率
    fn sys_cpu_usage(&self) -> Option<f64>;

    /// 获取系统cpu所有逻辑核心的占用率
    fn sys_processores_usage(&self) -> Option<Vec<f64>>;

    /// 获取系统cpu详细使用信息
    fn sys_cpu_detal(&self) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)>;

    /// 获取系统cpu所有逻辑核心的占用率
    fn sys_processores_detal(&self) -> Option<Vec<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)>>;

    /// 获取系统负载系数
    fn sys_loadavg(&self) -> Option<(f32, f32, f32)>;

    /// 获取系统虚拟内存详细信息
    fn sys_virtual_memory_detal(&self) -> Option<(u64, u64, u64, u64, u64, u64, u64, u64, u64, f32)>;

    /// 获取系统交换区详细信息
    fn sys_swap_detal(&self) -> Option<(u64, u64, u64, u64, u64, f32)>;

    /// 获取系统正常运行时长
    fn sys_uptime(&self) -> isize;

    /// 获取当前进程号
    fn process_current_pid(&self) -> i32;

    /// 获取指定进程详细信息
    fn process_detal(&self, i32) -> Option<(u32, u32, i64, i64, u32, f64, f64, u64, i64, u64, u64, u64, u64, u64, i32, i64, f64, String, String, String, PathBuf)>;

    /// 获取指定进程环境
    fn process_env(&self, i32) -> Option<HashMap<String, String>>;

    /// 获取指定进程内存信息
    fn process_memory(&self, i32) -> Option<(u64, u64, u64, u64, u64, u64)>;

    /// 获取指定进程文件句柄数量
    fn process_fd_size(&self, i32) -> Option<usize>;

    /// 获取指定进程文件句柄信息
    fn process_fd(&self, i32) -> Option<Vec<(i32, PathBuf)>>;

    /// 获取指定进程的线程id列表
    fn process_threads(&self, i32) -> Option<Vec<i32>>;

    /// 获取硬盘分区信息
    fn disk_part(&self, bool) -> Option<Vec<(String, String, String, String)>>;

    /// 获取硬盘占用信息
    fn disk_usage(&self, path: &str) -> Option<(u64, u64, u64, u64, u64, u64, f64)>;

    /// 获取硬盘IO详细信息
    fn disk_io_detal(&self) -> Option<Vec<(String, u64, u64, u64, u64, u64, u64, u64, u64, u64)>>;

    /// 获取网络IO详细信息
    fn network_io_detal(&self) -> Option<Vec<(String, u64, u64, u64, u64, u64, u64, u64, u64)>>;
}

pub mod common;

#[cfg(all(unix, not(target_os="android")))]
pub mod linux;

pub mod allocator;
pub mod trace;
pub mod counter;
extern crate sysinfo;
extern crate netstat;

#[cfg(any(unix))]
extern crate psutil;

use std::path::PathBuf

/*
* 系统特定平台状态
*/
pub trait SysSpecialStat {
    //获取系统cpu占用率
    fn sys_cpu_usage(&self) -> Option<f64>;

    //获取系统cpu所有逻辑核心的占用率
    fn sys_processores_usage(&self) -> Option<Vec<f64>>;

    //获取系统cpu详细使用信息
    fn sys_cpu_detal(&self) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)>;

    //获取系统cpu所有逻辑核心的占用率
    fn sys_processores_detal(&self) -> Option<Vec<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)>>;

    //获取系统负载系数
    fn sys_loadavg(&self) -> Option<(f32, f32, f32)>;

    //获取系统虚拟内存详细信息
    fn sys_virtual_memory_detal(&self) -> Option<(u64, u64, u64, u64, u64, u64, u64, u64, u64, f32)>;

    //获取系统交换区详细信息
    fn sys_swap_detal(&self) -> Option<(u64, u64, u64, u64, u64, f32)>;

    //获取系统正常运行时长
    fn sys_uptime(&self) -> isize;

    //获取当前进程号
    fn process_current_pid(&self) -> i32;

    //获取当前进程详细信息
    fn process_current_detal(&self) -> Option<(u32, u32, i64, i64, f64, f64, u64, i64, u64, u64, u64, u64, u64, i32, i64, f64, String, String, String, PathBuf)>;

    //获取当前进程内存信息
    fn process_current_memory(&self) -> Option<(u64, u64, u64, u64, u64, u64)>;
}

pub mod common;

#[cfg(any(unix))]
pub mod linux;
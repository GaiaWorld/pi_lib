//! # 获取Linux的系统信息
//!
//!
extern crate psutil;

use std::ptr;
use std::cmp;
use std::thread;
use std::sync::Arc;
use std::ffi::OsString;
use std::cell::RefCell;
use std::time::Duration;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use libc;
use psutil::{cpu, host, memory, process, disk, network};
use walkdir::{DirEntry, WalkDir};

use ::SysSpecialStat;

/*
* 默认间隔时长
*/
const DEFAULT_INTERVAL: f64 = 1.0;

/*
* 系统进程根路径
*/
const PROCESS_ROOT_PATH: &'static str = "/proc/";

/*
* 系统线程目录
*/
const THREADS_DIR: &'static str = "/task";

///
/// 获取当前线程的本地线程id
///
pub fn current_tid() -> i32 {
    unsafe { libc::syscall(libc::SYS_gettid) as i32 }
}

///
/// linux系统状态
///
pub struct LinuxSysStat {
    interval: f64,                                              //采集间隔时长，单位秒
    disk_counter: Arc<RefCell<disk::DiskIOCountersCollector>>,  //硬盘io计数器
    net_counter: Arc<RefCell<network::NetIOCountersCollector>>, //网络io计数器
}

impl SysSpecialStat for LinuxSysStat {
    fn sys_cpu_usage(&self) -> Option<f64> {
        if let Ok(usage) = cpu::cpu_percent(self.interval) {
            return Some(usage);
        }

        None
    }

    fn sys_processores_usage(&self) -> Option<Vec<f64>> {
        if let Ok(usages) = cpu::cpu_percent_percpu(self.interval) {
            return Some(usages);
        }

        None
    }

    fn sys_cpu_detal(&self) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)> {
        if let Ok(info) = cpu::cpu_times_percent(self.interval) {
            return Some((info.user,         //用户占用率，包括虚拟环境中虚拟cpu占用率
                         info.nice,         //低优先级进程占用率，包括虚拟环境中低优先级的虚拟cpu占用率
                         info.system,       //内核占用率
                         info.idle,         //空闲率
                         info.iowait,       //io阻塞占用率
                         info.irq,          //硬中断占用率
                         info.softirq,      //软中断占用率
                         info.steal,        //虚拟环境中其它系统占用率
                         info.guest,        //虚拟环境中虚拟cpu占用率
                         info.guest_nice)); //虚拟环境中低优先级的虚拟cpu占用率
        }

        None
    }

    fn sys_processores_detal(&self) -> Option<Vec<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)>> {
        let mut vec = Vec::new();

        if let Ok(infos) = cpu::cpu_times_percent_percpu(self.interval) {
            for info in infos {
                vec.push((info.user,
                          info.nice,
                          info.system,
                          info.idle,
                          info.iowait,
                          info.irq,
                          info.softirq,
                          info.steal,
                          info.guest,
                          info.guest_nice));
            }
        } else {
            return None;
        }

        Some(vec)
    }

    fn sys_loadavg(&self) -> Option<(f32, f32, f32)> {
        if let Ok(info) = host::loadavg() {
            return Some((info.one,          //1分钟前的负载系数
                         info.five,         //5分钟前的负载系数
                         info.fifteen));    //15分钟前的负载系数
        }

        None
    }

    fn sys_virtual_memory_detal(&self) -> Option<(u64, u64, u64, u64, u64, u64, u64, u64, u64, f32)> {
        if let Ok(info) = memory::virtual_memory() {
            return Some((info.total,        //虚拟内存总大小，单位KB
                         info.free,         //虚拟内存空闲大小，单位KB
                         info.used,         //虚拟内存已使用大小，单位KB
                         info.available,    //可分配到新进程的虚拟内存大小，单位KB
                         info.active,       //正在使用的文件缓冲区和高速缓存大小，单位KB
                         info.inactive,     //空闲的文件缓冲区和高速缓存大小，单位KB
                         info.buffers,      //文件缓冲区的大小，单位KB
                         info.cached,       //高速缓存的大小，单位KB
                         info.shared,       //tmpfs使用的大小，单位KB
                         info.percent));    //虚拟内存占用率
        }

        None
    }

    fn sys_swap_detal(&self) -> Option<(u64, u64, u64, u64, u64, f32)> {
        if let Ok(info) = memory::swap_memory() {
            return Some((info.total,        //交换区总大小，单位KB
                         info.free,         //交换区空闲大小，单位KB
                         info.used,         //交换区已使用大小，单位KB
                         info.sin,          //从硬盘交换到内存的换入大小，单位KB
                         info.sout,         //从内存交换到硬盘的换出大小，单位KB
                         info.percent));    //交换区占用率
        }

        None
    }

    fn sys_uptime(&self) -> isize {
        host::uptime()
    }

    fn process_current_pid(&self) -> i32 {
        unsafe { libc::getpid() }
    }

    fn process_detal(&self, pid: i32) -> Option<(u32, u32, i64, i64, u32, f64, f64, u64, i64, u64, u64, u64, u64, u64, i32, i64, f64, String, String, String, PathBuf)> {
        if let (sys_usage, user_usage, Some(info)) = get_cpu_usage_by_process(self, pid) {
            let mut cmd = "".to_string();
            let mut cwd = PathBuf::new();
            if let Ok(Some(r)) = info.cmdline() {
                cmd = r;
            }
            if let Ok(r) = info.cwd() {
                cwd = r;
            }

            return Some((info.uid,                  //进程所属用户id
                         info.gid,                  //进程所属组id
                         info.nice,                 //进程静态优先级，数字越小，优先级越高
                         info.priority,             //进程动态优先级，数字越小，优先级越高
                         info.rt_priority,          //进程实时优先级
                         sys_usage,                 //进程内核态cpu占用率
                         user_usage,                //进程用户态cpu占用率
                         info.vsize,                //进程虚拟内存大小，单位B
                         info.rss,                  //进程占用内存大小，单位B
                         info.rsslim,               //进程占用内存大小限制，单位B
                         info.minflt,               //进程次缺页数量
                         info.cminflt,              //子进程次缺页数量
                         info.majflt,               //进程主缺页数量
                         info.cmajflt,              //子进程主缺页数量
                         info.processor,            //进程最近在哪个逻辑核心上运行
                         info.num_threads,          //进程的当前线程数
                         info.starttime,            //进程启动时间，单位秒
                         info.comm,                 //进程启动指令
                         info.state.to_string(),    //进程当前状态
                         cmd,                       //进程启动指令行
                         cwd));                     //进程当前工作目录
        }

        None
    }

    fn process_env(&self, pid: i32) -> Option<HashMap<String, String>> {
        if let Ok(process) = process::Process::new(pid) {
            if let Ok(env) = process.environ() {
                return Some(env);
            }
        }

        None
    }

    fn process_memory(&self, pid: i32) -> Option<(u64, u64, u64, u64, u64, u64)> {
        if let Ok(process) = process::Process::new(pid) {
            if let Ok(memory) = process.memory() {
                return Some((process.vsize,     //进程虚拟内存大小，单位B
                             memory.size,       //进程总内存大小，单位B
                             memory.resident,   //进程占用内存大小，单位B
                             memory.share,      //进程共享页内存大小，单位B
                             memory.text,       //进程代码段内存大小，单位B
                             memory.data));     //进程数据段内存大小，单位B
            }
        }

        None
    }

    fn process_fd_size(&self, pid: i32) -> Option<usize> {
        if let Ok(process) = process::Process::new(pid) {
            if let Ok(fds) = process.open_fds() {
                return Some(fds.len());
            }
        }

        None
    }

    fn process_fd(&self, pid: i32) -> Option<Vec<(i32, PathBuf)>> {
        if let Ok(process) = process::Process::new(pid) {
            if let Ok(fds) = process.open_fds() {
                let mut vec = Vec::with_capacity(fds.len());
                for fd in fds {
                    vec.push((fd.number,    //文件句柄
                              fd.path));          //文件路径
                }
            }
        }

        None
    }

    fn process_threads(&self, pid: i32) -> Option<Vec<i32>> {
        threads(pid)
    }

    fn disk_part(&self, all: bool) -> Option<Vec<(String, String, String, String)>> {
        if let Ok(parts) = disk::disk_partitions(all) {
            let mut vec = Vec::with_capacity(parts.len());

            for part in parts {
                vec.push((part.device,  //硬盘驱动器
                          part.mountpoint,  //挂载点
                          part.fstype,      //文件系统类型
                          part.opts));      //选项
            }

            return Some(vec);
        }

        None
    }

    fn disk_usage(&self, path: &str) -> Option<(u64, u64, u64, u64, u64, u64, f64)> {
        if let Ok(usage) = disk::disk_usage(path) {
            return Some((usage.total,               //硬盘总大小，单位B
                         usage.free,                //硬盘空闲大小，单位B
                         usage.used,                //硬盘占用大小，单位B
                         usage.disk_inodes_total,   //硬盘文件节点总数
                         usage.disk_inodes_free,    //硬盘文件节点空闲数
                         usage.disk_inodes_used,    //硬盘文件节点占用数
                         usage.percent));           //硬盘占用率
        }

        None
    }

    fn disk_io_detal(&self) -> Option<Vec<(String, u64, u64, u64, u64, u64, u64, u64, u64, u64)>> {
        if let Ok(map) = self.disk_counter.borrow_mut().disk_io_counters_perdisk(true) {
            let mut vec = Vec::with_capacity(map.len());

            for (key, value) in map {
                vec.push((key,                  //硬盘名
                          value.read_count,         //硬盘累计读取次数
                          value.write_count,        //硬盘累计写入次数
                          value.read_bytes,         //硬盘累计读取字节数，单位B
                          value.write_bytes,        //硬盘累计写入字节数，单位B
                          value.read_time,          //硬盘累计读取时间，单位毫秒
                          value.write_time,         //硬盘累计写入时间，单位毫秒
                          value.read_merged_count,  //硬盘累计读取合并次数
                          value.write_merged_count, //硬盘累计写入合并次数
                          value.busy_time));        //硬盘繁忙时间，单位毫秒
            }

            return Some(vec);
        }

        None
    }

    fn network_io_detal(&self) -> Option<Vec<(String, u64, u64, u64, u64, u64, u64, u64, u64)>> {
        if let Ok(map) = self.net_counter.borrow_mut().net_io_counters_pernic(true) {
            let mut vec = Vec::with_capacity(map.len());

            for (key, value) in map {
                vec.push((key,              //网络接口名
                          value.bytes_send,     //网络接口累计发送字节数
                          value.bytes_recv,     //网络接口累计接收字节数
                          value.packets_send,   //网络接口累计发送数据包数
                          value.packets_recv,   //网络接口累计接收数据包数
                          value.errin,          //网络接口累计接收错误次数
                          value.errout,         //网络接口累计发送错误次数
                          value.dropin,         //网络接口累计丢弃的接收数据包数
                          value.dropout));      //网络接口累计丢弃的发送数据包数
            }

            return Some(vec);
        }

        None
    }
}

//获取进程在内核态和用户态的cpu占用率
fn get_cpu_usage_by_process(sys: &LinuxSysStat, pid: i32) -> (f64, f64, Option<process::Process>) {
    if let Ok(info) = process::Process::new(pid) {
        let (start_total_system, start_total_user, start_process_system, start_process_user) = get_cpu_args(&info);

        thread::sleep(Duration::from_millis((sys.interval * 1000.0) as u64));    //间隔指定时间，再次获取cpu占用时间

        if let Ok(info) = process::Process::new(pid) {
            let (end_total_system, end_total_user, end_process_system, end_process_user) = get_cpu_args(&info);

            let total_system = end_total_system - start_total_system;
            let total_user = end_total_user - start_total_user;
            if total_system <= 0 {
                if total_user <= 0 {
                    return (0.0, 0.0, Some(info));
                } else {
                    return (0.0, (100.0 * (end_process_user - start_process_user) as f64) / total_user as f64, Some(info));
                }
            } else {
                let system = (100.0 * (end_process_system - start_process_system) as f64) / total_system as f64;
                if total_user <= 0 {
                    return (system, 0.0, Some(info));
                } else {
                    return (system, (100.0 * (end_process_user - start_process_user) as f64) / total_user as f64, Some(info));
                }
            }
        }
    }

    (0.0, 0.0, None)
}

//获取系统和进程在内核态和用户态的cpu占用时间
fn get_cpu_args(process: &process::Process) -> (u64, u64, i64, i64) {
    if let Ok(sys) = cpu::cpu_times() {
        return (sys.system,                                         //系统内核态cpu占用时间，单位tick
                sys.user + sys.nice,                                //系统用户态cpu占用时间，单位tick
                process.stime_ticks as i64 + process.cstime_ticks,  //进程内核态cpu占用时间，单位tick
                process.utime_ticks as i64 + process.cutime_ticks)  //进程用户态cpu占用时间，单位tick
    }

    (0, 0, 0, 0)
}

//构建指定pid的线程路径
fn threads_path(pid: i32) -> Option<PathBuf> {
    let p = PROCESS_ROOT_PATH.to_string() + &pid.to_string() + THREADS_DIR;
    let path = PathBuf::from(p);
    if !path.exists() {
        //指定路径不存在
        return None;
    }

    Some(path)
}

//访问指定pid的线程列表
fn threads(pid: i32) -> Option<Vec<i32>> {
    if let Some(path) = threads_path(pid) {
        let mut wd = WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .same_file_system(true)
            .into_iter()
            .filter_entry(|dir| {
                !is_hidden(dir) //过滤掉隐藏目录
            });

        let mut vec = Vec::new();
        for entry in wd {
            if let Ok(dir) = entry {
                vec.push(dir.file_name().to_str().unwrap().parse::<i32>().unwrap());
            }
        }

        return Some(vec);
    }

    None
}

//判断目录是否是隐藏目录
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

impl LinuxSysStat {
    /// 构建linux系统状态
    pub fn new(interval: f64) -> Self {
        LinuxSysStat {
            interval,
            disk_counter: Arc::new(RefCell::new(disk::DiskIOCountersCollector::default())),
            net_counter: Arc::new(RefCell::new(network::NetIOCountersCollector::default())),
        }
    }
}

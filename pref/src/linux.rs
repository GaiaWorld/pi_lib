extern crate psutil;

use std::thread;
use std::time::Duration;

use psutil::{system, process};

use ::SysSpecialStat;

/*
* 默认间隔时长
*/
const DEFAULT_INTERVAL: f64 = 1.0;

/*
* linux系统状态
*/
pub struct LinuxSysStat {
    interval: f64,  //采集间隔时长，单位秒
}

impl SysSpecialStat for LinuxSysStat {
    fn sys_cpu_usage(&self) -> Option<f64> {
        if let Ok(usage) = system::cpu_percent(self.interval) {
            return Some(usage);
        }

        None
    }

    fn sys_processores_usage(&self) -> Option<Vec<f64>> {
        if let Ok(usages) = system::cpu_percent_percpu(self.interval) {
            return Some(usages);
        }

        None
    }

    fn sys_cpu_detal(&self) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)> {
        if let Ok(info) = system::cpu_times_percent(self.interval) {
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

        if let Ok(infos) = system::cpu_times_percent_percpu(self.interval) {
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
        if let Ok(info) = system::loadavg() {
            return Some((info.one,          //1分钟前的负载系数
                         info.five,         //5分钟前的负载系数
                         info.fifteen));    //15分钟前的负载系数
        }

        None
    }

    fn sys_virtual_memory_detal(&self) -> Option<(u64, u64, u64, u64, u64, u64, u64, u64, u64, f32)> {
        if let Ok(info) = system::virtual_memory() {
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
        if let Ok(info) = system::swap_memory() {
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
        system::uptime()
    }

    fn process_current_pid(&self) -> i32 {
        psutil::getpid()
    }

    fn process_current_detal(&self) -> Option<(u32, u32, i64, i64, f64, f64, u64, i64, u64, u64, u64, u64, u64, i32, i64, f64, String, String)> {
        if let (sys_usage, user_usage, Some(info)) = get_cpu_usage_by_process(self, self.process_current_pid()) {
            return Some((info.uid,                  //进程所属用户id
                         info.gid,                  //进程所属组id
                         info.nice,                 //进程静态优先级，数字越小，优先级越高
                         info.priority,             //进程动态优先级，数字越小，优先级越高
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
                         info.state.to_string()));  //进程当前状态
        }

        None
    }
}

//获取进程在内核态和用户态的cpu占用率
fn get_cpu_usage_by_process(sys: &LinuxSysStat, pid: i32) -> (f64, f64, Option<process::Process>) {
    if let Ok(info) = process::Process::new(sys.process_current_pid()) {
        let (start_total_system, start_total_user, start_process_system, start_process_user) = get_cpu_args(&info);

        thread::sleep(Duration::from_micros(10000));    //间隔10ms再次获取cpu占用时间

        if let Ok(info) = process::Process::new(sys.process_current_pid()) {
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
    if let Ok(sys) = system::cpu_times() {
        return (sys.system,                                         //系统内核态cpu占用时间，单位tick
                sys.user + sys.nice,                                //系统用户态cpu占用时间，单位tick
                process.stime_ticks as i64 + process.cstime_ticks,  //进程内核态cpu占用时间，单位tick
                process.utime_ticks as i64 + process.cutime_ticks)  //进程用户态cpu占用时间，单位tick
    }

    (0, 0, 0, 0)
}

impl LinuxSysStat {
    //构建linux系统状态
    pub fn new(interval: f64) -> Self {
        LinuxSysStat {
            interval,
        }
    }
}

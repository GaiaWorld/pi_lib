extern crate pref;

use std::time;
use std::thread;

use pref::common::{NetIPType, NetProtocolType, SysStat};
#[cfg(any(unix))]
use pref::SysSpecialStat;
#[cfg(any(unix))]
use pref::linux::LinuxSysStat;

#[test]
fn test_common() {
    let sys = SysStat::new();

    println!("processor count: {}", sys.processor_count());

    println!("cpu usage: {}", sys.cpu_usage());

    for n in 0..sys.processor_count() {
        println!("processor #{} usage: {}", n, sys.processor_usage(n));
    }

    let usage = sys.processores_usage();
    println!("cpu usage: {}", usage.0);
    for u in usage.1 {
        println!("processor usage: {}", u);
    }

    let usage = sys.memory_usage();
    println!("total memory: {}KB", usage.0);
    println!("free memory: {}KB", usage.1);
    println!("used memory: {}KB", usage.2);
    println!("total swap: {}KB", usage.3);
    println!("free swap: {}KB", usage.4);
    println!("used swap: {}KB", usage.5);

    println!("cuurent pid: {}", sys.current_pid());

    let info = sys.current_process_info();
    println!("current pid: {}", info.0);
    println!("process name: {}", info.1);
    println!("cwd: {:?}", info.2);
    println!("exe: {:?}", info.3);
    println!("cpu usage: {}", info.4);
    println!("memory: {}KB", info.5);
    println!("start time: {}", info.6);
    println!("status: {:?}", info.7);

    for disk in sys.disk_usage() {
        println!("mount: {:?}", disk.3);
        println!("\tname: {:?}", disk.0);
        println!("\ttype: {:?}", disk.1);
        println!("\tfile system: {:?}", disk.2);
        println!("\tfree: {:?}MB", disk.4 / 1024 / 1024);
        println!("\ttotal: {:?}MB", disk.5 / 1024 / 1024);
    }

    let usage = sys.net_io_usage();
    println!("net input: {}KB", usage.0 / 1024);
    println!("net output: {}KB", usage.1 / 1024);

    println!("sockets size: {}", sys.sockets_size(NetIPType::All, NetProtocolType::All));

    for socket in sys.sockets_info(NetIPType::All, NetProtocolType::All) {
        println!("local port: {}", socket.2);
        println!("\ttype: {:?}", socket.0);
        println!("\tlocal address: {:?}", socket.1);
        println!("\tremote address: {:?}", socket.3);
        println!("\tremote port: {:?}", socket.4);
        println!("\tstatus: {:?}", socket.5);
        println!("\tprocesses: {:?}", socket.6);
    }

    println!("cuurent pid: {}", sys.current_pid());
    for socket in sys.current_process_sockets_info(NetIPType::All, NetProtocolType::All) {
        println!("\tlocal port: {}", socket.2);
        println!("\t\ttype: {:?}", socket.0);
        println!("\t\tlocal address: {:?}", socket.1);
        println!("\t\tremote address: {:?}", socket.3);
        println!("\t\tremote port: {:?}", socket.4);
        println!("\t\tstatus: {:?}", socket.5);
        println!("\t\tprocesses: {:?}", socket.6);
    }

    println!("system uptime: {}", sys.uptime());
}

#[test]
fn test_psutil() {
    #[cfg(any(unix))]
    test_psutil_();
}

#[cfg(any(unix))]
fn test_psutil_() {
    thread::Builder::new().name("psutil001".to_string()).spawn(move || {
        //负载
        let mut count = 0;
        for _ in 0..1000000000 { count += 1; }
    });

    let sys = LinuxSysStat::new(0.2);

    //预热
    sys.sys_cpu_usage();

    if let Some(info) = sys.sys_cpu_usage() {
        println!("cpu usage: {}", info);
    }

    //预热
    sys.sys_processores_usage();

    let mut n = 0;
    if let Some(infos) = sys.sys_processores_usage() {
        for info in infos {
            println!("processor #{} usage: {}", n, info);
            n += 1;
        }
    }

    //预热
    sys.sys_cpu_detal();

    if let Some(info) = sys.sys_cpu_detal() {
        println!("cpu user usage: {}", info.0);
        println!("cpu nice usage: {}", info.1);
        println!("cpu system usage: {}", info.2);
        println!("cpu idle usage: {}", info.3);
        println!("cpu iowait usage: {}", info.4);
        println!("cpu irq usage: {}", info.5);
        println!("cpu soft irq usage: {}", info.6);
        println!("cpu steal usage: {}", info.7);
        println!("cpu guest usage: {}", info.8);
        println!("cpu guest nice usage: {}", info.9);
    }

    //预热
    sys.sys_processores_detal();

    n = 0;
    if let Some(infos) = sys.sys_processores_detal() {
        for info in infos {
            println!("processor #{}", n);
            println!("\tuser usage: {}", info.0);
            println!("\tnice usage: {}", info.1);
            println!("\tsystem usage: {}", info.2);
            println!("\tidle usage: {}", info.3);
            println!("\tiowait usage: {}", info.4);
            println!("\tirq usage: {}", info.5);
            println!("\tsoft irq usage: {}", info.6);
            println!("\tsteal usage: {}", info.7);
            println!("\tguest usage: {}", info.8);
            println!("\tguest nice usage: {}", info.9);
            n += 1;
        }
    }

    if let Some(info) = sys.sys_loadavg() {
        println!("load avg: {}, {}, {}", info.0, info.1, info.2);
    }

    if let Some(info) = sys.sys_virtual_memory_detal() {
        println!("sys total memory: {}KB", info.0 / 1024);
        println!("sys free memory: {}KB", info.1 / 1024);
        println!("sys used memory: {}KB", info.2 / 1024);
        println!("sys available memory: {}KB", info.3 / 1024);
        println!("sys active memory: {}KB", info.4 / 1024);
        println!("sys inactive memory: {}KB", info.5 / 1024);
        println!("sys buffers memory: {}KB", info.6 / 1024);
        println!("sys cached memory: {}KB", info.7 / 1024);
        println!("sys shared memory: {}KB", info.8 / 1024);
        println!("sys memory usage: {}", info.9);
    }

    if let Some(info) = sys.sys_swap_detal() {
        println!("sys total swap: {}KB", info.0 / 1024);
        println!("sys free swap: {}KB", info.1 / 1024);
        println!("sys used swap: {}KB", info.2 / 1024);
        println!("sys sin swap: {}KB", info.3 / 1024);
        println!("sys sout swap: {}KB", info.4 / 1024);
        println!("sys swap usage: {}", info.5);
    }

    println!("system uptime: {}", sys.sys_uptime());

    let pid = sys.process_current_pid();
    println!("current process: {}", pid);

    if let Some(info) = sys.process_detal(pid) {
        println!("process uid: {}", info.0);
        println!("process gid: {}", info.1);
        println!("process nice: {}", info.2);
        println!("process priority: {}", info.3);
        println!("process system cpu usage: {}", info.4);
        println!("process user cpu usage: {}", info.5);
        println!("process vm: {}KB", info.6 / 1024);
        println!("process rss: {}KB", info.7 / 1024);
        println!("process rss limit: {}KB", info.8 / 1024);
        println!("process minflt: {}", info.9);
        println!("process cminflt: {}", info.10);
        println!("process majflt: {}", info.11);
        println!("process cmajflt: {}", info.12);
        println!("process processor: {}", info.13);
        println!("process threads: {}", info.14);
        println!("process start time: {}", info.15);
        println!("process command: {}", info.16);
        println!("process status: {}", info.17);
        println!("process cwd: {:?}", info.19);
        println!("process cmd: {}", info.18);
    }

    if let Some(info) = sys.process_env(pid) {
        for (key, value) in info.iter() {
            println!("{}: {}", key, value);
        }
    }

    if let Some(info) = sys.process_memory(pid) {
        println!("process vm: {}KB", info.0 / 1024);
        println!("process total: {}KB", info.1 / 1024);
        println!("process res: {}KB", info.2 / 1024);
        println!("process share: {}KB", info.3 / 1024);
        println!("process text: {}KB", info.4 / 1024);
        println!("process data: {}KB", info.5 / 1024);
    }

    if let Some(size) = sys.process_fd_size(pid) {
        println!("process fd size: {}", size);
    }

    if let Some(infos) = sys.process_fd(pid) {
        for info in infos {
            println!("fd: {}", info.0);
            println!("\tfile: {:?}", info.1);
        }
    }

    if let Some(threads) = sys.process_threads(pid) {
        for thread in threads {
            println!("thread: {}", thread);
            if let Some(info) = sys.process_detal(thread) {
                println!("\tthread uid: {}", info.0);
                println!("\tthread gid: {}", info.1);
                println!("\tthread nice: {}", info.2);
                println!("\tthread priority: {}", info.3);
                println!("\tthread system cpu usage: {}", info.4);
                println!("\tthread user cpu usage: {}", info.5);
                println!("\tthread vm: {}KB", info.6 / 1024);
                println!("\tthread rss: {}KB", info.7 / 1024);
                println!("\tthread rss limit: {}KB", info.8 / 1024);
                println!("\tthread minflt: {}", info.9);
                println!("\tthread cminflt: {}", info.10);
                println!("\tthread majflt: {}", info.11);
                println!("\tthread cmajflt: {}", info.12);
                println!("\tthread processor: {}", info.13);
                println!("\tthread threads: {}", info.14);
                println!("\tthread start time: {}", info.15);
                println!("\tthread command: {}", info.16);
                println!("\tthread status: {}", info.17);
                println!("\tthread cwd: {:?}", info.19);
                println!("\tthread cmd: {}", info.18);
            }

            if let Some(info) = sys.process_memory(thread) {
                println!("\tthread vm: {}KB", info.0 / 1024);
                println!("\tthread total: {}KB", info.1 / 1024);
                println!("\tthread res: {}KB", info.2 / 1024);
                println!("\tthread share: {}KB", info.3 / 1024);
                println!("\tthread text: {}KB", info.4 / 1024);
                println!("\tthread data: {}KB", info.5 / 1024);
            }

            if let Some(size) = sys.process_fd_size(thread) {
                println!("\tthread fd size: {}", size);
            }

            if let Some(infos) = sys.process_fd(thread) {
                for info in infos {
                    println!("\tfd: {}", info.0);
                    println!("\t\tfile: {:?}", info.1);
                }
            }
        }

        if let Some(infos) = sys.disk_part(true) {
            for info in infos {
                println!("device: {}", info.0);
                println!("\tmount: {}", info.1);
                println!("\tfile system: {}", info.2);
                println!("\topts: {}", info.3);
                if let Some(usage) = sys.disk_usage(&info.1) {
                    println!("\tusage: {}", usage.6);
                    println!("\ttotal: {}KB", usage.0 / 1024);
                    println!("\tfree: {}KB", usage.1 / 1024);
                    println!("\tused: {}KB", usage.2 / 1024);
                    println!("\tinode total: {}", usage.3);
                    println!("\tindoe free: {}", usage.4);
                    println!("\tinode used: {}", usage.5);
                }
            }
        }

        if let Some(infos) = sys.disk_io_detal() {
            for info in infos {
                println!("disk: {}", info.0);
                println!("\trc: {}", info.1);
                println!("\twc: {}", info.2);
                println!("\trb: {}B", info.3);
                println!("\twb: {}B", info.4);
                println!("\trt: {}ms", info.5);
                println!("\twt: {}ms", info.6);
                println!("\trmc: {}", info.7);
                println!("\twmc: {}", info.8);
                println!("\tbusy: {}ms", info.9);
            }
        }

        if let Some(infos) = sys.network_io_detal() {
            for info in infos {
                println!("network interface: {}", info.0);
                println!("\tbs: {}B", info.1);
                println!("\tbr: {}B", info.2);
                println!("\tps: {}", info.3);
                println!("\tpr: {}", info.4);
                println!("\ter: {}", info.5);
                println!("\tes: {}", info.6);
                println!("\tdr: {}", info.7);
                println!("\tds: {}", info.8);
            }
        }
    }
}
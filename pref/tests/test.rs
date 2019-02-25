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
        loop {
            thread::sleep_ms(10000);
        }
    });

    let sys = LinuxSysStat::new(0.01);

    //预热
    sys.sys_cpu_usage();
    for _ in 0..100000000 {}

    if let Some(info) = sys.sys_cpu_usage() {
        println!("cpu usage: {}", info);
    }

    //预热
    sys.sys_processores_usage();
    for _ in 0..100000000 {}

    let mut n = 0;
    if let Some(infos) = sys.sys_processores_usage() {
        for info in infos {
            println!("processor #{} usage: {}", n, info);
            n += 1;
        }
    }

    //预热
    sys.sys_cpu_detal();
    for _ in 0..100000000 {}

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
    for _ in 0..100000000 {}

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
        println!("sys total memory: {}", info.0);
        println!("sys free memory: {}", info.1);
        println!("sys used memory: {}", info.2);
        println!("sys available memory: {}", info.3);
        println!("sys active memory: {}", info.4);
        println!("sys inactive memory: {}", info.5);
        println!("sys buffers memory: {}", info.6);
        println!("sys cached memory: {}", info.7);
        println!("sys shared memory: {}", info.8);
        println!("sys memory usage: {}", info.9);
    }

    if let Some(info) = sys.sys_swap_detal() {
        println!("sys total swap: {}", info.0);
        println!("sys free swap: {}", info.1);
        println!("sys used swap: {}", info.2);
        println!("sys sin swap: {}", info.3);
        println!("sys sout swap: {}", info.4);
        println!("sys swap usage: {}", info.5);
    }

    println!("system uptime: {}", sys.sys_uptime());

    println!("current process: {}", sys.process_current_pid());

    //预热
    for _ in 0..100000000 {}

    if let Some(info) = sys.process_current_detal() {
        println!("process uid: {}", info.0);
        println!("process gid: {}", info.1);
        println!("process nice: {}", info.2);
        println!("process priority: {}", info.3);
        println!("process system cpu usage: {}", info.4);
        println!("process user cpu usage: {}", info.5);
        println!("process vm: {}", info.6);
        println!("process rss: {}", info.7);
        println!("process rss limit: {}", info.8);
        println!("process minflt: {}", info.9);
        println!("process cminflt: {}", info.10);
        println!("process majflt: {}", info.11);
        println!("process cmajflt: {}", info.12);
        println!("process processor: {}", info.13);
        println!("process threads: {}", info.14);
        println!("process start time: {}", info.15);
        println!("process command: {}", info.16);
        println!("process status: {}", info.17);
    }
}
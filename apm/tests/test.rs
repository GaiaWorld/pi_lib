extern crate atom;

extern crate apm;

use std::time;
use std::thread;
use std::sync::atomic::Ordering;

use atom::Atom;

use apm::common::{NetIPType, NetProtocolType, SysStat};
#[cfg(any(unix))]
use apm::SysSpecialStat;
#[cfg(any(unix))]
use apm::linux::{LinuxSysStat, current_tid};

use apm::allocator::{CounterSystemAllocator, alloced_size};
use apm::trace::StackTracer;
use apm::counter::{GLOBAL_PREF_COLLECT};

#[global_allocator]
static ALLOCATOR: CounterSystemAllocator = CounterSystemAllocator;

#[test]
fn test_common() {
    let mut tracer = StackTracer::new();

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

    #[cfg(any(windows))]
    println!("cuurent pid: {}", sys.current_pid());
    #[cfg(any(unix))]
    println!("current pid: {}", sys.special_platform().unwrap().process_current_pid());

    #[cfg(any(windows))]
    {
        let info = sys.current_process_info();
        println!("current pid: {}", info.0);
        println!("process name: {}", info.1);
        println!("cwd: {:?}", info.2);
        println!("exe: {:?}", info.3);
        println!("cpu usage: {}", info.4);
        println!("memory: {}KB", info.5);
        println!("start time: {}", info.6);
        println!("status: {:?}", info.7);
    }

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

    #[cfg(any(windows))]
    let pid = sys.current_pid();
    #[cfg(any(unix))]
    let pid = sys.special_platform().unwrap().process_current_pid();

    println!("cuurent pid: {}", pid);
    for socket in sys.current_process_sockets_info(pid as i32, NetIPType::All, NetProtocolType::All) {
        println!("\tlocal port: {}", socket.2);
        println!("\t\ttype: {:?}", socket.0);
        println!("\t\tlocal address: {:?}", socket.1);
        println!("\t\tremote address: {:?}", socket.3);
        println!("\t\tremote port: {:?}", socket.4);
        println!("\t\tstatus: {:?}", socket.5);
        println!("\t\tprocesses: {:?}", socket.6);
    }

    println!("system uptime: {}", sys.uptime());

    println!("rust alloced size: {}B", alloced_size());

    tracer.print_stack();
}

#[test]
fn test_linux() {
    #[cfg(any(unix))]
    test_linux_();
}

#[cfg(any(unix))]
fn test_linux_() {
    thread::Builder::new().name("psutil001".to_string()).spawn(move || {
        //负载
        let mut count = 0;
        for _ in 0..1000000000 { count += 1; }
    });

    let sys_stat = SysStat::new();
    println!("processor count: {}", sys_stat.processor_count());

    let sys = sys_stat.special_platform().unwrap();

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

    println!("current thread: {}", current_tid());

    if let Some(info) = sys.process_detal(pid) {
        println!("process uid: {}", info.0);
        println!("process gid: {}", info.1);
        println!("process nice: {}", info.2);
        println!("process priority: {}", info.3);
        println!("process realtime priority: {}", info.4);
        println!("process system cpu usage: {}", info.5);
        println!("process user cpu usage: {}", info.6);
        println!("process vm: {}KB", info.7 / 1024);
        println!("process rss: {}KB", info.8 / 1024);
        println!("process rss limit: {}KB", info.9 / 1024);
        println!("process minflt: {}", info.10);
        println!("process cminflt: {}", info.11);
        println!("process majflt: {}", info.12);
        println!("process cmajflt: {}", info.13);
        println!("process processor: {}", info.14);
        println!("process threads: {}", info.15);
        println!("process start time: {}", info.16);
        println!("process command: {}", info.17);
        println!("process status: {}", info.18);
        println!("process cwd: {:?}", info.20);
        println!("process cmd: {}", info.19);
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
                println!("\tthread realtime priority: {}", info.4);
                println!("\tthread system cpu usage: {}", info.5);
                println!("\tthread user cpu usage: {}", info.6);
                println!("\tthread vm: {}KB", info.7 / 1024);
                println!("\tthread rss: {}KB", info.8 / 1024);
                println!("\tthread rss limit: {}KB", info.9 / 1024);
                println!("\tthread minflt: {}", info.10);
                println!("\tthread cminflt: {}", info.11);
                println!("\tthread majflt: {}", info.12);
                println!("\tthread cmajflt: {}", info.13);
                println!("\tthread processor: {}", info.14);
                println!("\tthread threads: {}", info.15);
                println!("\tthread start time: {}", info.16);
                println!("\tthread command: {}", info.17);
                println!("\tthread status: {}", info.18);
                println!("\tthread cwd: {:?}", info.20);
                println!("\tthread cmd: {}", info.19);
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

    println!("rust alloced size: {}B", alloced_size());
}

#[test]
fn test_counter() {
    let counter0 = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("static0"), 0).unwrap();
    let counter1 = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("static1"), 0).unwrap();
    let counter2 = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("static2"), 0).unwrap();
    let counter3 = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("static3"), 0).unwrap();
    let counter4 = GLOBAL_PREF_COLLECT.new_static_counter(Atom::from("static4"), 0).unwrap();
    GLOBAL_PREF_COLLECT.static_init_ok();

    //11个线程每秒总共采集约10100000个计数
    thread::spawn(move || {
        loop {
            for _ in 0..100 {
                counter0.sum(1);
                counter1.sum(1);
                counter2.sum(1);
                counter3.sum(1);
                counter4.sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("10000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("20000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("30000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("40000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("50000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("60000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("70000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("80000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("90000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    thread::spawn(move || {
        let mut vec = Vec::with_capacity(1000);
        for index in 0..1000 {
            vec.push(GLOBAL_PREF_COLLECT.new_dynamic_counter(Atom::from("100000".to_string() + &index.to_string()), 0).unwrap());
        }

        loop {
            for index in 0..1000 {
                vec[index].sum(1);
            }

            thread::sleep(time::Duration::from_millis(1)); //间隔1ms采集一次
        }
    });

    let mut n = 0;
    loop {
        println!("{}", n);
        let mut count = 0;
        let mut dyn_iter = GLOBAL_PREF_COLLECT.dynamic_iter();
        while let Some((_cid, _counter)) = dyn_iter.next() {
            count += 1;
//            println!("\t{}: {}", cid, counter.load(Ordering::Relaxed));
        }
        println!("\tcounter size: {}", count);

        let mut static_iter = GLOBAL_PREF_COLLECT.static_iter();
        while let Some((cid, counter)) = static_iter.next() {
            println!("\t{}: {}", cid, counter.load(Ordering::Relaxed));
        }

        n += 1;

        thread::sleep(time::Duration::from_millis(1000)); //间隔10000ms查看一次汇总
    }
}

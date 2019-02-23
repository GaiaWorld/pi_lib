extern crate pref;
#[cfg(any(unix))]
extern crate psutil;

use std::time;
use std::thread;

use pref::common::{NetIPType, NetProtocolType, GenSysStat};

#[test]
fn test_common() {
    let sys = GenSysStat::new();

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

    for proc in psutil::process::all().unwrap() {
        println!("pid: {:?}, name: {:?}, threads: {}", proc.pid, proc.comm, proc.num_threads);
    }

    let pid = psutil::getpid();
    println!("cur pid: {:?}", pid);

    for n in 1..psutil::process::Process::new(pid).unwrap().num_threads as usize {
        if let Ok(thread) = psutil::process::Process::new(pid + n as i32) {
            println!("!!!!!!thread {:?}, info: {:?}", pid + n as i32, thread);
        }

        if let Ok(mem) = psutil::process::Memory::new(pid + n as i32) {
            println!("!!!!!!thread {:?}, mem: {:?}", pid + n as i32, mem);
        }
    }
}
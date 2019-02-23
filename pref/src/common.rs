use std::sync::Arc;
use std::net::IpAddr;
use std::path::PathBuf;
use std::cell::RefCell;

use netstat::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, SocketInfo, get_sockets_info, iterate_sockets_info};
use sysinfo::{NetworkExt, System, SystemExt, ProcessorExt, ProcessExt, ProcessStatus, DiskExt};

/*
* 进程状态
*/
pub type ProcessState = ProcessStatus;

/*
* 硬盘类型
*/
pub type DiskType = sysinfo::DiskType;

/*
* TCP状态
*/
pub type TcpStatus = TcpState;

/*
* 网络连接信息
*/
type NetSocketsInfo = Vec<(NetProtocolType, IpAddr, u16, Option<IpAddr>, Option<u16>, Option<TcpStatus>, Vec<u32>)>;

/*
* 网络IP类型
*/
pub enum NetIPType {
    IPv4,
    IPv6,
    All,
}

/*
* 网络协议类型
*/
#[derive(Debug)]
pub enum NetProtocolType {
    TCP,
    UDP,
    All,
}

/*
* 通用系统状态
*/
#[derive(Clone)]
pub struct GenSysStat {
    inner: Arc<RefCell<System>>,
}

impl GenSysStat {
    //构建通用系统状态
    pub fn new() -> Self {
        GenSysStat {
            inner: Arc::new(RefCell::new(System::new())),
        }
    }

    //获取cpu逻辑核心数
    pub fn processor_count(&self) -> usize {
        self.inner.borrow_mut().refresh_system();

        let count = self.inner.borrow().get_processor_list().len();
        if count == 1 {
            return 1;
        }

        count - 1
    }

    //获取cpu占用率
    pub fn cpu_usage(&self) -> f32 {
        self.inner.borrow_mut().refresh_system();

        self.inner.borrow().get_processor_list()[0].get_cpu_usage()
    }

    //获取指定逻辑核心的占用率
    pub fn processor_usage(&self, n: usize) -> f32 {
        self.inner.borrow_mut().refresh_system();

        let inner = self.inner.borrow();
        let array = inner.get_processor_list();
        let count = array.len();
        if count == 1 && n == 0 {
            return array[n].get_cpu_usage();
        } else if count > 1 && n < count - 1 {
            return array[n].get_cpu_usage();
        }

        0.0
    }

    //获取cpu和所有逻辑核心的占用率
    pub fn processores_usage(&self) -> (f32, Vec<f32>) {
        self.inner.borrow_mut().refresh_system();

        let mut vec: Vec<f32>;
        let inner = self.inner.borrow();
        let array = inner.get_processor_list();
        let count = array.len();

        let cpu_usage = self.inner.borrow().get_processor_list()[0].get_cpu_usage();

        if count == 1 {
            vec = Vec::with_capacity(count);
            vec.push(cpu_usage);
        } else {
            vec = Vec::with_capacity(count - 1);
            for n in 1..count {
                vec.push(array[n].get_cpu_usage());
            }
        }

        (cpu_usage, vec)
    }

    //获取系统内存基础状态，单位KB
    pub fn memory_usage(&self) -> (u64, u64, u64, u64, u64, u64) {
        self.inner.borrow_mut().refresh_system();

        let inner = self.inner.borrow();
        (inner.get_total_memory(),  //系统总内存
         inner.get_free_memory(),   //系统空闲内存
         inner.get_used_memory(),   //系统已使用内存
         inner.get_total_swap(),    //系统总交换区
         inner.get_free_swap(),     //系统空闲交换区
         inner.get_used_swap())     //系统已使用交换区
    }

    //获取当前进程id
    pub fn current_pid(&self) -> usize {
        sysinfo::get_current_pid() as usize
    }

    //获取当前进程的基础状态
    pub fn current_process_info(&self) -> (usize, String, PathBuf, Vec<String>, f32, u64, u64, ProcessState) {
        let pid = sysinfo::get_current_pid() as usize;
        self.inner.borrow_mut().refresh_process(pid);

        let inner = self.inner.borrow();
        let process = inner.get_process(pid).unwrap();
        (pid,                           //当前进程id
         process.name().to_string(),    //当前进程名
         process.cwd().to_owned(),      //当前进程工作目录
         Vec::from(process.cmd()),   //当前进程指令行
         process.cpu_usage(),           //当前进程cpu占用率
         process.memory(),              //当前进程内存占用，单位KB
         process.start_time(),          //当前进程启动时间，单位秒
         process.status())              //当前进程运行状态
    }

    //获取硬盘基础状态
    pub fn disk_usage(&self) -> Vec<(String, DiskType, String, PathBuf, u64, u64)> {
        self.inner.borrow_mut().refresh_disks();

        let inner = self.inner.borrow();
        let disks = inner.get_disks();
        let mut vec = Vec::with_capacity(disks.len());

        for disk in inner.get_disks() {
            let disk_name: String;
            if let Ok(name) = disk.get_name().to_os_string().into_string() {
                disk_name = name;
            } else {
                disk_name = "".to_string();
            }

            vec.push(
                (
                        disk_name,                                                          //硬盘名
                        disk.get_type(),                                                    //硬盘类型
                        String::from_utf8_lossy(disk.get_file_system()).into_owned(),    //硬盘文件系统
                        disk.get_mount_point().to_owned(),                                  //硬盘挂载点
                        disk.get_available_space(),                                         //硬盘可用空间
                        disk.get_total_space()                                              //硬盘总空间
                    )
            );
        }

        vec
    }

    //获取网络io当前总流量，单位B
    pub fn net_io_usage(&self) -> (u64, u64) {
        self.inner.borrow_mut().refresh_network();

        let inner = self.inner.borrow();
        let net = inner.get_network();
        (net.get_income(), net.get_outcome())
    }

    //获取系统网络连接数
    pub fn sockets_size(&self, ip_type: NetIPType, protocol_type: NetProtocolType) -> usize {
        let (address_flag, protocol_flag) = filter_sockets_args(ip_type, protocol_type);

        if let Ok(mut sockets) = iterate_sockets_info(address_flag, protocol_flag) {
            return sockets.count();
        }

        0
    }

    //获取系统网络连接状态
    pub fn sockets_info(&self, ip_type: NetIPType, protocol_type: NetProtocolType) -> NetSocketsInfo {
        let mut vec = Vec::new();
        let (address_flag, protocol_flag) = filter_sockets_args(ip_type, protocol_type);

        if let Ok(sockets) = get_sockets_info(address_flag, protocol_flag) {
            for socket in sockets {
                fill_socket_info(&mut vec, socket);
            }
        }

        vec
    }

    //获取指定进程的网络连接状态
    pub fn process_sockets_info(&self, pid: usize, ip_type: NetIPType, protocol_type: NetProtocolType) -> NetSocketsInfo {
        let mut vec = Vec::new();
        let (address_flag, protocol_flag) = filter_sockets_args(ip_type, protocol_type);

        if let Ok(mut sockets) = iterate_sockets_info(address_flag, protocol_flag) {
            loop {
                if let Some(r) = sockets.next() {
                    if let Ok(socket) = r {
                        if contains_pid_sockets(pid, &socket) {
                            //有指定关联进程的socket
                            fill_socket_info(&mut vec, socket);
                        }
                    }
                } else {
                    //迭代完成
                    break;
                }
            }
        }

        vec
    }

    //获取当前进程的网络连接状态
    pub fn current_process_sockets_info(&self, ip_type: NetIPType, protocol_type: NetProtocolType) -> NetSocketsInfo {
        self.process_sockets_info(self.current_pid(), ip_type, protocol_type)
    }

    //获取系统正常运行时间，单位秒
    pub fn uptime(&self) -> u64 {
        self.inner.borrow_mut().refresh_system();

        self.inner.borrow().get_uptime()
    }
}

//过滤网络连接参数
fn filter_sockets_args(ip_type: NetIPType, protocol_type: NetProtocolType) -> (AddressFamilyFlags, ProtocolFlags) {
    let address_flag: AddressFamilyFlags;
    let protocol_flag: ProtocolFlags;
    match ip_type {
        NetIPType::IPv4 => {
            address_flag = AddressFamilyFlags::IPV4;
        },
        NetIPType::IPv6 => {
            address_flag = AddressFamilyFlags::IPV6;
        },
        NetIPType::All => {
            address_flag = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        },
    }
    match protocol_type {
        NetProtocolType::TCP => {
            protocol_flag = ProtocolFlags::TCP;
        },
        NetProtocolType::UDP => {
            protocol_flag = ProtocolFlags::UDP;
        },
        NetProtocolType::All => {
            protocol_flag = ProtocolFlags::TCP | ProtocolFlags::UDP;
        },
    }

    (address_flag, protocol_flag)
}

//检查网络连接的关联进程中是否有指定的pid
fn contains_pid_sockets(pid: usize, socket: &SocketInfo) -> bool {
    socket.associated_pids.binary_search(&(pid as u32)).is_ok()
}

//填充网络连接状态
fn fill_socket_info(vec: &mut NetSocketsInfo, socket: SocketInfo) {
    let socket_info = match &socket.protocol_socket_info {
        &ProtocolSocketInfo::Tcp(ref info) => {
            (NetProtocolType::TCP,      //协议类型
             info.local_addr,           //本地地址
             info.local_port,           //本地端口
             Some(info.remote_addr),    //远端地址
             Some(info.remote_port),    //远端端口
             Some(info.state),          //连接状态
             socket.associated_pids)    //连接关联进程
        },
        &ProtocolSocketInfo::Udp(ref info) => {
            (NetProtocolType::UDP,      //协议类型
             info.local_addr,           //本地地址
             info.local_port,           //本地端口
             None,                      //远端地址
             None,                      //远端端口
             None,                      //连接状态
             socket.associated_pids)    //连接关联进程
        },
    };
    vec.push(socket_info);
}
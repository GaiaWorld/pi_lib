/**
 * 全局唯一ID, 128字节
 * {当前的时间（纳秒ns）（8字节-584.9年），节点编号（2字节），节点启动时间(单位s)（4字节-136年），控制编号（2字节）}
 * 同一个GuidGen分配的guid，保证time不重复
 * 
 * 分布式系统可以利用控制编号来管理hash，进行一致hash命中
 */

use std::sync::atomic::{AtomicU64, Ordering};

use time::now_nanos;

// 全局唯一ID
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Guid(u128);

impl Guid {
	pub fn time(&self) -> u64 {
		(self.0 >> 64) as u64
	}
	pub fn node_id(&self) -> u16 {
		(self.0 as u64 >> 48) as u16
	}
	pub fn node_time(&self) -> u32 {
		(self.0 as u64 >> 16) as u32
	}
	pub fn ctrl_id(&self) -> u16 {
		self.0 as u16
	}
}

// Guid生成器
#[derive(Default)]
pub struct GuidGen {
	time: AtomicU64,
	node_id: u16,
	node_time: u32,
}

impl GuidGen {
	pub fn new(node_id: u16, node_time: u32) -> Self {
		GuidGen {
			time: AtomicU64::new(now_nanos()),
			node_id: node_id,
			node_time: node_time,
		}
	}
	// 分配全局唯一时间
	pub fn time(&self) -> u64 {
		let now = now_nanos();
		loop {
			let t = self.time.load(Ordering::Relaxed);
			if t < now {
				match self.time.compare_exchange(t, now, Ordering::SeqCst, Ordering::SeqCst) {
					Ok(_) => return now,
					Err(_) => ()
				}
			}else {
				return self.time.fetch_add(1, Ordering::SeqCst) + 1
			}
		}
	}
	// 分配全局唯一Guid
	pub fn gen(&self, ctrl_id: u16) -> Guid {
		let now = 0;
		loop {
			let t = self.time.load(Ordering::Relaxed);
			if t < now {
				match self.time.compare_exchange(t, now, Ordering::SeqCst, Ordering::SeqCst) {
					Ok(_) => return Guid((now as u128) << 64 | ((self.node_id as u64) << 48 | (self.node_time as u64) << 16 | ctrl_id as u64) as u128),
					Err(_) => ()
				}
			}else {
				let n = self.time.fetch_add(1, Ordering::SeqCst) + 1;
				return Guid((n as u128) << 64 | ((self.node_id as u64) << 48 | (self.node_time as u64) << 16 | ctrl_id as u64) as u128)
			}
		}
	}
}

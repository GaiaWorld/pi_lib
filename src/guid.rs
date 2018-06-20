/**
 * 全局唯一ID, 128位
 * {节点启动后的运行时间（纳秒ns）（8字节-584.9年），节点启动时间(单位s)（4字节-136年），节点编号（2字节），控制编号（2字节）}
 * 同一个GuidGen分配的guid，保证time不重复
 * 
 * 分布式系统可以利用控制编号来管理hash，进行一致hash命中
 */

use std::sync::atomic::{AtomicU64, Ordering};

use time::{now_nanos, start_secs};

// 全局唯一ID
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Guid(pub u128);

impl Guid {
	// 获取从1970年起的纳秒数
	#[inline]
	pub fn time(&self) -> u64 {
		(self.0 >> 64) as u64 + (self.0 as u64 >> 32) * 1000_000_000
	}
	#[inline]
	pub fn run_time(&self) -> u64 {
		(self.0 >> 64) as u64
	}
	#[inline]
	pub fn node_time(&self) -> u64 {
		self.0 as u64 >> 32
	}
	#[inline]
	pub fn node_id(&self) -> u16 {
		(self.0 as u32 >> 16) as u16
	}
	#[inline]
	pub fn ctrl_id(&self) -> u16 {
		self.0 as u16
	}
}

// Guid生成器
#[derive(Default)]
pub struct GuidGen {
	time: AtomicU64,
	node_time: u64,
	node_id: u16,
}

impl GuidGen {
	pub fn new(node_time: u64, node_id: u16) -> Self {
		let time = if node_time == 0 {
			start_secs()
		} else {
			node_time
		};
		GuidGen {
			time: AtomicU64::new(now_nanos()),
			node_time: time,
			node_id: node_id,
		}
	}
	pub fn node_time(&self) -> u64 {
		self.node_time
	}
	pub fn node_id(&self) -> u16 {
		self.node_id
	}
	// 分配全局唯一时间
	#[inline]
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
	#[inline]
	pub fn gen(&self, ctrl_id: u16) -> Guid {
		let now = now_nanos();
		loop {
			let t = self.time.load(Ordering::Relaxed);
			if t < now {
				match self.time.compare_exchange(t, now, Ordering::SeqCst, Ordering::SeqCst) {
					Ok(_) => return Guid((now as u128) << 64 | (self.node_time << 32 | (self.node_id as u64) << 16 | ctrl_id as u64) as u128),
					Err(_) => ()
				}
			}else {
				let n = self.time.fetch_add(1, Ordering::SeqCst) + 1;
				return Guid((n as u128) << 64 | (self.node_time << 32 | (self.node_id as u64) << 16 | ctrl_id as u64) as u128)
			}
		}
	}
}

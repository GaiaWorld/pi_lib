#![feature(integer_atomics)]
/**
 * 全局唯一ID, 64位
 * {1970年的时间（ms）（6字节-49.7天），节点编号（2字节）}
 * 同一个GuidGen分配的guid，保证time不重复
 *
 * 分布式系统可以利用控制编号来管理hash，进行一致hash命中
 */
extern crate time;

use std::sync::atomic::{AtomicU64, Ordering};

use time::{run_millis, now_millisecond};

// 全局唯一ID
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Guid(pub u64);

impl Guid {
	// 获取从1970年起的毫秒
	#[inline]
	pub fn time(&self) -> u64 {
		self.0 >> 16
	}
	#[inline]
	pub fn node_id(&self) -> u16 {
		self.0 as u16
	}

}

/**
* 全局唯一id生成器
*/
#[derive(Default, Debug)]
pub struct GuidGen {
	time: AtomicU64, // 启动后的运行时间， 单位毫秒
	node_start_ms: u64,
	node_id: u16,
}

impl GuidGen {
	/**
	* 构建全局唯一id生成器
	* @param node_start_ms 本地节点的启动时间，单位豪秒
	* @param node_id 本地节点编号
	* @returns 返回全局唯一id生成器
	*/
	pub fn new(node_start_ms: u64, node_id: u16) -> Self {
		let sec = if node_start_ms == 0 {
			now_millisecond()
		} else {
			node_start_ms
		};
		GuidGen {
			time: AtomicU64::new(run_millis()),
			node_start_ms: sec,
			node_id: node_id,
		}
	}
	// 返回启动UTC时间 单位毫秒
	pub fn node_time(&self) -> u64 {
		self.node_start_ms
	}
	pub fn node_id(&self) -> u16 {
		self.node_id
	}
	// 分配全局唯一毫秒时间
	#[inline]
	pub fn time(&self) -> u64 {
		let now = run_millis();
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
	pub fn gen(&self) -> Guid {
		let t = self.time() + self.node_start_ms;
		Guid(t << 16 | self.node_id as u64)
	}
}

#[test]
	fn test_guid() {
		use std::collections::HashMap;
		let guid = GuidGen::new(0, 0);
		
		let mut map = HashMap::new();
		let mut i = 1000000;
		while i > 0 {
			let uuid = guid.gen().0;
			map.insert(uuid, "");
			i = i - 1;
		}
		assert_eq!(map.len(), 1000000);

	}

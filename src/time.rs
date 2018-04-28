/**
 * 时间函数库
 * 提供全局时间算法
 * 时间同步算法！启动时间也要记录。
第1次的时间校准：100s +500ms
第2次的时间校准：200s +400ms，这时需要后退100ms，预计在50s内消化这个100ms，算法为：
T = Now - LastTime;
if
	T >= FixFixTime ->
		T + Fix;
	true ->
		T + Fix - ((FixFixTime - T) * FixFix div FixFixTime)
end
Now 200s
 */

use std::time::Instant;
use std::sync::{Once, ONCE_INIT};

static INIT: Once = ONCE_INIT;

static mut START: Option<Instant> = None;

pub fn now_nanos() -> u64 {
	let d = get().elapsed();
	d.as_secs() + d.subsec_nanos() as u64
}
pub fn now_micros() -> u64 {
	let d = get().elapsed();
	d.as_secs() + d.subsec_micros() as u64
}
pub fn now_millis() -> u64 {
	let d = get().elapsed();
	d.as_secs() + d.subsec_millis() as u64
}

fn get() -> Instant {
	unsafe {
		INIT.call_once(|| {
			START = Some(Instant::now());
		});
		START.unwrap()
	}
}

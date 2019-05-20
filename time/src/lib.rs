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
#[macro_use]
extern crate lazy_static;

use std::time::Instant;
use std::time::SystemTime;


/*
* 获取当前本地时间的秒数
*/
pub fn now_second() -> i64 {
	get_time().sec
}

/*
* 获取当前本地时间的毫秒数
*/
pub fn now_millisecond() -> i64 {
    let time = get_time();
	time.sec * 1000 + (time.nsec / 1000000) as i64
}

/*
* 获取当前本地时间的微秒数
*/
pub fn now_microsecond() -> i64 {
    let time = get_time();
	time.sec * 1000000 + (time.nsec / 1000) as i64
}

/*
* 获取当前本地时间的纳秒数
*/
pub fn now_nanosecond() -> i128 {
    let time = get_time();
    (time.sec * 1000000000) as i128 + time.nsec as i128
}

lazy_static! {
	static ref START: Instant = Instant::now();
	static ref START_SECS: u64 = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs();
}

// 启动后运行的纳秒数
pub fn now_nanos() -> u64 {
	let d = START.elapsed();
	d.as_secs() * 1000_000_000 + d.subsec_nanos() as u64
}
// 启动后运行的微秒数
pub fn now_micros() -> u64 {
	let d = START.elapsed();
	d.as_secs() * 1000_000 + d.subsec_micros() as u64
}
// 启动后运行的毫秒数
pub fn now_millis() -> u64 {
	let d = START.elapsed();
	d.as_secs() * 1000 + d.subsec_millis() as u64
}
// 启动后运行的秒数
pub fn now_second() -> u64 {
	START.elapsed().as_secs()
}
// 当前进程的启动时间，单位：秒
pub fn start_secs() -> u64 {
	*START_SECS
}
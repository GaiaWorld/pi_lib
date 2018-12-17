/**
 * 通用函数库
 */
extern crate time;
extern crate libc;
extern crate lz4;

use std::vec::Vec;
use std::sync::Arc;
use std::io::{Result as IoResult, Read, Write, ErrorKind};

use libc::c_void;
use lz4::{BlockSize, BlockMode, ContentChecksum, EncoderBuilder, Decoder};

/*
* 获取当前本地时间的秒数
*/
pub fn now_second() -> i64 {
	time::get_time().sec
}

/*
* 获取当前本地时间的毫秒数
*/
pub fn now_millisecond() -> i64 {
    let time = time::get_time();
	time.sec * 1000 + (time.nsec / 1000000) as i64
}

/*
* 获取当前本地时间的微秒数
*/
pub fn now_microsecond() -> i64 {
    let time = time::get_time();
	time.sec * 1000000 + (time.nsec / 1000) as i64
}

/*
* 获取当前本地时间的纳秒数
*/
pub fn now_nanosecond() -> i128 {
    let time = time::get_time();
    (time.sec * 1000000000) as i128 + time.nsec as i128
}

/*
* 将box转换为*const c_void
*/
#[inline]
pub fn box2void<T>(ptr_box: Box<T>) -> *const c_void {
    Box::into_raw(ptr_box) as *const c_void
}

/*
* 将*mut c_void转换为box
*/
#[inline]
pub fn void2box<T>(ptr_void: *mut c_void) -> Box<T> {
    unsafe { Box::from_raw(ptr_void as *mut T) }
}

/*
* 将Arc转换为*const c_void
*/
#[inline]
pub fn arc2void<T>(ptr_box: Arc<T>) -> *const c_void {
    Arc::into_raw(ptr_box) as *const c_void
}

/*
* 将*mut c_void转换为Arc
*/
#[inline]
pub fn void2arc<T>(ptr_void: *mut c_void) -> Arc<T> {
    unsafe { Arc::from_raw(ptr_void as *mut T) }
}

/*
* 将*const c_void转换为usize
*/
#[inline]
pub fn void2usize(ptr_void: *const c_void) -> usize {
    ptr_void as usize
}

/*
* 将usize转换为*const c_void
*/
#[inline]
pub fn usize2void(ptr: usize) -> *const c_void {
    ptr as *const c_void
}

/*
* 压缩级别
*/
pub enum CompressLevel {
    Low = 0x1,
    Mid = 0x5,
    High = 0xa,
}

/*
* 同步压缩指定的二进制数据
*/
pub fn compress(src: &[u8], dst: &mut Vec<u8>, level: CompressLevel) -> IoResult<()> {
    dst.truncate(0);
    EncoderBuilder::new()
                    .block_size(BlockSize::Max64KB)
                    .block_mode(BlockMode::Linked)
                    .checksum(ContentChecksum::ChecksumEnabled)
                    .level(level as u32)
                    .auto_flush(true)
                    .build(dst)
                    .and_then(|mut encoder| {
                        encoder.write_all(src)?;
                        let (_, result) = encoder.finish();
                        result
                    })
}

/*
* 同步解压指定的二进制数据
*/
pub fn uncompress(src: &[u8], dst: &mut Vec<u8>) -> IoResult<()> {
    dst.truncate(0);
    Decoder::new(src)
            .and_then(|mut decoder| {
                loop {
                    match decoder.read_to_end(dst) {
                        Ok(_) => {
                            ()
                        },
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                            //重复读
                            continue;
                        },
                        Err(e) => {
                            return Err(e);
                        },
                    }
                    let (_, result) = decoder.finish();
                    return result;
                }
            })
}

pub type Bin = Arc<Vec<u8>>;

pub type SResult<T> = Result<T, String>;
pub type OptResult = Option<SResult<()>>;

pub type Callback = Arc<Fn(SResult<()>)>;
pub type ReadCallback = Arc<Fn(SResult<Bin>)>;


// 为Vec增加的新方法
pub trait VecIndex {
	type Item;
	fn index(&self, item: &Self::Item) -> Option<usize>;
	fn swap_delete(&mut self, item: &Self::Item) -> Option<Self::Item>;
}

impl<T: PartialEq> VecIndex for Vec<T> {
	type Item = T;
	#[inline]
	fn index(&self, item: &T) -> Option<usize> {
		self.iter().position(|x| *x == *item)
	}
	#[inline]
	fn swap_delete(&mut self, item: &T) -> Option<T> {
		match self.index(item) {
			Some(i) => Some(self.swap_remove(i)),
			_ => None,
		}
	}
}

#[inline]
pub fn err_map<T, E: ToString>(err: Result<T, E>) -> Result<T, String>{
	match err {
		Ok(o) => Ok(o),
		Err(e) => Err(e.to_string())
	}
}

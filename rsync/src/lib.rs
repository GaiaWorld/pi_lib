//! 数据同步,可以计算数据的差异，并根据差异和旧数据构建出新的数据
//! 算法原理参考http://blog.csdn.net/russell_tao/article/details/7240661
//!
//! 本库使用crc32代替MD5
//! 在上述的连接中，客户端与服务器的数据同步过程可以看做：
//! 客户端根据当前数据计算crc32+校验和并发送给服务器 -> 服务器根据该crc32+校验和，与自身数据比较,得到数据差异 -> 服务器发送数据差异给客户端 -> 客户端收到差异，根据差异和自身数据还原出服务器上的最新数据。

#![feature(nll)]

extern crate crc;
extern crate adler32;

use std::collections::HashMap;

use adler32::RollingAdler32;
use crc::crc32::{Digest, IEEE};
use crc::Hasher32;

/// 远程同步，用于计算数据的差异以及根据差异和旧数据构建出新的数据
/// # Examples
/// ```
///    let oldStr = "qwertyuiopasdfghjklzxcvbnmrrrrrrrrrrrefregrtytfessfrer";
///    let newStr = "qzetyuioasdfgzjklzxcwbkmrerrrrrrrrrrrtghgtrrrrrfgsfrer";
///    let oldBytes = String::from(oldStr).into_bytes();
///    let newBytes = String::from(newStr).into_bytes();
///
///    let rsync = RSync::new(5);
///    let check_sums = rsync.check_sum(&oldBytes); // 根据旧数据，算出校验和
///    let diffs = rsync.diff(&newBytes, check_sums); // 根据旧数据的校验和、新数据，算出新数据与旧数据的差异
///    let r = rsync.sync(&oldBytes, diffs); // 根据旧数据和差异，还原为新的数据
///
///    let s = String::from_utf8(r).expect("-----------------------------");
///    assert_eq!(&s, newStr); 
/// ```
pub struct RSync{
	size: usize
}

impl RSync{
	/// 创建远程同步实例
	/// block_size为块大小设置，单位：字节，如果设置为0，表示使用默认值，默认值为64字节
	pub fn new(block_size: usize) -> RSync{
		RSync{
			size: match block_size{
				0 => 64,
				_ => block_size
			}
		}
	}

	/// 计算校验和
	pub fn check_sum(&self, data: &[u8]) -> Vec<CheckSum>{
		let data_len = data.len();
		let block_size = self.size; // 块大小
		// 从第0块开始，计算每块的MD5和Alder32校验和
		let mut block_index: usize = 0;
		let mut start = 0;
		let mut end = block_size;
		if end > data_len {
			end = data_len;
		}
		
		let mut results: Vec<CheckSum> = Vec::new(); // 计算出的校验和结果
		let mut result; // 当前块的校验和

		let mut d = Digest::new(IEEE);
		while start < data_len{
			let chunk  = &data[start..end];
			d.reset();
			d.write(chunk);
			let weak = RollingAdler32::from_buffer(chunk).hash(); // 当前块的adler32值
			let strong = d.sum32(); // 当前块的crc32值

			// 当前块的校验和放入最终校验和数组中
			result = CheckSum{weak: weak, strong: strong, index: block_index};
			results.push(result);

			// 索引到下一个块
			block_index += 1;
			start += block_size;
			end += block_size;
			if end > data_len {
				end = data_len;
			}
		}

		return results;
	}

	/// 根据新的数据和旧的校验和，计算新旧数据的差异
	/// 返回数据差异
	pub fn diff(&self, new_data: &[u8], old_check_sums: Vec<CheckSum>) -> Vec<Diff>{
		let mut results = Vec::new(); // 数据差异

		let block_size = self.size; // 块大小
		let len = new_data.len();

		let mut start = 0;
		let mut end = block_size;
		if end > len {
			end = len;
		}

		let mut hashtable = create_hashtable(old_check_sums);
		let mut crc = Digest::new(IEEE);

		let mut last_matched_end = 0;
		let mut prev_rolling_weak: Option<RollingAdler32> = None; // 上次的adler32
		
		let mut weak;
		// let mut weak_16;
		while end <= len {
			// 如果上一次已经计算出一个adler32，则根据上一次的adler32计算当前块的adler32很快
			// 否则，需要计算当前块所有字节的adler32
			match &mut prev_rolling_weak {
				&mut Some(ref mut v) => {
					v.remove(block_size, new_data[start - 1]); 
					v.update(new_data[end]);
					weak = v.hash();
				},
				None => {let v = RollingAdler32::from_buffer(&new_data[start..end]);
					weak = v.hash();
				},
			};
			// weak_16 = weak16(weak); // 为什么计算weak_16？？

			// 找到相同weak的所有校验和，如果存在一个，与本块的强校验（crc32）相等，则认为是相同的块
			let check_sums = hashtable.get_mut(&weak);
			let mut match_check_sum: Option<&CheckSum> = None;
			let mut strong_number: Option<u32> = None;
			if let Some(check_sums) = check_sums {
				for check_sum in check_sums.iter() {
					// 如果adler32相等，则
					let might_match = check_sum; // 可能匹配的校验和
					let strong = match strong_number {
						Some(r) => r,
						None => {
							let chunk = &new_data[start..end];
							crc.reset();
							crc.write(chunk);
							let r = crc.sum32();
							strong_number = Some(r);
							r
						}
					};
					
					if might_match.strong == strong {
						match_check_sum = Some(&might_match);
						break;
					};
				}
			}
			match match_check_sum {
				Some(r) => {
					// 如果当前块存在与之匹配的块，则上一次匹配块的结束未知到当前块的开始未知的数据为差异部分
					// 记录下差异的数据，以及该差异数据位于第“index”块之前
					let d = match start > last_matched_end {
						true => Some(Vec::from(&new_data[last_matched_end..start])),
						false => None,
					};
					results.push(Diff{index: Some(r.index), data: d});
					last_matched_end = end;
					start = end;
					end += block_size;
					prev_rolling_weak = None;
				},
				None => {
					start += 1;
					end += 1;
				}
			}
		}
		if last_matched_end < len {
			// index为none表示结尾数据
			results.push(Diff{
				index: None,
				data: Some(Vec::from(&new_data[last_matched_end..len]))
			});
		}
		results
	}

	/// 同步数据
	/// 根据数据差异和旧数据，得到新数据
	pub fn sync(&self, old_data: &[u8], diffs: Vec<Diff>) -> Vec<u8>{
		let mut synced = Vec::new(); // 新数据
		for chunk in diffs.into_iter() {
			match chunk.data {
				Some(v) => {
					synced.extend_from_slice(&v);
					if let Some(i) = chunk.index {
						synced.extend_from_slice(rawslice(old_data, i, self.size));
					}
				},
				None => synced.extend_from_slice(rawslice(old_data, chunk.index.unwrap(), self.size))
			}
		}

		return synced;
	}
}

pub struct CheckSum{
	index: usize,
	weak: u32,
	strong: u32,
}

pub struct Diff{
	index: Option<usize>,
	data: Option<Vec<u8>>,
}

// //以校验和的弱校验值为key，创建映射表
// fn create_hashtable(check_sums: Vec<CheckSum>) -> HashMap<u16, Vec<CheckSum>> {
// 	let mut map: HashMap<u16, Vec<CheckSum>> = HashMap::new();
// 	for check_sum in check_sums.into_iter() {
// 		let weak16 = weak16(check_sum.weak);
// 		match map.get_mut(&weak16) {
// 			Some(arr) => arr.push(check_sum),
// 			None => {map.insert(weak16, vec![check_sum]);},
// 		};
// 	}
// 	map
// }

//以校验和的弱校验值为key，创建映射表
fn create_hashtable(check_sums: Vec<CheckSum>) -> HashMap<u32, Vec<CheckSum>> {
	let mut map: HashMap<u32, Vec<CheckSum>> = HashMap::new();
	for check_sum in check_sums.into_iter() {
		let weak = check_sum.weak;
		match map.get_mut(&weak) {
			Some(arr) => arr.push(check_sum),
			None => {map.insert(weak, vec![check_sum]);},
		};
	}
	map
}

// fn weak16(weak: u32) -> u16 {
// 	(weak >> 16) as u16
// }

fn rawslice(raw: &[u8], index: usize, chunk_size: usize) -> &[u8] {
	let start = index*chunk_size;
	let len = raw.len();
	let end = match start + chunk_size > len{
		true => len,
		false => start + chunk_size,
	};
	&raw[start..end]
}

#[test]
fn test(){
	let str1 = "qwertyuiopasdfghjklzxcvbnmrrrrrrrrrrrefregrtytfessfrer";
	let str2 = "qzetyuioasdfgzjklzxcwbkmrerrrrrrrrrrrtghgtrrrrrfgsfrer";
	let bytes1 = String::from(str1).into_bytes();
	let bytes2 = String::from(str2).into_bytes();
	let rsync = RSync::new(5);
	let check_sums = rsync.check_sum(&bytes1);
	let diffs = rsync.diff(&bytes2, check_sums);
	let r = rsync.sync(&bytes1, diffs);
	let s = String::from_utf8(r).expect("-----------------------------");
	assert_eq!(&s, str2);
}
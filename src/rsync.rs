/*!
* 数据同步,可以计算数据的差异，并根据差异和原有数据构建出新的数据
* 算法原理参考http://blog.csdn.net/russell_tao/article/details/7240661
*/

use std::collections::HashMap;

use adler32::RollingAdler32;
use crc::crc32::{Digest, IEEE};
use crc::Hasher32;

pub struct RSync{
	size: usize
}

impl RSync{
	pub fn new(block_size: usize) -> RSync{
		RSync{
			size: match block_size{
				0 => 64,
				_ => block_size
			}
		}
	}

	//计算差异
	pub fn diff(&self, new_data: &[u8], old_check_sums: Vec<CheckSum>) -> Vec<Diff>{
		let len = new_data.len();
		let mut results = Vec::new();
		let mut start = 0;
		let mut end = match self.size > len {
			true => len,
			false => self.size
		};
		let mut last_matched_end = 0;
		let mut prev_rolling_weak: Option<RollingAdler32> = None;
		let mut hashtable = create_hashtable(old_check_sums);
		let mut crc = Digest::new(IEEE);
		
		let mut weak;
		let mut weak_16;
		while end <= len {
			match &mut prev_rolling_weak {
				&mut Some(ref mut v) => {
					v.remove(self.size, new_data[start]); 
					v.update(new_data[end]);
					weak = v.hash();
				},
				None => {let v = RollingAdler32::from_buffer(&new_data[start..end]);
					weak = v.hash();
				},
			};
			weak_16 = weak16(weak);

			let check_sums = hashtable.get_mut(&weak_16);
			match check_sums {
				Some(check_sums) => {
					for check_sum in check_sums.iter() {
						if check_sum.weak == weak {
							let might_match = check_sum;
							let chunk = &new_data[start..end];
							crc.reset();
							crc.write(chunk);
							let strong = crc.sum32();
							match might_match.strong == strong {
								true => {
									let d = match start > last_matched_end {
										true => Some(Vec::from(&new_data[last_matched_end..start])),
										false => None,
									};
									results.push(Diff{index: Some(might_match.index), data: d});
									start = end;
									last_matched_end = end;
									end += self.size;
									prev_rolling_weak = None;
									break;
								},
								false => (),
							};
						}
						start += 1;
						end += 1;
					}
				},
				None => {
					start += 1;
					end += 1;
				}
			}
		}
		if last_matched_end < len {
			results.push(Diff{
				index: None,
				data: Some(Vec::from(&new_data[last_matched_end..len]))
			});
		}
		results
	}

	//同步数据
	pub fn sync(&self, old_data: &[u8], diffs: Vec<Diff>) -> Vec<u8>{
		let mut synced = Vec::new();
		for chunk in diffs.into_iter() {
			match chunk.data{
				Some(v) => {
					synced.extend_from_slice(&v);
					match chunk.index{
						Some(i) => synced.extend_from_slice(rawslice(old_data, i, self.size)),
						None => ()
					};
				},
				None => synced.extend_from_slice(rawslice(old_data, chunk.index.unwrap(), self.size))
			}
		}

		return synced;
	}

	//计算校验和
	pub fn check_sum(&self, data: &[u8]) -> Vec<CheckSum>{
		let data_len = data.len();
		let incr = self.size;
		let mut start = 0;
		let mut end = match incr > data_len{
			true => data_len,
			false => incr
		};
		let mut block_index: usize = 0;
		let mut results: Vec<CheckSum> = Vec::new();
		let mut result;

		let mut d = Digest::new(IEEE);
		while start < data_len{
			let chunk  = &data[start..end];
			let weak = RollingAdler32::from_buffer(chunk).hash();
			d.reset();
			d.write(chunk);
			let strong = d.sum32();

			result = CheckSum{weak: weak, strong: strong, index: block_index};
			results.push(result);
			start += incr;
			end = match (end + incr) > data_len{
				true => data_len,
				false => end + incr,
			};
			block_index += 1;
		}

		return results;
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

//以校验和的弱校验值为key，创建映射表
fn create_hashtable(check_sums: Vec<CheckSum>) -> HashMap<u16, Vec<CheckSum>> {
	let mut map: HashMap<u16, Vec<CheckSum>> = HashMap::new();
	for check_sum in check_sums.into_iter() {
		let weak16 = weak16(check_sum.weak);
		let cs = map.get_mut(&weak16);
		match cs {
			Some(arr) => arr.push(check_sum),
			None => {map.insert(weak16, vec![check_sum]);},
		};
	}
	map
}

fn weak16(weak: u32) -> u16 {
	(weak >> 16) as u16
}

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
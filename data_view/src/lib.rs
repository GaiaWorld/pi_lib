//! 视图。
//! 可以从 二进制对象中读写多种数值类型的底层接口,使用它时,不用考虑不同平台的字节序问题。
//! 此外，额外提供一个名为move_part的接口，可以将二进制的一个片段拷贝到改二进制的另一个位置

use std::vec::Vec;
use std::ops::Range;
use std::mem::transmute;

pub trait GetView {
	fn get_u8(&self, usize) -> u8;

	fn get_lu16(&self, usize) -> u16;

	fn get_lu32(&self, usize) -> u32;

	fn get_lu64(&self, usize) -> u64;

	fn get_lu128(&self, usize) -> u128;

	fn get_li8(&self, usize) -> i8;

	fn get_li16(&self, usize) -> i16;

	fn get_li32(&self, usize) -> i32;

	fn get_li64(&self, usize) -> i64;

	fn get_li128(&self, usize) -> i128;

	fn get_lf32(&self, usize) -> f32;

	fn get_lf64(&self, usize) -> f64;

	fn get_bu16(&self, usize) -> u16;

	fn get_bu32(&self, usize) -> u32;

	fn get_bu64(&self, usize) -> u64;

	fn get_bu128(&self, usize) -> u128;

	fn get_bi8(&self, usize) -> i8;

	fn get_bi16(&self, usize) -> i16;

	fn get_bi32(&self, usize) -> i32;

	fn get_bi64(&self, usize) -> i64;

	fn get_bi128(&self, usize) -> i128;

	fn get_bf32(&self, usize) -> f32;

	fn get_bf64(&self, usize) -> f64;
}

pub trait SetView {
	fn set_u8(&mut self, u8, usize);

	fn set_lu16(&mut self, u16, usize);

	fn set_lu32(&mut self, u32, usize);

	fn set_lu64(&mut self, u64, usize);

	fn set_lu128(&mut self, u128, usize);

	fn set_i8(&mut self, i8, usize);

	fn set_li16(&mut self, i16, usize);

	fn set_li32(&mut self, i32, usize);

	fn set_li64(&mut self, i64, usize);

	fn set_li128(&mut self, i128, usize);

	fn set_lf32(&mut self, f32, usize);

	fn set_lf64(&mut self, f64, usize);

	fn set_bu16(&mut self, u16, usize);

	fn set_bu32(&mut self, u32, usize);

	fn set_bu64(&mut self, u64, usize);

	fn set_bu128(&mut self, u128, usize);

	fn set_bi16(&mut self, i16, usize);

	fn set_bi32(&mut self, i32, usize);

	fn set_bi64(&mut self, i64, usize);

	fn set_bi128(&mut self, i128, usize);

	fn set_bf32(&mut self, f32, usize);

	fn set_bf64(&mut self, f64, usize);

	fn set(&mut self, &[u8], usize);

	fn move_part(&mut self, Range<usize>, usize);
}

impl GetView for [u8] {
	fn get_u8(&self, offset: usize) -> u8{
		unsafe { *(self.as_ptr().wrapping_offset(offset as isize) as *const u8) }
	}

	fn get_lu16(&self, offset: usize) -> u16{
		let r: (u8, u8) = (self[offset], self[offset + 1]);
		u16::from_le(unsafe {transmute(r)})
		// unsafe { u16::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const u16)) }
	}

	fn get_lu32(&self, offset: usize) -> u32{
		unsafe { u32::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const u32)) }
	}

	fn get_lu64(&self, offset: usize) -> u64{
		unsafe { u64::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const u64)) }
	}

	fn get_lu128(&self, offset: usize) -> u128{
		unsafe { u128::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const u128)) }
	}
	fn get_li8(&self, offset: usize) -> i8{
		unsafe { i8::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const i8)) }
	}

	fn get_li16(&self, offset: usize) -> i16{
		unsafe { i16::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const i16)) }
	}

	fn get_li32(&self, offset: usize) -> i32{
		unsafe { i32::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const i32)) }
	}

	fn get_li64(&self, offset: usize) -> i64{
		unsafe { i64::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const i64)) }
	}
	fn get_li128(&self, offset: usize) -> i128{
		unsafe { i128::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const i128)) }
	}

	fn get_lf32(&self, offset: usize) -> f32{
		unsafe { transmute::<u32, f32>(u32::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const u32))) }
	}

	fn get_lf64(&self, offset: usize) -> f64{
		unsafe { transmute::<u64, f64>(u64::from_le(*(self.as_ptr().wrapping_offset(offset as isize) as *const u64)))  }
	}

	fn get_bu16(&self, offset: usize) -> u16{
		unsafe { u16::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const u16)) }
	}

	fn get_bu32(&self, offset: usize) -> u32{
		unsafe { u32::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const u32)) }
	}

	fn get_bu64(&self, offset: usize) -> u64{
		unsafe { u64::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const u64)) }
	}
	fn get_bu128(&self, offset: usize) -> u128{
		unsafe { u128::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const u128)) }
	}

	fn get_bi8(&self, offset: usize) -> i8{
		unsafe { *(self.as_ptr().wrapping_offset(offset as isize) as *const i8) }
	}

	fn get_bi16(&self, offset: usize) -> i16{
		unsafe { i16::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const i16)) }
	}

	fn get_bi32(&self, offset: usize) -> i32{
		unsafe { i32::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const i32)) }
	}

	fn get_bi64(&self, offset: usize) -> i64{
		unsafe { i64::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const i64)) }
	}
	fn get_bi128(&self, offset: usize) -> i128{
		unsafe { i128::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const i128)) }
	}
	fn get_bf32(&self, offset: usize) -> f32{
		unsafe { transmute::<u32, f32>(u32::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const u32))) }
	}

	fn get_bf64(&self, offset: usize) -> f64{
		unsafe { transmute::<u64, f64>(u64::from_be(*(self.as_ptr().wrapping_offset(offset as isize) as *const u64)))  }
	}
}

impl SetView for Vec<u8> {
	fn set_u8(&mut self, v: u8, offset: usize){
		unsafe { 
			let l = self.len();
			self.set_len(l + 1);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u8) = v.to_le() 
		}
	}

	fn set_lu16(&mut self, v: u16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u16) = v.to_le()
		}
	}

	fn set_lu32(&mut self, v: u32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = v.to_le()
		}
	}

	fn set_lu64(&mut self, v: u64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = v.to_le()
		}
	}

	fn set_lu128(&mut self, v: u128, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 16);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u128) = v.to_le()
		}
	}
	fn set_i8(&mut self, v: i8, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 1);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i8) = v.to_le()
		}
	}

	fn set_li16(&mut self, v: i16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i16) = v.to_le()
		}
	}

	fn set_li32(&mut self, v: i32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i32) = v.to_le()
		}
	}

	fn set_li64(&mut self, v: i64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i64) = v.to_le()
		}
	}

	fn set_li128(&mut self, v: i128, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 16);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i128) = v.to_le()
		}
	}

	fn set_lf32(&mut self, v: f32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = transmute::<f32, u32>(v).to_le()
		}
	}

	fn set_lf64(&mut self, v: f64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = transmute::<f64, u64>(v).to_le()
		}
	}


	fn set_bu16(&mut self, v: u16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u16) = v.to_be()
		}
	}

	fn set_bu32(&mut self, v: u32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = v.to_be()
		}
	}

	fn set_bu64(&mut self, v: u64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = v.to_be()
		}
	}

	fn set_bu128(&mut self, v: u128, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 16);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u128) = v.to_be()
		}
	}

	fn set_bi16(&mut self, v: i16, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 2);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i16) = v.to_be()
		}
	}

	fn set_bi32(&mut self, v: i32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i32) = v.to_be()
		}
	}

	fn set_bi64(&mut self, v: i64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i64) = v.to_be()
		}
	}

	fn set_bi128(&mut self, v: i128, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 16);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut i128) = v.to_be()
		}
	}

	fn set_bf32(&mut self, v: f32, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 4);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u32) = transmute::<f32, u32>(v).to_be()
		}
	}

	fn set_bf64(&mut self, v: f64, offset: usize){
		unsafe {
			let l = self.len();
			self.set_len(l + 8);
			*(self.as_mut_ptr().wrapping_offset(offset as isize) as *mut u64) = transmute::<f64, u64>(v).to_be()
		}
	}


	fn set(&mut self, data: &[u8], offset: usize) {
		unsafe{ 
			let len = self.len();
			let dl = data.len();
			if len < offset + dl{
				self.set_len(offset + dl);
			}
			data.as_ptr().copy_to(self.as_mut_ptr().wrapping_offset(offset as isize), dl)
		}
	}

	fn move_part(&mut self, range: Range<usize>, offset: usize) {
		unsafe{
			let len = self.len();
			let dl = range.end - range.start;
			if len < offset + dl{
				self.set_len(offset + dl);
			}
			let src = self.as_mut_ptr();
			src.wrapping_offset(range.start as isize).copy_to(src.wrapping_offset(offset as isize), dl)
		}
	}
}
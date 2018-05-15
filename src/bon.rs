// 二进制数据模块

// 小端-非网络字节序，和quic一致

// 用于通讯的类型需要压缩表示，充分利用第一个字节
// 0=null
// 1=true
// 2=false
// 3=浮点数0.0，4=浮点数1.0，5=16位浮点数，6=32位浮点数，7=64位浮点数，8=128位浮点数;
// 9~29= -1~19
// 30=8位正整数，31=16位正整数，32=32位正整数，33=48位正整数，34=64位正整数
// 35=8位负整数，36=16位负整数，37=32位负整数，38=48位负整数，39=64位负整数

// 40-104=0-64长度的二进制数据，
// 105=8位长度的二进制数据，106=16位长度的二进制数据，107=32位长度的二进制数据，108=48位长度的二进制数据，109=64位长度的二进制数据

// 110-174=0-64长度的UTF8字符串，
// 175=8位长度的UTF8字符串，176=16位长度的UTF8字符串，177=32位长度的UTF8字符串，178=48位长度的UTF8字符串，179=64位长度的UTF8字符串

// 180-244=0-64长度的容器，包括对象、数组和map、枚举
// 245=8位长度的容器，246=16位长度的容器，247=32位长度的容器，248=48位长度的容器，249=64位长度的容器
// 之后的一个4字节的整数表示类型。
// 类型：
// 	0 表示忽略
// 	1 通用对象
// 	2 通用数组
// 	3 通用map
	
// 如果是通用对象、数组、map，后面会有一个动态长度的整数，表示元素的数量。

// 容器，由于有总大小的描述，从而可以只对感兴趣的部分作反序列化
// TODO 定义一个全类型的枚举 enum BonType<T>， ReadNext WriteNext 的 T 应该为BonType。提供一个 read(&self) -> BonType<T>

use data_view::DataView;
use std::ops::{Range};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use atom::Atom;
use std::marker::Sized;

pub enum EnumValue {
	Void,
	Bool(bool),
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
	Str(String),
	Bin(Vec<u8>),
	Arr(Arc<Vec<EnumValue>>),
	Map(HashMap<Arc<EnumValue>, Arc<EnumValue>>),
	Struct(Arc<StructValue>),
}

pub struct StructValue {
	pub hash: u32,
	pub fields: Vec<FieldValue>
}

pub struct FieldValue {
	pub name: Atom,
	pub fvalue: EnumValue,
}

/**
 * @description 二进制数据缓存
 * @example
 */
pub struct BonBuffer {
	// u8数组
	bytes: Vec<u8>,
	// 头部指针
	head: usize,
	// 尾部指针
	tail:usize,
}

impl BonBuffer{

	pub fn with_bytes(buf: Vec<u8>, head:Option<usize>, tail: Option<usize>) -> BonBuffer {
		let h  = match head {
			Some(v) => {assert!(v <= buf.len(), "invalid head"); v},
			None => 0
		};

		let t  = match tail {
			Some(v) => {assert!(v > h, "invalid tail"); v},
			None => 0
		};
		BonBuffer{
			bytes: buf,
			head: h,
			tail: t,
		}
	}

	pub fn with_capacity(size: usize) -> BonBuffer {
		BonBuffer{
			bytes: Vec::with_capacity(size),
			head: 0,
			tail: 0,
		}
	}

	pub fn new() -> BonBuffer {
		BonBuffer{
			bytes: Vec::new(),
			head: 0,
			tail: 0,
		}
	}

	pub fn get_byte(&self) -> &Vec<u8> {
		&self.bytes
	}

	pub fn unwrap(self) -> Vec<u8> {
		self.bytes
	}

	pub fn clear(&mut self) {
		self.head = 0;
		self.tail = 0;
	}

	pub fn write_u8(&mut self, v: u8){
		self.write_unit32(v as u32);
	}

	pub fn write_u16(&mut self, v: u16){
		self.write_unit32(v as u32);
	}

	pub fn write_u32(&mut self, v: u32){
		self.write_unit32(v);
	}

	pub fn write_u64(&mut self, v: u64){
		self.write_unit64(v);
	}

	pub fn write_i8(&mut self, v: i8){
		self.write_int32(v as i32);
	}

	pub fn write_i16(&mut self, v: i16){
		self.write_int32(v as i32);
	}

	pub fn write_i32(&mut self, v: i32){
		self.write_int32(v);
	}

	pub fn write_i64(&mut self, v: i64){
		self.write_int64(v);
	}
	pub fn write_nil(&mut self) {
		self.bytes.set_bu8(0, self.tail);
		self.tail += 1;
	}

	pub fn write_bool(&mut self, v: bool) {
		self.bytes.set_lu8(match v{true => 1, false => 2}, self.tail);
		self.tail += 1;
	}

	pub fn write_f32(&mut self, v: f32) {
		if v == 0.0 {
			self.try_extend_capity(1);
			self.bytes.set_lu8(3, self.tail);
			self.tail += 1;
			return;
		}
		if v == 1.0 {
			self.try_extend_capity(1);
			self.bytes.set_lu8(4, self.tail);
			self.tail += 1;
			return;
		}
		self.try_extend_capity(5);
		self.bytes.set_lu8(6, self.tail);
		self.bytes.set_lf32( v, self.tail + 1);
		self.tail += 5;
	}

	pub fn write_f64(&mut self, v: f64) {
		if v == 0.0 {
			self.try_extend_capity(1);
			self.bytes.set_lu8(3, self.tail);
			self.tail += 1;
			return;
		}
		if v == 1.0 {
			self.try_extend_capity(1);
			self.bytes.set_lu8(4, self.tail);
			self.tail += 1;
			return;
		}
		self.try_extend_capity(9);
		self.bytes.set_lu8(7, self.tail);
		self.bytes.set_lf64(v, self.tail + 1);
		self.tail += 9;
	}

	pub fn write_pint(&mut self, v: u32) {
		if v > 0x20000000{
			//panic!("invalid pint:" + v);
		}if v < 0x80 {
			self.try_extend_capity(1);
			self.bytes.set_lu8(v as u8, self.tail);
			self.tail += 1;
		}else if v < 0x4000 {
			self.try_extend_capity(2);
			self.bytes.set_lu16((0x8000 + v) as u16, self.tail);
			self.tail += 2;
		}else{
			self.try_extend_capity(4);
			self.bytes.set_lu32( (0xC0000000 + v) as u32, self.tail);
			self.tail += 4;
		}
	}

	pub fn write_utf8(&mut self, s: &str) {
		self.write_data(s.as_bytes(), 110);
	}

	pub fn write_bin(&mut self, arr: &[u8], range: Range<usize>) {
		self.write_data(&arr[range], 40)
	}

	// 写二进制数据
	pub fn write_data(&mut self, arr: &[u8], t: u8) {
		let length = arr.len();
		if length <= 64 {
			self.try_extend_capity(1 + length);
			// 长度小于等于64， 本字节直接表达
			self.bytes.set_lu8( t + length as u8, self.tail);
			self.tail += 1;
		} else if length <= 0xff {
			self.try_extend_capity(2 + length);
			// 长度小于256， 用下一个1字节记录
			self.bytes.set_lu8( t + 65, self.tail);
			self.bytes.set_lu8( length as u8, self.tail + 1);
			self.tail += 2;
		} else if length <= 0xffff {
			self.try_extend_capity(3 + length);
			self.bytes.set_lu8( t + 66, self.tail);
			self.bytes.set_lu16( length as u16, self.tail + 1);
			self.tail += 3;
		} else if length <= 0xffffffff {
			self.try_extend_capity(5 + length);
			self.bytes.set_lu8( t + 67, self.tail);
			self.bytes.set_lu32(  length as u32, self.tail + 1);
			self.tail += 5;
		} else if length as u64 <= 0xffffffffffff {
			self.try_extend_capity(7 + length);
			self.bytes.set_lu8( t + 68, self.tail);
			self.bytes.set_lu16((length & 0xffff) as u16, self.tail + 1);
			self.bytes.set_lu32( (length >> 16) as u32, self.tail + 3);
			self.tail += 7;
		} else {
			self.try_extend_capity(9 + length);
			self.bytes.set_lu8( t + 69, self.tail);
			self.bytes.set_lu64(t as u64, self.tail + 1);
			self.tail += 9;
		}
		//let a = self.bytes.capacity();
		self.bytes.set(arr, self.tail);
		self.tail += length;
		//let arrlen = arr.len();
		//let byteslen = self.bytes.len();
		//let a = 0;
	}

	//容器有数组，map，枚举，struct
	pub fn write_container<T, F>(&mut self, o: &T, write_next: F, estimated_size: Option<usize>) where F: Fn(&mut BonBuffer, &T) {
		let mut t = self.bytes.len();
		let len_bytes: usize;//描述容器长度的值的字节数
		let capacity = self.bytes.capacity();
		// 根据预估大小，预留出足够的空间来写入容器的总大小
		let estimated_size = match estimated_size{Some(v) => v, None => 0xffff};
		let mut limit_size: u64;
		
		if estimated_size <= 64 {
			self.try_extend_capity(5 + estimated_size);
			len_bytes = 0;
			limit_size = 64;
		} else if estimated_size <= 0xff {
			self.try_extend_capity(6 + estimated_size);
			len_bytes = 1;
			limit_size = 0xff;
		} else if estimated_size <= 0xffff {
			self.try_extend_capity(8 + estimated_size);
			len_bytes = 3;
			limit_size = 0xffff;
		} else if estimated_size <= 0xffffffff {
			self.try_extend_capity(10 + estimated_size);
			len_bytes = 5;
			limit_size = 0xffffffff;
		} else if estimated_size as u64 <= 0xffffffffffff {
			self.try_extend_capity(12 + estimated_size);
			len_bytes = 7;
			limit_size = 0xffffffffffff;
		} else {
			self.try_extend_capity(14 + estimated_size);
			len_bytes = 9;
			limit_size = 0xffffffffffffffff;
		}
		t = t + 5 + len_bytes;//容器长度字节数的分类为1字节， 容器类型为4字节, 容器长度字节数位len_bytes
		write_next(self, o);
		let len = (self.bytes.len() - t) as u64;
		// 判断实际写入的大小超出预期的大小，需要移动数据
		if limit_size < len && len > 64{
			let mut len_bytes1: usize = 0;
			if limit_size <= 64 && len <= 0xff {
				len_bytes1 = 1;
				limit_size = 0xff;
			} else if len <= 0xffff {
				len_bytes1 = 3;
				limit_size = 0xffff;
			} else if len <= 0xffffffff {
				len_bytes1 = 5;
				limit_size = 0xffffffff;
			} else if len <= 0xffffffffffff as u64 {
				len_bytes1 = 7;
				limit_size = 0xffffffffffff;
			} else if len <= 0xfffffffffffffffe + 1{
				len_bytes1 = 9;
				limit_size = 0xffffffffffffffff;
			}

			let offset = len_bytes1 - len_bytes;
			let l = self.bytes.len();
			self.try_extend_capity(l + offset - capacity);
			self.bytes.move_part(t..l, t + offset);
			self.tail += offset;
		}
		// 根据实际的限制大小，写入实际长度
		match limit_size {
			64 => {
				self.bytes.set_lu8((180 + len) as u8, t);
			},
			0xff =>{
				self.bytes.set_lu8(245, t);
				self.bytes.set_lu8(len as u8, t + 1);
			},
			0xffff =>{
				self.bytes.set_lu8( 246, t);
				self.bytes.set_lu16(len as u16, t + 1);
			},
			0xffffffff => {
				self.bytes.set_lu8(247, t);
				self.bytes.set_lu32(len as u32, t + 1);
			},
			0xffffffffffff => {
				self.bytes.set_lu8(248, t);
				self.bytes.set_lu16((len & 0xffff) as u16, t + 1);
				self.bytes.set_lu32((len >> 16) as u32, t + 3);
			},

			_ => {
				self.bytes.set_lu8(249, t);
				self.bytes.set_lu64(len as u64, t + 1);
			},
		}
	}

	pub fn get_type(&mut self) -> u8 {
		self.bytes.get_lu8(self.head)
	}

	pub fn read_bool(&mut self) -> bool {
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		match t {
			1 => true,
			2 => false,
			_ => {panic!("You want to read a bool, in fact, it's {}", t);}
		}
	}

	pub fn read_u8(&mut self) -> u8 {
		self.read_integer::<u32>() as u8
	}

	pub fn read_u16(&mut self) -> u16 {
		self.read_integer::<u32>() as u16
	}

	pub fn read_u32(&mut self) -> u32 {
		self.read_integer::<u32>()
	}

	pub fn read_u64(&mut self) -> u64 {
		self.read_integer::<u64>()
	}

	pub fn read_i8(&mut self) -> i8 {
		self.read_integer::<i32>() as i8
	}

	pub fn read_i16(&mut self) -> i16 {
		self.read_integer::<i32>() as i16
	}

	pub fn read_i32(&mut self) -> i32 {
		self.read_integer::<i32>()
	}

	pub fn read_i64(&mut self) -> i64 {
		self.read_integer::<i64>()
	}

	pub fn read_f32(&mut self) -> f32 {
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		match t {
			3 => {0.0},
			4 => {1.0},
			6 => {
				self.head += 4;
				self.bytes.get_lf32(self.head - 4) as f32
			},
			_ => {
				panic!("You want to read a f32, in fact, it's {}", t);
			}
		}
	}

	pub fn read_f64(&mut self) -> f64 {
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		match t {
			3 => {0.0},
			4 => {1.0},
			6 => {
				self.head += 4;
				self.bytes.get_lf32(self.head - 4) as f64
			},
			7 => {
				self.head += 8;
				self.bytes.get_lf64(self.head - 8)
			},
			_ => {
				panic!("You want to read a f64, in fact, it's {}", t);
			}
		}
	}

	pub fn read_bin(&mut self) -> Vec<u8> {
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		let len: usize;
		if t >= 40 && t <= 104{
			len = (t as usize) - 40;
			self.head += len;
		}else {
			match t {
				105 => {
					len = self.bytes.get_lu8(self.head) as usize as usize;
					self.head += len + 1;
				},
				106 => {
					len = self.bytes.get_lu16(self.head) as usize;
					self.head += len + 2;
				},
				107 => {
					len = self.bytes.get_lu32(self.head) as usize;
					self.head += len + 4;
				},
				108 => {
					len = self.bytes.get_lu16(self.head) as usize + (self.bytes.get_lu32(self.head + 2) * 0x10000) as usize;
					self.head += len + 6;
				},
				109 => {
					len = self.bytes.get_lu64(self.head) as usize;
					self.head += len + 8;
				},
				_ => {
					panic!("You want to read a &[u8], in fact, it's {}", t);
				}
			};
		}

		let mut dst = Vec::with_capacity(len);
		unsafe{ dst.set_len(len); }
		(&mut dst).clone_from_slice(&self.bytes[self.head - len..self.head]);
		dst
	}

	pub fn read_utf8(&mut self) -> String {
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		let len: usize;
		if t >= 101 && t <= 174{
			len = t as usize - 110;
			self.head += len;
		}else{
			match t {
				175 => {
					len = self.bytes.get_lu8(self.head) as usize as usize;
					self.head += len + 1;
				},
				176 => {
					len = self.bytes.get_lu16(self.head) as usize;
					self.head += len + 2;
				},
				177 => {
					len = self.bytes.get_lu32(self.head) as usize;
					self.head += len + 4;
				},
				178 => {
					len = self.bytes.get_lu16(self.head) as usize + (self.bytes.get_lu32(self.head + 2) * 0x10000) as usize;
					self.head += len + 6;
				},
				179 => {
					len = self.bytes.get_lu64(self.head) as usize;
					self.head += len + 8;
				}
				_ => {
					panic!("You want to read a string, in fact, it's {}", t);
				}
			}
		}

		let mut dst = Vec::with_capacity(len);
		unsafe{ dst.set_len(len); }
		(&mut dst).clone_from_slice(&self.bytes[self.head - len..self.head]);
		String::from_utf8(dst).expect("u8array transformation string exception")
	}

	pub fn read_container<T, F>(&mut self, read_next: F) -> T where F: FnOnce(&mut BonBuffer, &u32) -> T{
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		let len: usize;
		if t >= 180 && t <= 244{
			len = t as usize - 180;
			self.head += len;
		}else{
			match t {
				245 => {
					//len = self.bytes.get_lu8(self.head) as usize;
					self.head += 5;
				},
				246 => {
					//len = self.bytes.get_lu16(self.head) as usize;
					self.head += 6;
				},
				247 => {
					//len = self.bytes.get_lu32(self.head) as usize;
					self.head += 8;
				},
				248 => {
					//len = self.bytes.get_lu16(self.head) as usize + (self.bytes.get_lu32(self.head + 2) * 0x10000) as usize;
					self.head += 10;
				},
				249 => {
					//len = self.bytes.get_lu64(self.head) as usize;
					self.head += 12;
				},
				_ => {
					panic!("You want to read a container, in fact, it's {}", t);
				}
			}
		}
		let tt = &self.bytes.get_lu32(self.head - 4);
		read_next(self, tt)
	}

	pub fn is_nil(&mut self) -> bool{
		let first = self.bytes.get_lu8(self.head);
		if first == 0{
			self.head += 1;
			true
		}else{
			false
		}
	}

	pub fn read(&mut self) -> EnumValue{
		let first = self.bytes.get_lu8(self.head);
		self.head += 1;
		match first{
			0 => {EnumValue::Void},
			1 => {EnumValue::Bool(true)},
			2 => {EnumValue::Bool(false)},
			3 => {EnumValue::F32(0.0)},
			4 => {EnumValue::F32(1.0)},
			5 => {panic!("16 bit floating-point number temporarily unsupported");},
			6 => {
				self.head += 4;
				EnumValue::F32(self.bytes.get_lf32(self.head - 4))
			},
			7 => {
				self.head += 8;
				EnumValue::F64(self.bytes.get_lf64(self.head - 8))
			},
			8 => {panic!("128 bit floating-point number temporarily unsupported");},
			9 => {EnumValue::I8(-1)},
			10..30 => {EnumValue::U8(first - 10)},
			30 => {
				self.head += 1;
				EnumValue::U8(self.bytes.get_lu8(self.head - 1))
			},
			31 => {
				self.head += 2;
				EnumValue::U16(self.bytes.get_lu16(self.head - 2))
			},
			32 => {
				self.head += 4;
				EnumValue::U32(self.bytes.get_lu32(self.head - 4))
			},
			33 => {
				self.head += 6;
				EnumValue::U64(self.bytes.get_lu16(self.head - 6) as u64 + ((self.bytes.get_lu32(self.head - 4) as u64) << 16))
			},
			34 => {
				self.head += 8;
				EnumValue::U64(self.bytes.get_lu64(self.head - 8) as u64)
			},
			35 => {
				self.head += 1;
				EnumValue::I16(-(self.bytes.get_lu8(self.head - 1) as i16))
			},
			36 => {
				self.head += 2;
				EnumValue::I32(-(self.bytes.get_lu16(self.head - 2) as i32))
			},
			37 => {
				self.head += 4;
				EnumValue::I64(-(self.bytes.get_lu32(self.head - 4) as i64))
			},
			38 => {
				self.head += 6;
				EnumValue::I64(-(self.bytes.get_lu16(self.head - 6) as i64) - ((self.bytes.get_lu32(self.head - 4) as i64) << 16))
			}
			39 => {
				self.head += 8;
				EnumValue::I64(-(self.bytes.get_lu64(self.head - 4) as i64))//类型39，能表达i65，但此处限制最大i64，溢出会损失精度
			},
			40..110 => {
				EnumValue::Bin(self.read_bin())
			},
			110..180 => {
				EnumValue::Str(self.read_utf8())
			}
			_ => {
				panic!("待实现");
			}
		}
	}

	// 0=null
// 1=true
// 2=false
// 3=浮点数0.0，4=浮点数1.0，5=16位浮点数，6=32位浮点数，7=64位浮点数，8=128位浮点数;
// 9~29= -1~19
// 30=8位正整数，31=16位正整数，32=32位正整数，33=48位正整数，34=64位正整数
// 35=8位负整数，36=16位负整数，37=32位负整数，38=48位负整数，39=64位负整数

// 40-104=0-64长度的二进制数据，
// 105=8位长度的二进制数据，106=16位长度的二进制数据，107=32位长度的二进制数据，108=48位长度的二进制数据，109=64位长度的二进制数据

// 110-174=0-64长度的UTF8字符串，
// 175=8位长度的UTF8字符串，176=16位长度的UTF8字符串，177=32位长度的UTF8字符串，178=48位长度的UTF8字符串，179=64位长度的UTF8字符串

// 180-244=0-64长度的容器，包括对象、数组和map、枚举
// 245=8位长度的容器，246=16位长度的容器，247=32位长度的容器，248=48位长度的容器，249=64位长度的容器
// 之后的一个4字节的整数表示类型。



	fn read_integer<T: AsFrom<u32> + AsFrom<u64> + AsFrom<i32> + AsFrom<i64>>(&mut self) -> T {
		let t = self.bytes.get_lu8(self.head);
		self.head += 1;
		if t >= 9 && t <= 29{
			T::from((t -10) as u32)
		}else{
			match t {
				30 => {
					self.head += 1;
					T::from(self.bytes.get_lu8(self.head - 1) as u32)
				},
				31 => {
					self.head += 2;
					T::from(self.bytes.get_lu16(self.head - 2) as u32)
				},
				32 => {
					self.head += 4;
					T::from(self.bytes.get_lu32(self.head - 4))
				},
				33 => {
					self.head += 6;
					
					T::from(self.bytes.get_lu16(self.head - 6) as u64 + ((self.bytes.get_lu32(self.head - 4) as u64)  << 16)  )
				},
				34 => {
					self.head += 8;
					T::from(self.bytes.get_lu64(self.head - 8) as u64)
				},
				35 => {
					self.head += 1;
					T::from(-(self.bytes.get_lu8(self.head - 1) as i32))
				},
				36 => {
					self.head += 2;
					T::from(-(self.bytes.get_lu16(self.head - 2) as i32))
				},
				37 => {
					self.head += 4;
					T::from(-(self.bytes.get_lu32(self.head - 4) as i64))
				},
				38 => {
					self.head += 6;
					T::from(-(self.bytes.get_lu16(self.head - 6) as i64) - ((self.bytes.get_lu32(self.head - 4) as i64) << 16))
				}
				39 => {
					self.head += 8;
					T::from(-(self.bytes.get_lu64(self.head - 4) as i64))//类型39，能表达i65，但此处限制最大i64，溢出会损失精度
				},
				_ => {
					panic!("You want to read a integer, in fact, it's {}", t);
				}
			}			
		}
	}	

	fn extend_capity(&mut self, len: usize) {
		let old_capacity = self.bytes.capacity();
		if old_capacity > 4194304 {//4M
			self.bytes.reserve_exact(len * 2);//准确扩容
		}else{
			self.bytes.reserve(len);//使用vec内部规则扩容（扩大为原有大小的两倍）
		}
		
	}

	fn try_extend_capity(&mut self, len: usize){
		if self.bytes.len() + len > self.bytes.capacity(){
			self.extend_capity(len);
		}
	}

	fn write_int32(&mut self, v: i32) {
		if v >= -1 && v < 20 {
			self.try_extend_capity(1);
			self.bytes.set_lu8((v + 10) as u8, self.tail);
			self.tail += 1;
			return;
		}
		let mut i: u8 = 0;
		let mut v1 = v;
		if v < 0 {
			v1 = -v;
			i = 5;
		}
		if v1 <= 0xFF {
			self.try_extend_capity(2);
			self.bytes.set_lu8(30 + i, self.tail);
			self.bytes.set_lu8(v1 as u8, self.tail + 1);
			self.tail += 2;
		} else if v1 <= 0xFFFF {
			self.try_extend_capity(3);
			self.bytes.set_lu8(31 + i, self.tail);
			self.bytes.set_lu16(v1 as u16, self.tail + 1);
			self.tail += 3;
		} else {
			self.try_extend_capity(5);
			self.bytes.set_lu8(32 + i, self.tail);
			self.bytes.set_lu32( v1 as u32, self.tail + 1);
			self.tail += 5;
		}
	}

	fn write_int64(&mut self, v: i64) {
		if v >= -1 && v < 20 {
			self.try_extend_capity(1);
			self.bytes.set_lu8((v + 10) as u8, self.tail);
			self.tail += 1;
			return;
		}
		let mut i: u8 = 0;
		let mut v1 = v;
		if v1 < 0 {
			v1 = -v;
			i = 5;
		}
		if v1 <= 0xFF {
			self.try_extend_capity(2);
			self.bytes.set_lu8(30 + i, self.tail);
			self.bytes.set_lu8(v1 as u8, self.tail + 1);
			self.tail += 2;
		} else if v1 <= 0xFFFF {
			self.try_extend_capity(3);
			self.bytes.set_lu8(31 + i,self.tail);
			self.bytes.set_lu16(v1 as u16, self.tail + 1);
			self.tail += 3;
		} else if v1 <= 0xFFFFFFFF {
			self.try_extend_capity(5);
			self.bytes.set_lu8(32 + i, self.tail);
			self.bytes.set_lu32( v1 as u32, self.tail + 1);
			self.tail += 5;
		} else if v1 <= 0xFFFFFFFFFFFF {
			self.try_extend_capity(7);
			self.bytes.set_lu8(32 + i, self.tail);
			self.bytes.set_lu16((v1 & 0xffff) as u16, self.tail + 1);
			self.bytes.set_lu32( (v1 >> 16) as u32, self.tail + 3);
			self.tail += 7;
		} else {
			self.try_extend_capity(9);
			self.bytes.set_lu8(33 + i, self.tail);
			self.bytes.set_lu64(v1 as u64, self.tail + 1);
			self.tail += 9;
		}
	}

	fn write_unit32(&mut self, v: u32) {
		if v < 20 {
			self.try_extend_capity(1);
			self.bytes.set_lu8((v + 10) as u8, self.tail);
			self.tail += 1;
		}else if v <= 0xFF {
			self.try_extend_capity(2);
			self.bytes.set_lu8(30 as u8, self.tail);
			self.bytes.set_lu8(v as u8, self.tail + 1);
			self.tail += 2;
		} else if v <= 0xFFFF {
			self.try_extend_capity(3);
			self.bytes.set_lu8(31 as u8, self.tail);
			self.bytes.set_lu16(v as u16, self.tail + 1);
			self.tail += 3;
		} else {
			self.try_extend_capity(5);
			self.bytes.set_lu8(32 as u8, self.tail);
			self.bytes.set_lu32( v as u32, self.tail + 1);
			self.tail += 5;
		}
	}

	fn write_unit64(&mut self, v: u64) {
		if v < 20 {
			self.try_extend_capity(1);
			self.bytes.set_lu8((v + 10) as u8, self.tail);
			self.tail += 1;
		}else if v <= 0xFF {
			self.try_extend_capity(2);
			self.bytes.set_lu8(30 as u8, self.tail);
			self.bytes.set_lu8(v as u8, self.tail + 1);
			self.tail += 2;
		} else if v <= 0xFFFF {
			self.try_extend_capity(3);
			self.bytes.set_lu8(31 as u8, self.tail);
			self.bytes.set_lu16(v as u16, self.tail + 1);
			self.tail += 3;
		} else if v <= 0xFFFFFFFF {
			self.try_extend_capity(5);
			self.bytes.set_lu8(32 as u8, self.tail);
			self.bytes.set_lu32( v as u32, self.tail + 1);
			self.tail += 5;
		} else if v <= 0xFFFFFFFFFFFF {
			self.try_extend_capity(7);
			self.bytes.set_lu8(33 as u8, self.tail);
			self.bytes.set_lu16((v & 0xffff) as u16, self.tail + 1);
			self.bytes.set_lu32( (v >> 16) as u32, self.tail + 3);
			self.tail += 7;
		} else {
			self.try_extend_capity(9);
			self.bytes.set_lu8(34 as u8, self.tail);
			self.bytes.set_lu64(v as u64, self.tail + 1);
			self.tail += 9;
		}
	}
}

trait AsFrom<T> {
	fn from(T) -> Self;
}


impl AsFrom<u32> for u32{
	fn from(t: u32) -> u32 {
		t
	}
}
impl AsFrom<u64> for u32{
	fn from(t: u64) -> u32 {
		t as u32
	}
}

impl AsFrom<i32> for u32{
	fn from(t: i32) -> u32 {
		t as u32
	}
}

impl AsFrom<i64> for u32{
	fn from(t: i64) -> u32 {
		t as u32
	}
}

impl AsFrom<u64> for u64{
	fn from(t: u64) -> u64 {
		t
	}
}
impl AsFrom<u32> for u64{
	fn from(t: u32) -> u64 {
		t as u64
	}
}

impl AsFrom<i32> for u64{
	fn from(t: i32) -> u64 {
		t as u64
	}
}

impl AsFrom<i64> for u64{
	fn from(t: i64) -> u64 {
		t as u64
	}
}

impl AsFrom<u64> for i32{
	fn from(t: u64) -> i32 {
		t as i32
	}
}
impl AsFrom<u32> for i32{
	fn from(t: u32) -> i32 {
		t as i32
	}
}

impl AsFrom<i32> for i32{
	fn from(t: i32) -> i32 {
		t
	}
}

impl AsFrom<i64> for i32{
	fn from(t: i64) -> i32 {
		t as i32
	}
}

impl AsFrom<u64> for i64{
	fn from(t: u64) -> i64 {
		t as i64
	}
}
impl AsFrom<u32> for i64{
	fn from(t: u32) -> i64 {
		t as i64
	}
}

impl AsFrom<i32> for i64{
	fn from(t: i32) -> i64 {
		t as i64
	}
}

impl AsFrom<i64> for i64{
	fn from(t: i64) -> i64 {
		t
	}
}

pub trait Encode: Sized{
	fn encode(&self, bb: &mut BonBuffer);
}

pub trait Decode: Sized{
	fn decode(bb: &mut BonBuffer) -> Self;
}

impl Encode for u8{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_u8(self.clone());
	}
}

impl Decode for u8{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_u8()
	}
}

impl Encode for u16{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_u16(self.clone());
	}
}

impl Decode for u16{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_u16()
	}
}

impl Encode for u32{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_u32(self.clone());
	}
}

impl Decode for u32{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_u32()
	}
}

impl Encode for u64{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_u64(self.clone());
	}
}

impl Decode for u64{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_u64()
	}
}

impl Encode for i8{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_i8(self.clone());
	}
}

impl Decode for i8{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_i8()
	}
}

impl Encode for i16{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_i16(self.clone());
	}
}

impl Decode for i16{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_i16()
	}
}

impl Encode for i32{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_i32(self.clone())
	}
}

impl Decode for i32{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_i32()
	}
}

impl Encode for i64{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_i64(self.clone());
	}
}

impl Decode for i64{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_i64()
	}
}

impl Encode for bool{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_bool(self.clone());
	}
}

impl Decode for bool{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_bool()
	}
}

impl Encode for usize{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_u64(self.clone() as u64);
	}
}

impl Decode for usize{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_u64() as usize
	}
}

impl Encode for isize{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_i64(self.clone() as i64);
	}
}

impl Decode for isize{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_i64() as isize
	}
}

impl Encode for String{
	fn encode(&self, bb: &mut BonBuffer){
		bb.write_utf8(self);
	}
}

impl Decode for String{
	fn decode(bb: &mut BonBuffer) -> Self{
		bb.read_utf8()
	}
}

impl<K: Encode + Eq + Hash, V: Encode> Encode for HashMap<K, V>{
	fn encode(&self, bb: &mut BonBuffer){
		//self.typeid().encode(bb);
		self.len().encode(bb);
		for (k, v) in self.iter(){
			k.encode(bb);
			v.encode(bb);
		}
	}
}

impl<K: Decode + Eq + Hash, V: Decode> Decode for HashMap<K, V>{
	fn decode(bb: &mut BonBuffer) -> Self{
		let mut map = HashMap::new();
		let count = usize::decode(bb);
		for _ in 0..count{
			map.insert(K::decode(bb), V::decode(bb));
		}
		map
	}
}

impl<T: Encode> Encode for Vec<T>{
	fn encode(&self, bb: &mut BonBuffer){
		self.len().encode(bb);
		for v in self.iter(){
			v.encode(bb);
		}
	}
}

impl<T: Decode> Decode for Vec<T> {
	fn decode(bb: &mut BonBuffer) -> Vec<T>{
		let count = usize::decode(bb);
		let mut vec = Vec::new();
		for _ in 0..count{
			vec.push(T::decode(bb));
		}
		vec
	}
}

impl<T: Encode> Encode for Option<T>{
	fn encode(&self, bb: &mut BonBuffer){
		match self{
			&Some(ref v) => {v.encode(bb);}
			&None => {bb.write_nil();}
		}
	}
}

impl<T: Decode> Decode for Option<T> {
	fn decode(bb: &mut BonBuffer) -> Option<T>{
		match bb.is_nil(){
			true => None,
			false => Some(T::decode(bb)),
		}
	}
}

#[test]
fn test_u8() {
    let mut buf = BonBuffer::new();
    buf.write_u8(5);
    buf.write_u8(50);
    assert_eq!(buf.read_u8(), 5);
    assert_eq!(buf.read_u8(), 50);
}

#[test]
fn test_u16() {
    let mut buf = BonBuffer::new();
    buf.write_u16(18);
	buf.write_u16(50);
    buf.write_u16(65534);
    assert_eq!(buf.read_u16(), 18);
    assert_eq!(buf.read_u16(), 50);
	assert_eq!(buf.read_u16(), 65534);
}

#[test]
fn test_u32() {
    let mut buf = BonBuffer::new();
    buf.write_u32(18);
	buf.write_u32(50);
    buf.write_u32(65534);
	buf.write_u32(4294967293);
    assert_eq!(buf.read_u32(), 18);
    assert_eq!(buf.read_u32(), 50);
	assert_eq!(buf.read_u32(), 65534);
	assert_eq!(buf.read_u32(), 4294967293);
}

#[test]
fn test_u64() {
    let mut buf = BonBuffer::new();
    buf.write_u64(18);
	buf.write_u64(50);
    buf.write_u64(65534);
	buf.write_u64(4294967293);
	//buf.write_u64(18446744073709551990);
    assert_eq!(buf.read_u64(), 18);
    assert_eq!(buf.read_u64(), 50);
	assert_eq!(buf.read_u64(), 65534);
	assert_eq!(buf.read_u64(), 4294967293);
	//assert_eq!(buf.read_u64(), 18446744073709551990);
}

#[test]
fn test_i8() {
    let mut buf = BonBuffer::new();
    buf.write_i8(15);
	buf.write_i8(-11);
	buf.write_u64(50);
    assert_eq!(buf.read_i8(), 15);
    assert_eq!(buf.read_i8(), -11);
	assert_eq!(buf.read_i8(), 50);
}

#[test]
fn test_i16() {
    let mut buf = BonBuffer::new();
    buf.write_i16(15);
	buf.write_i16(-11);
	buf.write_i16(50);
	buf.write_i16(32766);
	buf.write_i16(-32765);
    assert_eq!(buf.read_i16(), 15);
    assert_eq!(buf.read_i16(), -11);
	assert_eq!(buf.read_i16(), 50);
	assert_eq!(buf.read_i16(), 32766);
	assert_eq!(buf.read_i16(), -32765);
}

#[test]
fn test_i32() {
    let mut buf = BonBuffer::new();
    buf.write_i32(15);
	buf.write_i32(-11);
	buf.write_i32(50);
	buf.write_i32(32766);
	buf.write_i32(-32765);
	buf.write_i32(2147483645);
	buf.write_i32(-2147483643);
    assert_eq!(buf.read_i32(), 15);
    assert_eq!(buf.read_i32(), -11);
	assert_eq!(buf.read_i32(), 50);
	assert_eq!(buf.read_i32(), 32766);
	assert_eq!(buf.read_i32(), -32765);
	assert_eq!(buf.read_i32(), 2147483645);
	assert_eq!(buf.read_i32(), -2147483643);
}

#[test]
fn test_i64() {
    let mut buf = BonBuffer::new();
    buf.write_i64(15);
	buf.write_i64(-11);
	buf.write_i64(50);
	buf.write_i64(32766);
	buf.write_i64(-32765);
	buf.write_i64(2147483645);
	buf.write_i64(-2147483643);
	buf.write_i64(2147483652);
	buf.write_i64(-2147483653);
    assert_eq!(buf.read_i64(), 15);
    assert_eq!(buf.read_i64(), -11);
	assert_eq!(buf.read_i64(), 50);
	assert_eq!(buf.read_i64(), 32766);
	assert_eq!(buf.read_i64(), -32765);
	assert_eq!(buf.read_i64(), 2147483645);
	assert_eq!(buf.read_i64(), -2147483643);
	assert_eq!(buf.read_i64(), 2147483652);
	assert_eq!(buf.read_i64(), -2147483653);
}

#[test]
fn test_f32() {
    let mut buf = BonBuffer::new();
    buf.write_f32(1.0);
	buf.write_f32(0.0);
	buf.write_f32(5.0);
	buf.write_f32(-6.0);
    assert_eq!(buf.read_f32(), 1.0);
    assert_eq!(buf.read_f32(), 0.0);
	assert_eq!(buf.read_f32(), 5.0);
	assert_eq!(buf.read_f32(), -6.0);
}

#[test]
fn test_f64() {
    let mut buf = BonBuffer::new();
    buf.write_f64(1.0);
	buf.write_f64(0.0);
	buf.write_f64(5.0);
	buf.write_f64(-6.0);
    assert_eq!(buf.read_f64(), 1.0f64);
    assert_eq!(buf.read_f64(), 0.0f64);
	assert_eq!(buf.read_f64(), 5.0f64);
	assert_eq!(buf.read_f64(), -6.0f64);
}

#[test]
fn test_utf8() {
    let mut buf = BonBuffer::new();
    buf.write_utf8("123byufgeruy");
    assert_eq!(buf.read_utf8(), "123byufgeruy");
}

#[test]
fn test_bin() {
    let mut buf = BonBuffer::new();
	let arr = [5; 10];
    buf.write_bin(&arr,0..10);
    assert_eq!(buf.read_bin(), arr);
}


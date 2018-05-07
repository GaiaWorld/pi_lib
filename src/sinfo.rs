/**
 * 结构体信息
 */

use std::vec::Vec;
use std::collections::HashMap;
use std::sync::Arc;

use atom::Atom;
use bon::{BonBuffer, Encode, Decode};

// 枚举结构体字段的所有类型
pub enum EnumType {
	Bool,
	U8,
	U16,
	U32,
	U64,
	U128,
	U256,
	Usize,
	I8,
	I16,
	I32,
	I64,
	I128,
	I256,
	Isize,
	F32,
	F64,
	BigI,
	Str,
	Bin,
	UTC,
	Arr(Arc<EnumType>),
	Map(Arc<EnumType>, Arc<EnumType>),
	Struct(Arc<StructInfo>),
}

impl Encode for EnumType{
	fn encode(&self, bb:&mut BonBuffer){
		match self{
			&EnumType::Bool => {1.encode(bb);},
			&EnumType::U8 => {2.encode(bb);},
			&EnumType::U16 => {3.encode(bb);},
			&EnumType::U32 => {4.encode(bb);},
			&EnumType::U64 => {5.encode(bb);},
			&EnumType::U128 => {6.encode(bb);},
			&EnumType::U256 => {7.encode(bb);},
			&EnumType::Usize => {8.encode(bb);},
			&EnumType::I8 => {9.encode(bb);},
			&EnumType::I16 => {10.encode(bb);},
			&EnumType::I32 => {11.encode(bb);},
			&EnumType::I64 => {12.encode(bb);},
			&EnumType::I128 => {13.encode(bb);},
			&EnumType::I256 => {14.encode(bb);},
			&EnumType::Isize => {15.encode(bb);},
			&EnumType::F32 => {16.encode(bb);},
			&EnumType::F64 => {17.encode(bb);},
			&EnumType::BigI => {18.encode(bb);},
			&EnumType::Str => {19.encode(bb);},
			&EnumType::Bin => {20.encode(bb);},
			&EnumType::UTC => {21.encode(bb);},
			&EnumType::Arr(ref v) => {22.encode(bb); v.encode(bb);},
			&EnumType::Map(ref k, ref v) => {23.encode(bb); k.encode(bb); v.encode(bb);},
			&EnumType::Struct(ref v) => {24.encode(bb); v.encode(bb);},
		};
	}
}

impl Decode for EnumType{
	fn decode(bb:&mut BonBuffer) -> EnumType{
		let t = u8::decode(bb);
		match t{
			1 => {EnumType::Bool},
			2 => {EnumType::U8},
			3 => {EnumType::U16},
			4 => {EnumType::U32},
			5 => {EnumType::U64},
			6 => {EnumType::U128},
			7 => {EnumType::U256},
			8 => {EnumType::Usize},
			9 => {EnumType::I8},
			10 => {EnumType::I16},
			11 => {EnumType::I32},
			12 => {EnumType::I64},
			13 => {EnumType::I128},
			14 => {EnumType::I256},
			15 => {EnumType::Isize},
			16 => {EnumType::F32},
			17 => {EnumType::F64},
			18 => {EnumType::BigI},
			19 => {EnumType::Str},
			20 => {EnumType::Bin},
			21 => {EnumType::UTC},
			22 => {EnumType::Arr(Arc::new(EnumType::decode(bb)))},
			23 => {EnumType::Map(Arc::new(EnumType::decode(bb)), Arc::new(EnumType::decode(bb)))},
			24 => {EnumType::Struct(Arc::new(StructInfo::decode(bb)))},
			_ => {panic!("EnumType is not exist:{}", t);}
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

pub struct StructInfo {
	pub name: Atom,
	pub name_hash: u32,
	pub annotates: Option<HashMap<Atom, Atom>>,
	pub fields: Vec<FieldInfo>,
}

impl StructInfo {
	pub fn new(name:Atom, name_hash:u32) -> Self {
		StructInfo {
			name:name,
			name_hash: name_hash,
			annotates: None,
			fields: Vec::new(),
		}
	}
}

impl Encode for StructInfo{
	fn encode(&self, bb: &mut BonBuffer){
		self.name.encode(bb);
		self.name_hash.encode(bb);
	}
}

impl Decode for StructInfo{
	fn decode(bb: &mut BonBuffer) -> StructInfo{
		StructInfo{
			name: Atom::decode(bb),
			name_hash: u32::decode(bb),
			annotates: Option::decode(bb),
			fields: Vec::decode(bb),
		}
	}
}

pub struct FieldInfo {
	pub name: Atom,
	pub ftype: EnumType,
	pub annotates: Option<HashMap<Atom, Atom>>,
}

impl Encode for FieldInfo{
	fn encode(&self, bb: &mut BonBuffer){
		self.name.encode(bb);
		self.ftype.encode(bb);
		self.annotates.encode(bb);
	}
}

impl Decode for FieldInfo{
	fn decode(bb: &mut BonBuffer) -> FieldInfo{
		FieldInfo{
			name: Atom::decode(bb),
			ftype: EnumType::decode(bb),
			annotates: Option::decode(bb),
		}
	}
}
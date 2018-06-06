/**
 * 结构体信息
 */

use std::vec::Vec;
use std::collections::HashMap;
use std::sync::Arc;

use atom::Atom;
use bon::{WriteBuffer, ReadBuffer, Encode, Decode};

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
	fn encode(&self, bb:&mut WriteBuffer){
		match self{
			&EnumType::Bool => {0.encode(bb);},
			&EnumType::U8 => {1.encode(bb);},
			&EnumType::U16 => {2.encode(bb);},
			&EnumType::U32 => {3.encode(bb);},
			&EnumType::U64 => {4.encode(bb);},
			&EnumType::U128 => {5.encode(bb);},
			&EnumType::U256 => {6.encode(bb);},
			&EnumType::Usize => {7.encode(bb);},
			&EnumType::I8 => {8.encode(bb);},
			&EnumType::I16 => {9.encode(bb);},
			&EnumType::I32 => {10.encode(bb);},
			&EnumType::I64 => {11.encode(bb);},
			&EnumType::I128 => {12.encode(bb);},
			&EnumType::I256 => {13.encode(bb);},
			&EnumType::Isize => {14.encode(bb);},
			&EnumType::F32 => {15.encode(bb);},
			&EnumType::F64 => {16.encode(bb);},
			&EnumType::BigI => {17.encode(bb);},
			&EnumType::Str => {18.encode(bb);},
			&EnumType::Bin => {19.encode(bb);},
			&EnumType::UTC => {20.encode(bb);},
			&EnumType::Arr(ref v) => {21.encode(bb); v.encode(bb);},
			&EnumType::Map(ref k, ref v) => {22.encode(bb); k.encode(bb); v.encode(bb);},
			&EnumType::Struct(ref v) => {23.encode(bb); v.encode(bb);},
		};
	}
}

impl Decode for EnumType{
	fn decode(bb:&mut ReadBuffer) -> EnumType{
		let t = u8::decode(bb);
		match t{
			0 => {EnumType::Bool},
			1 => {EnumType::U8},
			2 => {EnumType::U16},
			3 => {EnumType::U32},
			4 => {EnumType::U64},
			5 => {EnumType::U128},
			6 => {EnumType::U256},
			7 => {EnumType::Usize},
			8 => {EnumType::I8},
			9 => {EnumType::I16},
			10 => {EnumType::I32},
			11 => {EnumType::I64},
			12 => {EnumType::I128},
			13 => {EnumType::I256},
			14 => {EnumType::Isize},
			15 => {EnumType::F32},
			16 => {EnumType::F64},
			17 => {EnumType::BigI},
			18 => {EnumType::Str},
			19 => {EnumType::Bin},
			20 => {EnumType::UTC},
			21 => {EnumType::Arr(Arc::new(EnumType::decode(bb)))},
			22 => {EnumType::Map(Arc::new(EnumType::decode(bb)), Arc::new(EnumType::decode(bb)))},
			23 => {EnumType::Struct(Arc::new(StructInfo::decode(bb)))},
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
	pub notes: Option<HashMap<Atom, Atom>>,
	pub fields: Vec<FieldInfo>,
}

impl StructInfo {
	pub fn new(name:Atom, name_hash:u32) -> Self {
		StructInfo {
			name:name,
			name_hash: name_hash,
			notes: None,
			fields: Vec::new(),
		}
	}
	pub fn get_note(&self, key: &Atom) -> Option<&Atom> {
		match self.notes {
			Some(ref map) => map.get(key),
			_ => None
		}
	}
}

impl Encode for StructInfo{
	fn encode(&self, bb: &mut WriteBuffer){
		self.name.encode(bb);
		self.name_hash.encode(bb);
	}
}

impl Decode for StructInfo{
	fn decode(bb: &mut ReadBuffer) -> StructInfo{
		Atom::decode(bb);
		u32::decode(bb);
		Option::<HashMap<Atom, Atom>>::decode(bb);
		Vec::<FieldInfo>::decode(bb);
		StructInfo{
			name: Atom::decode(bb),
			name_hash: u32::decode(bb),
			notes: Option::decode(bb),
			fields: Vec::decode(bb),
		}
	}
}

pub struct FieldInfo {
	pub name: Atom,
	pub ftype: EnumType,
	pub notes: Option<HashMap<Atom, Atom>>,
}


impl FieldInfo{
	pub fn get_note(&self, key: &Atom) -> Option<&Atom> {
		match self.notes {
			Some(ref map) => map.get(key),
			_ => None
		}
	}
}
impl Encode for FieldInfo{
	fn encode(&self, bb: &mut WriteBuffer){
		self.name.encode(bb);
		self.ftype.encode(bb);
		self.notes.encode(bb);
	}
}

impl Decode for FieldInfo{
	fn decode(bb: &mut ReadBuffer) -> FieldInfo{
		FieldInfo{
			name: Atom::decode(bb),
			ftype: EnumType::decode(bb),
			notes: Option::decode(bb),
		}
	}
}
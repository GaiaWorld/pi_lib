/**
 * 结构体信息
 */

use std::vec::Vec;
use std::collections::HashMap;

use atom::Atom;
use bon::{BonCode, BonBuffer};

// 枚举结构体字段的所有类型
pub enum EnumType {
	Void,
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
	Vec,
	Map,
	Tuple(Atom),
	Struct(Atom),
	Enum(Atom),
	Func(Atom),
	Ref(Atom),
}

pub struct StructInfo {
	pub name: Atom,
	pub name_hash: u32,
	pub notes: Option<HashMap<Atom, Atom>>,
	pub fields: Vec<FieldInfo>,
}

pub struct FieldInfo {
	pub name: Atom,
	pub ftype: EnumType,
	pub notes: Option<HashMap<Atom, Atom>>,
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

impl BonCode for StructInfo {

	fn bon_encode(&self, bb: &mut BonBuffer, _: fn(&mut BonBuffer, &Self)) {

	}
	fn bon_decode(bb: &mut BonBuffer, _: fn(&BonBuffer,  &u32) -> Self) -> Self {
		StructInfo {
			name:Atom::from(""),
			name_hash: 0,
			notes: None,
			fields: Vec::new(),
		}
	}

}

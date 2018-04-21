/**
 * 全局的线程安全的常量字符串池
 */

use std::vec::Vec;
use std::collections::HashMap;

use atom::Atom;

// 枚举结构体字段的所有类型
pub enum EnumType {
	Bool,
	U8,
	U16,
	U32,
	U64,
	U128,
	Usize,
	I8,
	I16,
	I32,
	I64,
	I128,
	Isize,
	F32,
	F64,
	Str,
	Vec,
	Struct(Atom),
	Enum(Atom),
	Func(Atom),
}

pub struct StructInfo {
	pub name: Atom,
	pub name_hash: u32,
	pub annotates: Option<HashMap<Atom, Atom>>,
	pub fields: Vec<FieldInfo>,
}

pub struct FieldInfo {
	pub name: Atom,
	pub ftype: EnumType,
	pub annotates: Option<HashMap<Atom, Atom>>,
}
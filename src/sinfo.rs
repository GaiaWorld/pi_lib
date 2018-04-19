/**
 * 全局的线程安全的常量字符串池
 */

use std::sync::Arc;
use std::vec::Vec;
use std::collections::HashMap;

use str::Str;

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
	Struct(Str),
	Enum(Str),
	Func(Str),
}

pub struct StructInfo {
	pub name: Str,
	pub name_hash: u32,
	pub annotates: Option<HashMap<Str, Str>>,
	pub fields: Vec<FieldInfo>,
}

pub struct FieldInfo {
	pub name: Str,
	pub ftype: EnumType,
	pub annotates: Option<HashMap<Str, Str>>,
}
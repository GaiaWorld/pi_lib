#![allow(warnings)]
extern crate rustc_serialize;

pub use rustc_serialize::hex;

use std::{fmt, ops, cmp, str};
use std::hash::{Hash, Hasher};
use std::cmp::{Ordering};
use std::str::FromStr;

use rustc_serialize::hex::{ToHex, FromHex, FromHexError};

/**
* 32位hash值
*/
pub struct H32([u8; 4]);

impl Default for H32 {
    fn default() -> Self {
        H32([0u8; 4])
    }
}

impl AsRef<H32> for H32 {
    fn as_ref(&self) -> &H32 {
        self
    }
}

impl Clone for H32 {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(&self.0);
        result
    }
}

impl From<[u8; 4]> for H32 {
    fn from(h: [u8; 4]) -> Self {
        H32(h)
    }
}

impl From<H32> for [u8; 4] {
    fn from(h: H32) -> Self {
        h.0
    }
}

impl<'a> From<&'a [u8]> for H32 {
    fn from(slc: &[u8]) -> Self {
        let mut inner = [0u8; 4];
        inner[..].clone_from_slice(&slc[0..4]);
        H32(inner)
    }
}

impl From<&'static str> for H32 {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap()
    }
}

impl From<u8> for H32 {
    fn from(v: u8) -> Self {
        let mut result = Self::default();
        result.0[0] = v;
        result
    }
}

impl str::FromStr for H32 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = try!(s.from_hex());
        match vec.len() {
            4 => {
                let mut result = [0u8; 4];
                result.copy_from_slice(&vec);
                Ok(H32(result))
            },
            _ => Err(FromHexError::InvalidHexLength)
        }
    }
}

impl fmt::Debug for H32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl fmt::Display for H32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl ops::Deref for H32 {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for H32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl cmp::PartialEq for H32 {
    fn eq(&self, other: &Self) -> bool {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref == other_ref
    }
}

impl cmp::PartialOrd for H32 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref.partial_cmp(other_ref)
    }
}


impl Hash for H32 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&self.0);
        state.finish();
    }
}

impl Eq for H32 { }

impl H32 {
    /**
    * 获取32位hash值的数组
    * @returns 返回32位hash值的数组
    */
    pub fn take(self) -> [u8; 4] {
        self.0
    }

    /**
    * 获取32位hash值的字符串
    * @returns 返回32位hash值的字符串
    */
    pub fn tohex(&self) -> String {
        self.to_hex()
    }

    /**
    * 将32位hash值的数组，转换为32位hash值
    * @param buf 32位hash值的数组
    * @returns 返回32位hash值
    */
    pub fn from_buf(buf: &[u8]) -> H32{
        H32::from(buf)
    }

    /**
    * 将32位hash值的字符串，转换为32位hash值
    * @param hex 32位hash值的字符串
    * @returns 返回32位hash值
    */
    pub fn fromhex(hex: &str) -> H32{
        H32::from_str(hex).expect("string can not change to H32")
    }

    /**
    * 比较另一个32位hash值
    * @param other 另一个32位hash值
    * @returns 返回比较结果，-1小于，0等于，1大于
    */
    pub fn cmp(&self, other: &H32) -> i8 {
        ord_change(&self.partial_cmp(other).unwrap())
    }

    pub fn reversed(&self) -> Self {
        let mut result = self.clone();
        result.reverse();
        result
    }

    pub fn size() -> usize {
        4
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

/**
* 48位hash值
*/
pub struct H48([u8; 6]);

impl Default for H48 {
    fn default() -> Self {
        H48([0u8; 6])
    }
}

impl AsRef<H48> for H48 {
    fn as_ref(&self) -> &H48 {
        self
    }
}

impl Clone for H48 {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(&self.0);
        result
    }
}

impl From<[u8; 6]> for H48 {
    fn from(h: [u8; 6]) -> Self {
        H48(h)
    }
}

impl From<H48> for [u8; 6] {
    fn from(h: H48) -> Self {
        h.0
    }
}

impl<'a> From<&'a [u8]> for H48 {
    fn from(slc: &[u8]) -> Self {
        let mut inner = [0u8; 6];
        inner[..].clone_from_slice(&slc[0..6]);
        H48(inner)
    }
}

impl From<&'static str> for H48 {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap()
    }
}

impl From<u8> for H48 {
    fn from(v: u8) -> Self {
        let mut result = Self::default();
        result.0[0] = v;
        result
    }
}

impl str::FromStr for H48 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = try!(s.from_hex());
        match vec.len() {
            6 => {
                let mut result = [0u8; 6];
                result.copy_from_slice(&vec);
                Ok(H48(result))
            },
            _ => Err(FromHexError::InvalidHexLength)
        }
    }
}

impl fmt::Debug for H48 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl fmt::Display for H48 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl ops::Deref for H48 {
    type Target = [u8; 6];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for H48 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl cmp::PartialEq for H48 {
    fn eq(&self, other: &Self) -> bool {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref == other_ref
    }
}

impl cmp::PartialOrd for H48 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref.partial_cmp(other_ref)
    }
}


impl Hash for H48 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&self.0);
        state.finish();
    }
}

impl Eq for H48 { }

impl H48 {
    /**
    * 获取48位hash值的数组
    * @returns 返回48位hash值的数组
    */
    pub fn take(self) -> [u8; 6] {
        self.0
    }

    /**
    * 获取48位hash值的字符串
    * @returns 返回48位hash值的字符串
    */
    pub fn tohex(&self) -> String {
        self.to_hex()
    }

    /**
    * 将48位hash值的数组，转换为48位hash值
    * @param buf 48位hash值的数组
    * @returns 返回48位hash值
    */
    pub fn from_buf(buf: &[u8]) -> H48{
        H48::from(buf)
    }

    /**
    * 将48位hash值的字符串，转换为48位hash值
    * @param hex 48位hash值的字符串
    * @returns 返回48位hash值
    */
    pub fn fromhex(hex: &str) -> H48{
        H48::from_str(hex).expect("string can not change to H48")
    }

    /**
    * 比较另一个48位hash值
    * @param other 另一个48位hash值
    * @returns 返回比较结果，-1小于，0等于，1大于
    */
    pub fn cmp(&self, other: &H48) -> i8 {
        ord_change(&self.partial_cmp(other).unwrap())
    }

    pub fn reversed(&self) -> Self {
        let mut result = self.clone();
        result.reverse();
        result
    }

    pub fn size() -> usize {
        6
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

/**
* 160位hash值
*/
pub struct H160([u8; 20]);

impl Default for H160 {
    fn default() -> Self {
        H160([0u8; 20])
    }
}

impl AsRef<H160> for H160 {
    fn as_ref(&self) -> &H160 {
        self
    }
}

impl Clone for H160 {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(&self.0);
        result
    }
}

impl From<[u8; 20]> for H160 {
    fn from(h: [u8; 20]) -> Self {
        H160(h)
    }
}

impl From<H160> for [u8; 20] {
    fn from(h: H160) -> Self {
        h.0
    }
}

impl<'a> From<&'a [u8]> for H160 {
    fn from(slc: &[u8]) -> Self {
        let mut inner = [0u8; 20];
        inner[..].clone_from_slice(&slc[0..20]);
        H160(inner)
    }
}

impl From<&'static str> for H160 {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap()
    }
}

impl From<u8> for H160 {
    fn from(v: u8) -> Self {
        let mut result = Self::default();
        result.0[0] = v;
        result
    }
}

impl str::FromStr for H160 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = try!(s.from_hex());
        match vec.len() {
            20 => {
                let mut result = [0u8; 20];
                result.copy_from_slice(&vec);
                Ok(H160(result))
            },
            _ => Err(FromHexError::InvalidHexLength)
        }
    }
}

impl fmt::Debug for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl fmt::Display for H160 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl ops::Deref for H160 {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for H160 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl cmp::PartialEq for H160 {
    fn eq(&self, other: &Self) -> bool {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref == other_ref
    }
}

impl cmp::PartialOrd for H160 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref.partial_cmp(other_ref)
    }
}


impl Hash for H160 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&self.0);
        state.finish();
    }
}

impl Eq for H160 { }

impl H160 {
    /**
    * 获取160位hash值的数组
    * @returns 返回160位hash值的数组
    */
    pub fn take(self) -> [u8; 20] {
        self.0
    }

    /**
    * 获取160位hash值的字符串
    * @reutrns 返回160位hash值的字符串
    */
    pub fn tohex(&self) -> String {
        self.to_hex()
    }

    /**
    * 将160位hash值的数组，转换为160位hash值
    * @param buf 160位hash值的数组
    * @returns 返回160位hash值
    */
    pub fn from_buf(buf: &[u8]) -> H160{
        H160::from(buf)
    }

    /**
    * 将160位hash值的字符串，转换为160位hash值
    * @param hex 160位hash值的字符串
    * @returns 返回160位hash值
    */
    pub fn fromhex(hex: &str) -> H160{
        H160::from_str(hex).expect("string can not change to H32")
    }

    /**
    * 比较另一个160位hash值
    * @param other 另一个160位hash值
    * @returns 返回比较结果，-1小于，0等于，1大于
    */
    pub fn cmp(&self, other: &H160) -> i8 {
        ord_change(&self.partial_cmp(other).unwrap())
    }

    pub fn reversed(&self) -> Self {
        let mut result = self.clone();
        result.reverse();
        result
    }

    pub fn size() -> usize {
        20
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

/**
* 256位hash值
*/
pub struct H256([u8; 32]);

impl Default for H256 {
    fn default() -> Self {
        H256([0u8; 32])
    }
}

impl AsRef<H256> for H256 {
    fn as_ref(&self) -> &H256 {
        self
    }
}

impl Clone for H256 {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(&self.0);
        result
    }
}

impl From<[u8; 32]> for H256 {
    fn from(h: [u8; 32]) -> Self {
        H256(h)
    }
}

impl From<H256> for [u8; 32] {
    fn from(h: H256) -> Self {
        h.0
    }
}

impl<'a> From<&'a [u8]> for H256 {
    fn from(slc: &[u8]) -> Self {
        let mut inner = [0u8; 32];
        inner[..].clone_from_slice(&slc[0..32]);
        H256(inner)
    }
}

impl From<&'static str> for H256 {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap()
    }
}

impl From<u8> for H256 {
    fn from(v: u8) -> Self {
        let mut result = Self::default();
        result.0[0] = v;
        result
    }
}

impl str::FromStr for H256 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = try!(s.from_hex());
        match vec.len() {
            32 => {
                let mut result = [0u8; 32];
                result.copy_from_slice(&vec);
                Ok(H256(result))
            },
            _ => Err(FromHexError::InvalidHexLength)
        }
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl ops::Deref for H256 {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for H256 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl cmp::PartialEq for H256 {
    fn eq(&self, other: &Self) -> bool {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref == other_ref
    }
}

impl cmp::PartialOrd for H256 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref.partial_cmp(other_ref)
    }
}


impl Hash for H256 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&self.0);
        state.finish();
    }
}

impl Eq for H256 { }

impl H256 {
    /**
    * 获取256位hash值的数组
    * @returns 返回256位hash值的数组
    */
    pub fn take(self) -> [u8; 32] {
        self.0
    }

    /**
    * 获取256位hash值的字符串
    * @returns 返回256位hash值的字符串
    */
    pub fn tohex(&self) -> String {
        self.to_hex()
    }

    /**
    * 将256位hash值的数组，转换为256位hash值
    * @param buf 256位hash值的数组
    * @param 返回256位hash值
    */
    pub fn from_buf(buf: &[u8]) -> H256{
        H256::from(buf)
    }

    /**
    * 将256位hash值的字符串，转换为256位hash值
    * @param hex 256位hash值的字符串
    * @param 返回256位hash值
    */
    pub fn fromhex(hex: &str) -> H256{
        H256::from_str(hex).expect("string can not change to H256")
    }

    /**
    * 比较另一个256位hash值
    * @param other 另一个256位hash值
    * @reutrns 返回比较结果，-1小于，0等于，1大于
    */
    pub fn cmp(&self, other: &H256) -> i8 {
        ord_change(&self.partial_cmp(other).unwrap())
    }

    pub fn reversed(&self) -> Self {
        let mut result = self.clone();
        result.reverse();
        result
    }

    pub fn size() -> usize {
        32
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}


impl H256 {
	#[inline]
	pub fn from_reversed_str(s: &'static str) -> Self {
		H256::from(s).reversed()
	}

	#[inline]
	pub fn to_reversed_str(&self) -> String {
		self.reversed().to_string()
	}
}

/**
* 512位hash值
*/
pub struct H512([u8; 64]);

impl Default for H512 {
    fn default() -> Self {
        H512([0u8; 64])
    }
}

impl AsRef<H512> for H512 {
    fn as_ref(&self) -> &H512 {
        self
    }
}

impl Clone for H512 {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(&self.0);
        result
    }
}

impl From<[u8; 64]> for H512 {
    fn from(h: [u8; 64]) -> Self {
        H512(h)
    }
}

impl From<H512> for [u8; 64] {
    fn from(h: H512) -> Self {
        h.0
    }
}

impl<'a> From<&'a [u8]> for H512 {
    fn from(slc: &[u8]) -> Self {
        let mut inner = [0u8; 64];
        inner[..].clone_from_slice(&slc[0..64]);
        H512(inner)
    }
}

impl From<&'static str> for H512 {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap()
    }
}

impl From<u8> for H512 {
    fn from(v: u8) -> Self {
        let mut result = Self::default();
        result.0[0] = v;
        result
    }
}

impl str::FromStr for H512 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = try!(s.from_hex());
        match vec.len() {
            64 => {
                let mut result = [0u8; 64];
                result.copy_from_slice(&vec);
                Ok(H512(result))
            },
            _ => Err(FromHexError::InvalidHexLength)
        }
    }
}

impl fmt::Debug for H512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl fmt::Display for H512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl ops::Deref for H512 {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for H512 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl cmp::PartialEq for H512 {
    fn eq(&self, other: &Self) -> bool {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref == other_ref
    }
}

impl cmp::PartialOrd for H512 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref.partial_cmp(other_ref)
    }
}


impl Hash for H512 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&self.0);
        state.finish();
    }
}

impl Eq for H512 { }

impl H512 {
    /**
    * 获取512位hash值的数组
    * @returns 返回512位hash值的数组
    */
    pub fn take(self) -> [u8; 64] {
        self.0
    }

    /**
    * 获取512位hash值的字符串
    * @returns 返回512位hash值的字符串
    */
    pub fn tohex(&self) -> String {
        self.to_hex()
    }

    /**
    * 将512位hash值的数组，转换为512位hash值
    * @param buf 512位hash值的数组
    * @returns 返回512位hash值
    */
    pub fn from_buf(buf: &[u8]) -> H512{
        H512::from(buf)
    }

    /**
    * 将512位hash值的字符串，转换为512位hash值
    * @param hex 512位hash值的字符串
    * @returns 返回512位hash值
    */
    pub fn fromhex(hex: &str) -> H512{
        H512::from_str(hex).expect("string can not change to H512")
    }

    /**
    * 比较另一个512位hash值
    * @param other 另一个512位hash值
    * @returns 返回比较结果，-1小于，0等于，1大于
    */
    pub fn cmp(&self, other: &H512) -> i8 {
        ord_change(&self.partial_cmp(other).unwrap())
    }

    pub fn reversed(&self) -> Self {
        let mut result = self.clone();
        result.reverse();
        result
    }

    pub fn size() -> usize {
        64
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

/**
* 520位hash值
*/
pub struct H520([u8; 65]);

impl Default for H520 {
    fn default() -> Self {
        H520([0u8; 65])
    }
}

impl AsRef<H520> for H520 {
    fn as_ref(&self) -> &H520 {
        self
    }
}

impl Clone for H520 {
    fn clone(&self) -> Self {
        let mut result = Self::default();
        result.copy_from_slice(&self.0);
        result
    }
}

impl From<[u8; 65]> for H520 {
    fn from(h: [u8; 65]) -> Self {
        H520(h)
    }
}

impl From<H520> for [u8; 65] {
    fn from(h: H520) -> Self {
        h.0
    }
}

impl<'a> From<&'a [u8]> for H520 {
    fn from(slc: &[u8]) -> Self {
        let mut inner = [0u8; 65];
        inner[..].clone_from_slice(&slc[0..65]);
        H520(inner)
    }
}

impl From<&'static str> for H520 {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap()
    }
}

impl From<u8> for H520 {
    fn from(v: u8) -> Self {
        let mut result = Self::default();
        result.0[0] = v;
        result
    }
}

impl str::FromStr for H520 {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = try!(s.from_hex());
        match vec.len() {
            65 => {
                let mut result = [0u8; 65];
                result.copy_from_slice(&vec);
                Ok(H520(result))
            },
            _ => Err(FromHexError::InvalidHexLength)
        }
    }
}

impl fmt::Debug for H520 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl fmt::Display for H520 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_hex())
    }
}

impl ops::Deref for H520 {
    type Target = [u8; 65];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for H520 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl cmp::PartialEq for H520 {
    fn eq(&self, other: &Self) -> bool {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref == other_ref
    }
}

impl cmp::PartialOrd for H520 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let self_ref: &[u8] = &self.0;
        let other_ref: &[u8] = &other.0;
        self_ref.partial_cmp(other_ref)
    }
}


impl Hash for H520 {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write(&self.0);
        state.finish();
    }
}

impl Eq for H520 { }

impl H520 {
    /**
    * 获取520位hash值的数组
    * @returns 返回520位hash值的数组
    */
    pub fn take(self) -> [u8; 65] {
        self.0
    }

    /**
    * 获取520位hash值的字符串
    * @returns 返回520位hash值的字符串
    */
    pub fn tohex(&self) -> String {
        self.to_hex()
    }

    /**
    * 将520位hash值的数组，转换为520位hash值
    * @param buf 520位hash值的数组
    * @returns 返回520位hash值
    */
    pub fn from_buf(buf: &[u8]) -> H520{
        H520::from(buf)
    }

    /**
    * 将520位hash值的字符串，转换为520位hash值
    * @param hex 520位hash值的字符串
    * @returns 返回520位hash值
    */
    pub fn fromhex(hex: &str) -> H520{
        H520::from_str(hex).expect("string can not change to H520")
    }

    /**
    * 比较另一个520位hash值
    * @param other 另一个520位hash值
    * @returns 返回比较结果，-1小于，0等于，1大于
    */
    pub fn cmp(&self, other: &H520) -> i8 {
        ord_change(&self.partial_cmp(other).unwrap())
    }

    pub fn reversed(&self) -> Self {
        let mut result = self.clone();
        result.reverse();
        result
    }

    pub fn size() -> usize {
        65
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

fn ord_change(ord: &Ordering) -> i8{
    match ord{
        Ordering::Equal => 0,
        Ordering::Greater => 1,
        Ordering::Less => -1,
    }
}


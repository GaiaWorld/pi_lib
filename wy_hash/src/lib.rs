//! WyHash

#![allow(warnings)]

#![feature(exclusive_range_pattern)]


use std::{hash::Hasher};

use rand_core::{RngCore, SeedableRng};
use wy_rng::mum;

pub const P0: u64 = 0xa076_1d64_78bd_642f;
pub const P1: u64 = 0xe703_7ed1_a0b4_28db;
pub const P2: u64 = 0x8ebc_6af0_9c88_c6e3;
pub const P3: u64 = 0x5899_65cc_7537_4cc3;

/// WyHash hasher
#[derive(Clone)]
pub struct WyHash {
    seed: u64,
    size: u64,
    secret: [u64; 4],
}

impl WyHash {
    /// Create hasher with a seed
    #[inline]
    pub fn new(seed: u64, secret: [u64; 4]) -> Self {
        WyHash {
            seed,
            size: 0,
            secret,
        }
    }
    #[inline]
    pub fn wyhash(&mut self, bytes: &[u8]) {
        self.seed ^= self.secret[0];
        let mut start = 0;
        let len = bytes.len();
        self.size += len as u64;
        if len >= start + 48 {
            let mut seed1 = self.seed;
            let mut seed2 = self.seed;
            while len >= start + 48 {
                self.seed = mum(
                    read_le(&bytes[start..]) ^ self.secret[1],
                    read_le(&bytes[start + 8..]) ^ self.seed,
                );
                seed1 = mum(
                    read_le(&bytes[start + 16..]) ^ self.secret[2],
                    read_le(&bytes[start + 24..]) ^ seed1,
                );
                seed2 = mum(
                    read_le(&bytes[start + 32..]) ^ self.secret[3],
                    read_le(&bytes[start + 40..]) ^ seed2,
                );
                start += 48;
            }
            self.seed ^= seed1 ^ seed2;
        }
        while len >= start + 16 {
            self.seed = mum(
                read_le(&bytes[start..]) ^ self.secret[1],
                read_le(&bytes[start + 8..]) ^ self.seed,
            );
            start += 16;
        }
        if len >= start + 8 {
            self.seed = mum(
                read_le(&bytes[start..]) ^ self.secret[1],
                self.seed,
            );
            start += 8;
        }
        if len > start {
            self.seed = mum(
                read_le_0_7(&bytes[start..]) ^ self.secret[1],
                self.seed,
            );
        }
    }
}

impl Default for WyHash {
    #[inline(always)]
    fn default() -> Self {
        WyHash::new(0, [P0, P1, P2, P3])
    }
}

impl Hasher for WyHash {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() > 0 {
            self.wyhash(bytes);
        }
    }
    #[inline(always)]
    fn finish(&self) -> u64 {
        mum(self.secret[1] ^ self.size, self.seed)
    }
    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.seed = mum((i as u64) ^ self.secret[1], self.seed ^ self.secret[0]);
        self.size += 1;
    }
    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.seed = mum((i as u64) ^ self.secret[1], self.seed ^ self.secret[0]);
        self.size += 2;
    }
    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.seed = mum((i as u64) ^ self.secret[1], self.seed ^ self.secret[0]);
        self.size += 4;
    }
    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.seed = mum(i ^ self.secret[1], self.seed ^ self.secret[0]);
        self.size += 8;
    }
    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.seed = mum((i as u64) ^ self.secret[1], ((i >> 64) as u64) ^ self.seed ^ self.secret[0]);
        self.size += 16;
    }
    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.seed = mum((i as u64) ^ self.secret[1], self.seed ^ self.secret[0]);
        self.size += usize::BITS as u64;
    }
}


/// 读取0-7字节的u64
#[inline]
pub fn read_le_0_7(bytes: &[u8]) -> u64 {
    match bytes.len() {
        1 => bytes[0] as u64,
        2 => unsafe {u16::from_le_bytes(*(bytes as *const _ as *const [u8; 2])) as u64 },
        3 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]) as u64,
        4 => unsafe {u32::from_le_bytes(*(bytes as *const _ as *const [u8; 4])) as u64 }
        5 => u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], 0, 0, 0]) as u64,
        6 => u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0]) as u64,
        7 => u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], 0]) as u64,
        _ => 0,
    }
}
/// 读取大于等于8字节的u64
#[inline(always)]
pub fn read_le(bytes: &[u8]) -> u64 {
    unsafe {u64::from_le_bytes(*(bytes as *const _ as *const [u8; 8])) }
    //u64::from_le_bytes(bytes.try_into().unwrap())
}

/// Generate new secret for wyhash
pub fn make_secret(seed: u64) -> [u64; 4] {
    let c = [
        15_u8, 23, 27, 29, 30, 39, 43, 45, 46, 51, 53, 54, 57, 58, 60, 71, 75, 77, 78, 83, 85, 86,
        89, 90, 92, 99, 101, 102, 105, 106, 108, 113, 114, 116, 120, 135, 139, 141, 142, 147, 149,
        150, 153, 154, 156, 163, 165, 166, 169, 170, 172, 177, 178, 180, 184, 195, 197, 198, 201,
        202, 204, 209, 210, 212, 216, 225, 226, 228, 232, 240,
    ];
    let mut secret = [0_u64; 4];
    let mut rng = wy_rng::WyRng::seed_from_u64(seed);
    for i in 0..secret.len() {
        loop {
            secret[i] = 0;
            for j in (0..64).step_by(8) {
                secret[i] |= u64::from(c[((rng.next_u64() as usize) % c.len())]) << j;
            }
            if secret[i] % 2 == 0 {
                continue;
            }
            let incorrect_number_of_ones_found = (0..i)
                .step_by(1)
                .find(|j| (secret[*j] ^ secret[i]).count_ones() != 32);
            if incorrect_number_of_ones_found.is_none() {
                break;
            }
        }
    }
    secret
}

#[test]
fn test() {
    let mut h = WyHash::default();
    h.write_u8(1);
    println!("1: {}", h.finish());
    h.write_u8(1);
    println!("1: {}", h.finish());
    h = WyHash::default();
    h.write(&[1]);
    println!("1: {}", h.finish());
    h.write(&[1]);
    println!("11: {}", h.finish());
    h = WyHash::default();
    h.write(b"hellhell");
    //h.write(&[]);
    println!("hello: {}", h.finish());
    h.write(b"worlworl");
    println!("helloworld: {}", h.finish());
    //assert_eq!(h.finish(), 14277199482324177244); // 9723359729180093834
    h = WyHash::default();
    h.write(b"hellhellworlworl");
    println!("helloworld: {}", h.finish());
//     h = WyHash::default();
//     h.write_u8(0);
//     assert_eq!(h.finish(), 2495792281036420879);
//     h.write_u32(11);
//     assert_eq!(h.finish(), 13451875736397521462);
//     h.write(&[0]);
//     println!("hash: {}", h.finish());
//     assert_eq!(h.finish(), 14686941155276824898);
//     h.write(&[0,2,3,45,54,6,67,4,8,9,45,54,6,67,4,8,9,45,54,6,67,4,8,9,9,45,54,6,67,4,8,9,9,45,54,6,67,4,8,9,9,45,54,6,67,4,8,9,9,45,54,6,67,4,8,9,9,45,54,6,67,4,8,9,]);
//     println!("hash: {}", h.finish());
//     assert_eq!(h.finish(), 5476865104113569038);
}
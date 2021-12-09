//! WyRand

#![allow(warnings)]
#![feature(exclusive_range_pattern)]

use std::fmt;
use rand::{Error, prelude::*};

/// mum 函数
#[inline(always)]
pub fn mum(a: u64, b: u64) -> u64 {
    let r = (a as u128) * (b as u128);
    ((r >> 64) as u64) ^ (r as u64)
    // let r = u128::from(a).wrapping_mul(u128::from(b));
    // (r.wrapping_shr(64) as u64) ^ (r as u64)
}

#[derive(Default, Clone)]
pub struct WyRng(u64);

impl fmt::Debug for WyRng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WyRng").field(&self.0).finish()
    }
}
impl RngCore for WyRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0xa0761d6478bd642f);
        mum(self.0, self.0 ^ 0xe7037ed1a0b428db)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut left = dest;
        while left.len() >= 8 {
            let (l, r) = { left }.split_at_mut(8);
            left = r;
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            l.copy_from_slice(&chunk);
        }
        let n = left.len();
        if n > 4 {
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        } else if n > 0 {
            let chunk: [u8; 4] = self.next_u32().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl SeedableRng for WyRng {
    type Seed = [u8; 8];
    #[inline(always)]
    fn from_seed(seed: Self::Seed) -> Self {
        WyRng(u64::from_le_bytes(seed))
    }
    #[inline(always)]
    fn seed_from_u64(state: u64) -> Self {
        WyRng(state)
    }
}

#[test]
fn test() {
    let mut r = WyRng::seed_from_u64(100000000);
    assert_eq!(r.next_u64(), 10554334685998524674);
}

#[test]
fn test_thread_rng() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    println!("u8: {}", rng.gen::<u8>());
    println!("i8: {}", rng.gen::<i8>());
    println!("u16: {}", rng.gen::<u16>());
    println!("i16: {}", rng.gen::<i16>());

    println!("f32: {}", rng.gen::<f32>());
    println!("f64: {}", rng.gen::<f64>());

    println!("[1.3, 2.6] f32: {}", rng.gen_range(1.3f32..2.6f32));
}

#[test]
fn test_wy_rng() {
    use rand::Rng;

    let mut rng = WyRng::seed_from_u64(1000);

    println!("u8: {}", rng.gen::<u8>());
    println!("i8: {}", rng.gen::<i8>());
    println!("u16: {}", rng.gen::<u16>());
    println!("i16: {}", rng.gen::<i16>());

    println!("f32: {}", rng.gen::<f32>());
    println!("f64: {}", rng.gen::<f64>());

    println!("[1.3, 2.6] f32: {}", rng.gen_range(1.3f32..2.6f32));
}
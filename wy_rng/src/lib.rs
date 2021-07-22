//! WyRand

#![allow(warnings)]
#![feature(exclusive_range_pattern)]

extern crate rand_core;

use std::fmt;

use rand_core::{impls, Error, RngCore, SeedableRng};

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
        impls::fill_bytes_via_next(self, dest)
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

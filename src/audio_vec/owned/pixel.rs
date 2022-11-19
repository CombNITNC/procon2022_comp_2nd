use cast::{i64, u64};
use num::{integer::ExtendedGcd, Integer};
use serde::{Deserialize, Serialize};
use std::ops;

use super::mod_int::{ModInt, ModInt924844033, ModInt998244353};

/// 924844033 と 924844033 の 2 種類の法における剰余を格納する. Garner のアルゴリズムにより 924844033 × 998244353 = 923220333347995649 を法とした値を求める.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pixel(ModInt924844033, ModInt998244353);

impl PartialOrd for Pixel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pixel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u64().cmp(&other.as_u64())
    }
}

fn garner(m1: ModInt924844033, m2: ModInt998244353) -> u64 {
    let r1 = m1.as_u32();
    let m1 = ModInt924844033::N;
    let r2 = m2.as_u32();
    let m2 = ModInt998244353::N;
    let (ExtendedGcd { gcd, x, .. }, lcm) = i64(m1).extended_gcd_lcm(&i64(m2));
    // 924844033 * x + 998244353 * y = gcd = 1
    debug_assert_eq!(gcd, 1, "not co-prime modulo");
    let diff = r1.abs_diff(r2);
    let tmp = i64(diff) / gcd * x % (i64(m2) / gcd);
    u64((i64(r1) + i64(m1) * tmp).rem_euclid(lcm)).unwrap()
}

impl Pixel {
    #[inline]
    pub fn from_signed(value: i64) -> Self {
        Self(ModInt::from_signed(value), ModInt::from_signed(value))
    }

    #[inline]
    #[cfg(test)]
    pub fn from_unsigned(value: u64) -> Self {
        Self(ModInt::new(value), ModInt::new(value))
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        garner(self.0, self.1)
    }

    #[inline]
    pub fn into_inner(self) -> (ModInt924844033, ModInt998244353) {
        (self.0, self.1)
    }

    #[inline]
    pub unsafe fn from_inner(tuple: (ModInt924844033, ModInt998244353)) -> Self {
        Self(tuple.0, tuple.1)
    }

    #[inline]
    pub fn clamp(self, min: i64, max: i64) -> Self {
        Self(self.0.clamp(min, max), self.1.clamp(min, max))
    }
}

impl ops::Add for Pixel {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl ops::AddAssign for Pixel {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
        self.1 = self.1 + rhs.1;
    }
}

impl ops::Sub for Pixel {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl ops::Mul for Pixel {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl std::iter::Sum for Pixel {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        use ops::Add;
        iter.fold(Default::default(), Pixel::add)
    }
}

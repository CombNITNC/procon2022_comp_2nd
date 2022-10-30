use num::{traits::Pow, One, Zero};
use serde::{Deserialize, Serialize};

const R: u64 = 1 << 32;

/// modulo * modulo_inv ≡ -1 (mod R) となる modulo_inv を求める.
const fn find_neg_inv(modulo: u32) -> u32 {
    let mut inv_mod = 0u32;
    let mut t = 0;
    let mut i = 1u32;
    loop {
        if t % 2 == 0 {
            t += modulo;
            inv_mod = inv_mod.wrapping_add(i);
        }
        t /= 2;
        if let Some(next_i) = i.checked_mul(2) {
            i = next_i;
        } else {
            break;
        }
    }
    inv_mod
}

const fn find_r2(modulo: u32) -> u32 {
    let modulo = modulo as u64;
    let r = R % modulo;
    (r * r % modulo) as u32
}

pub type ModInt998244353 = ModInt<998244353>;
#[test]
fn const_test_998244353() {
    assert_eq!(ModInt998244353::N, 0x3B800001);
    assert_eq!(ModInt998244353::N_PRIME, 0x3B7FFFFF);
    assert_eq!(ModInt998244353::R2, 0x378DFBC6);
}

/// MOD を法として 2^32 を掛けたモンゴメリ表現. MOD は素数であることが期待される.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct ModInt<const MOD: u32>(u32);

impl<const MOD: u32> From<ModInt<MOD>> for u32 {
    #[inline]
    fn from(mod_int: ModInt<MOD>) -> Self {
        mod_int.as_u32()
    }
}

impl<const MOD: u32> ModInt<MOD> {
    #[inline]
    pub fn new(n: u32) -> Self {
        Self(Self::reduce(n as u64 * Self::R2 as u64))
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        Self::reduce(self.0 as u64)
    }

    /// この剰余である整数の法. 素数であるとする.
    pub const N: u32 = MOD;
    /// N のモジュラー逆数. N * N_PRIME ≡ -1 となる数.
    pub const N_PRIME: u32 = find_neg_inv(MOD);
    /// モンゴメリ表現に用いる係数 R ≡ 2^32 を 2 乗した数. R2 ≡ 2^64 となる数.
    pub const R2: u32 = find_r2(MOD);

    #[inline]
    pub fn inv(self) -> Self {
        self.pow(MOD - 2)
    }

    #[inline]
    pub fn reduce(x: u64) -> u32 {
        let modulo = MOD as u64;
        debug_assert!(x < modulo * R as u64);

        let x_n_prime = (x as u32).wrapping_mul(Self::N_PRIME) as u64;
        let mul = (x + x_n_prime * modulo) / R;
        let ret = if modulo <= mul { mul - modulo } else { mul };

        debug_assert!(ret < modulo);
        ret as u32
    }
}

macro_rules! impl_from_for_mod_int {
    ($t:ty) => {
        impl<const MOD: u32> From<$t> for ModInt<MOD> {
            #[inline]
            fn from(x: $t) -> Self {
                Self::new(x as u32)
            }
        }
        impl<const MOD: u32> From<&'_ $t> for ModInt<MOD> {
            #[inline]
            fn from(&x: &'_ $t) -> Self {
                Self::new(x as u32)
            }
        }
    };
}

impl_from_for_mod_int!(u64);
impl_from_for_mod_int!(u32);
impl_from_for_mod_int!(i32);

impl<const MOD: u32> std::ops::Add for ModInt<MOD> {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<const MOD: u32> std::ops::AddAssign for ModInt<MOD> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        if MOD <= self.0 {
            self.0 -= MOD;
        }
    }
}

impl<const MOD: u32> std::ops::Sub for ModInt<MOD> {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<const MOD: u32> std::ops::SubAssign for ModInt<MOD> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        if let Some(sub) = self.0.checked_sub(rhs.0) {
            self.0 = sub;
        } else {
            self.0 += MOD;
            self.0 -= rhs.0;
        }
    }
}

impl<const MOD: u32> std::ops::Mul for ModInt<MOD> {
    type Output = Self;

    #[inline]
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<const MOD: u32> std::ops::MulAssign for ModInt<MOD> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = Self::reduce(self.0 as u64 * rhs.0 as u64);
    }
}

impl<const MOD: u32> std::ops::Div for ModInt<MOD> {
    type Output = Self;

    #[inline]
    fn div(mut self, rhs: Self) -> Self::Output {
        self /= rhs;
        self
    }
}

impl<const MOD: u32> std::ops::DivAssign for ModInt<MOD> {
    #[inline]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn div_assign(&mut self, rhs: Self) {
        *self *= rhs.inv();
    }
}

impl<const MOD: u32> std::ops::Neg for ModInt<MOD> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(0) - self
    }
}

impl<const MOD: u32> std::iter::Sum for ModInt<MOD> {
    #[inline]
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self(0), |a, b| a + b)
    }
}

impl<const MOD: u32> std::iter::Product for ModInt<MOD> {
    #[inline]
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::new(1), |a, b| a * b)
    }
}

impl<const MOD: u32> Zero for ModInt<MOD> {
    #[inline]
    fn zero() -> Self {
        Self(0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl<const MOD: u32> One for ModInt<MOD> {
    #[inline]
    fn one() -> Self {
        Self::new(1)
    }
}

impl<const MOD: u32> Pow<u32> for ModInt<MOD> {
    type Output = Self;

    #[inline]
    fn pow(mut self, mut exp: u32) -> Self::Output {
        if exp == 0 {
            return Self::new(1);
        }
        let mut y = Self::new(1);
        while 0 < exp {
            if exp % 2 == 1 {
                y *= self;
            }
            self *= self;
            exp /= 2;
        }
        y
    }
}

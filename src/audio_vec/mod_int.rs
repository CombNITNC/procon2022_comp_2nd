use num::{One, Zero};
use wrapping_proc_macro::wrapping;

const fn find_neg_inv(n: u32) -> u32 {
    let n = n as i64;
    ((-n) % n) as u32
}

const fn find_r(n: u32) -> u32 {
    let n = n as i64;
    let mut ret = n;
    wrapping! {
        ret *= 2i64 - n * ret;
        ret *= 2i64 - n * ret;
        ret *= 2i64 - n * ret;
        ret *= 2i64 - n * ret;
    }
    ret as u32
}

pub type ModInt998244353 = ModInt<998244353>;
pub type ModInt924844033 = ModInt<924844033>;

/// MOD を法としたモンゴメリ表現. MOD は素数であることが期待される.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        debug_assert_eq!(Self::R.wrapping_mul(Self::N), 1);
        debug_assert!(Self::N < (1 << 30));
        debug_assert_eq!(Self::N % 2, 1);
        Self::reduce(n as u64 % MOD as u64)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// この剰余である整数の法. 素数であるとする.
    pub const N: u32 = MOD;
    /// N のモジュラー逆数. N * N_PRIME ≡ -1 となる数.
    pub const N_PRIME: u32 = find_neg_inv(MOD);
    /// N の逆数. N * R ≡ 1 となる数.
    pub const R: u32 = find_r(MOD);

    #[inline]
    pub fn pow(mut self, mut exp: u32) -> Self {
        if exp == 0 {
            return Self(1);
        }
        let mut y = Self(1);
        while 0 < exp {
            if exp % 2 == 1 {
                y *= self;
            }
            self *= self;
            exp /= 2;
        }
        y
    }

    #[inline]
    pub fn inv(self) -> Self {
        self.pow(MOD - 2)
    }

    #[inline]
    pub fn factorial(self) -> Self {
        let mut result = Self(1);
        for x in 2..=self.0 {
            result *= Self(x);
        }
        result
    }

    #[inline]
    pub const fn reduce(t: u64) -> Self {
        let n = Self::N as u64;
        let r = Self::R as u64;
        let n_prime = Self::N_PRIME as u64;
        let t_prime = wrapping! {
            (t + (t * n_prime % r) * n) / r
        };
        Self(if n <= t_prime { t_prime - n } else { t_prime } as u32)
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
        wrapping! {
            self.0 += rhs.0 - 2u32 * MOD;
        }
        if (self.0 as i32) < 0 {
            wrapping! {
                self.0 += 2u32 * MOD;
            }
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
        wrapping! {
            self.0 -= rhs.0;
        }
        if (self.0 as i32) < 0 {
            wrapping! {
                self.0 += 2u32 * MOD;
            }
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
        *self = Self::reduce(self.0 as u64 * rhs.0 as u64);
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
        iter.fold(Self(1), |a, b| a * b)
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
        Self(1)
    }
}

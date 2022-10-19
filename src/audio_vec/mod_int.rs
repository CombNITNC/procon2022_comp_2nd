use num::{One, Zero};

#[inline]
fn reduce<const MOD: u32, const MOD_INV: u32>(n: u64) -> u32 {
    debug_assert_eq!(MOD.wrapping_mul(MOD_INV), 1);
    ((n + (n * -(MOD_INV as i64) as u64 * MOD as u64)) >> 32) as u32
}

pub type ModInt998244353 = ModInt<998244353, 3296722945>;

/// MOD を法とした整数. MOD.wrapping_mul(MOD_INV) が 1 でなければならない.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModInt<const MOD: u32, const MOD_INV: u32>(u32);

impl<const MOD: u32, const MOD_INV: u32> From<ModInt<MOD, MOD_INV>> for u32 {
    #[inline]
    fn from(mod_int: ModInt<MOD, MOD_INV>) -> Self {
        mod_int.as_u32()
    }
}

impl<const MOD: u32, const MOD_INV: u32> ModInt<MOD, MOD_INV> {
    #[inline]
    pub fn new(n: u32) -> Self {
        debug_assert_eq!(MOD.wrapping_mul(MOD_INV), 1);
        Self(n)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn modulo(&self) -> u32 {
        MOD
    }

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
}

macro_rules! impl_from_for_mod_int {
    ($t:ty) => {
        impl<const MOD: u32, const MOD_INV: u32> From<$t> for ModInt<MOD, MOD_INV> {
            #[inline]
            fn from(x: $t) -> Self {
                Self(x as u32 % MOD)
            }
        }
        impl<const MOD: u32, const MOD_INV: u32> From<&'_ $t> for ModInt<MOD, MOD_INV> {
            #[inline]
            fn from(&x: &'_ $t) -> Self {
                Self(x as u32 % MOD)
            }
        }
    };
}

impl_from_for_mod_int!(u64);
impl_from_for_mod_int!(u32);
impl_from_for_mod_int!(i32);

impl<const MOD: u32, const MOD_INV: u32> std::ops::Add for ModInt<MOD, MOD_INV> {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::AddAssign for ModInt<MOD, MOD_INV> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0 - 2 * MOD;
        if (self.0 as i32) < 0 {
            self.0 += 2 * MOD;
        }
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::Sub for ModInt<MOD, MOD_INV> {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::SubAssign for ModInt<MOD, MOD_INV> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        if (self.0 as i32) < 0 {
            self.0 += 2 * MOD;
        }
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::Mul for ModInt<MOD, MOD_INV> {
    type Output = Self;

    #[inline]
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::MulAssign for ModInt<MOD, MOD_INV> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = reduce::<MOD, MOD_INV>(self.0 as u64 * rhs.0 as u64);
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::Div for ModInt<MOD, MOD_INV> {
    type Output = Self;

    #[inline]
    fn div(mut self, rhs: Self) -> Self::Output {
        self /= rhs;
        self
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::DivAssign for ModInt<MOD, MOD_INV> {
    #[inline]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn div_assign(&mut self, rhs: Self) {
        *self *= rhs.inv();
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::ops::Neg for ModInt<MOD, MOD_INV> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(0) - self
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::iter::Sum for ModInt<MOD, MOD_INV> {
    #[inline]
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self(0), |a, b| a + b)
    }
}

impl<const MOD: u32, const MOD_INV: u32> std::iter::Product for ModInt<MOD, MOD_INV> {
    #[inline]
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self(1), |a, b| a * b)
    }
}

impl<const MOD: u32, const MOD_INV: u32> Zero for ModInt<MOD, MOD_INV> {
    #[inline]
    fn zero() -> Self {
        Self(0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl<const MOD: u32, const MOD_INV: u32> One for ModInt<MOD, MOD_INV> {
    #[inline]
    fn one() -> Self {
        Self(1)
    }
}

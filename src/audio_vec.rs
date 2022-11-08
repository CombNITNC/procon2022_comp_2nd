use cast::{i64, u64};
use num::{integer::ExtendedGcd, Integer};
use serde::{Deserialize, Serialize};

use self::{
    mod_int::{ModInt, ModInt924844033, ModInt998244353},
    ntt::Ntt,
};

pub mod mod_int;
pub mod ntt;

/// 音声データのベクトル. 計算結果は 923220333347995649 を法とした値になる.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioVec {
    // 互いに素な整数 924844033 と 998244353 を使った 2 種類の波形データ. Garner のアルゴリズムにより 924844033 × 998244353 = 923220333347995649 を法とした値を求める.
    vec1: Vec<ModInt924844033>,
    vec2: Vec<ModInt998244353>,
    /// 要素にアクセスするときに添字にこの値を足す. 遅れ方向が正になる.
    delay: isize,
}

/// `x ≡ r1 (mod 924844033), x ≡ r2 (mod 998244353)` となる `x` を求める.
fn garner(r1: ModInt924844033, r2: ModInt998244353) -> u64 {
    let r1 = r1.as_u32();
    let m1 = ModInt924844033::N;
    let r2 = r2.as_u32();
    let m2 = ModInt998244353::N;
    let (ExtendedGcd { gcd, x, .. }, lcm) = i64(m1).extended_gcd_lcm(&i64(m2));
    // 924844033 * x + 998244353 * y = gcd = 1
    debug_assert_eq!(gcd, 1, "not co-prime modulo");

    let diff = r1.abs_diff(r2);
    let tmp = i64(diff) / gcd * x % (i64(m2) / gcd);
    u64((i64(r1) + i64(m1) * tmp).rem_euclid(lcm)).unwrap()
}

impl AudioVec {
    #[inline]
    pub fn len(&self) -> usize {
        self.vec1.len()
    }

    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        if self.vec1.len() < other.vec1.len() {
            return other.add(self);
        }
        let mut cloned = self.clone();
        cloned.add_assign(other);
        cloned
    }

    #[inline]
    pub fn add_assign(&mut self, other: &Self) {
        for i in 0..self.vec1.len() {
            self.vec1[i] += other.vec1[(i as isize - self.delay + other.delay) as usize];
            self.vec2[i] += other.vec2[(i as isize - self.delay + other.delay) as usize];
        }
    }

    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        if self.vec1.len() < other.vec1.len() {
            return other.add(self);
        }
        let mut cloned = self.clone();
        cloned.sub_assign(other);
        cloned
    }

    #[inline]
    pub fn sub_assign(&mut self, other: &Self) {
        for i in 0..self.vec1.len() {
            self.vec1[i] -= other.vec1[(i as isize - self.delay + other.delay) as usize];
            self.vec2[i] -= other.vec2[(i as isize - self.delay + other.delay) as usize];
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.vec1
            .iter()
            .zip(self.vec2.iter())
            .map(|(&a, &b)| garner(a, b))
    }

    #[inline]
    pub fn squared(&self) -> impl Iterator<Item = u64> + '_ {
        self.iter().map(|a| a * a)
    }

    #[inline]
    pub fn squared_norm(&self) -> u64 {
        self.squared().sum()
    }

    #[inline]
    pub fn delayed(&mut self, delay: isize) {
        self.delay = delay;
    }

    #[inline]
    pub fn flip(&mut self) {
        self.vec1.reverse();
        self.vec2.reverse();
        self.delay = -self.delay;
    }

    #[inline]
    pub fn clip(&mut self) {
        for elem in self.vec1.iter_mut() {
            *elem = (*elem).min(ModInt::new(u16::MAX as u32));
        }
        for elem in self.vec2.iter_mut() {
            *elem = (*elem).min(ModInt::new(u16::MAX as u32));
        }
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.vec1.resize(len, Default::default());
        self.vec2.resize(len, Default::default());
    }

    #[inline]
    pub fn convolution(
        &self,
        other: &Self,
        (ntt1, ntt2): (&Ntt<924844033>, &Ntt<998244353>),
    ) -> Vec<u64> {
        if self.vec1.is_empty() && other.vec1.is_empty() {
            return vec![];
        }

        let len = self.vec1.len() + other.vec1.len() - 1;
        if self.vec1.len().min(other.vec1.len()) <= 40 {
            // too tiny vectors
            let mut res = vec![0; len];
            for (i, (&left1, &left2)) in self.vec1.iter().zip(self.vec2.iter()).enumerate() {
                for (j, (&right1, &right2)) in other.vec1.iter().zip(other.vec2.iter()).enumerate()
                {
                    res[i + j] += garner(left1 * right1, left2 * right2);
                }
            }
            return res;
        }

        let buf_len = len.next_power_of_two();

        let convolution_924844033 = {
            let mut buf1 = self.vec1.clone();
            let mut buf2 = other.vec1.clone();
            buf1.resize(buf_len, Default::default());
            buf2.resize(buf_len, Default::default());
            ntt1.transform(&mut buf1);
            ntt1.transform(&mut buf2);
            for (elem1, elem2) in buf1.iter_mut().zip(buf2.iter_mut()) {
                *elem1 *= *elem2;
            }
            ntt1.inverse_transform(&mut buf1);
            buf1
        };
        let convolution_998244353 = {
            let mut buf1 = self.vec2.clone();
            let mut buf2 = other.vec2.clone();
            buf1.resize(buf_len, Default::default());
            buf2.resize(buf_len, Default::default());
            ntt2.transform(&mut buf1);
            ntt2.transform(&mut buf2);
            for (elem1, elem2) in buf1.iter_mut().zip(buf2.iter_mut()) {
                *elem1 *= *elem2;
            }
            ntt2.inverse_transform(&mut buf1);
            buf1
        };
        convolution_924844033
            .into_iter()
            .zip(convolution_998244353)
            .map(|(a, b)| garner(a, b))
            .collect()
    }
}

impl AudioVec {
    #[inline]
    pub fn from_pcm(pcm: &[i16]) -> Self {
        let vec1 = pcm
            .iter()
            .map(|&x| {
                if 0 <= x {
                    x as i64
                } else {
                    (x as i64) + ModInt924844033::N as i64
                }
            })
            .map(|x| x.try_into().unwrap_or_else(|_| panic!("{x}")))
            .map(ModInt924844033::new)
            .collect();
        let vec2 = pcm
            .iter()
            .map(|&x| {
                if 0 <= x {
                    x as i64
                } else {
                    (x as i64) + ModInt998244353::N as i64
                }
            })
            .map(|x| x.try_into().unwrap_or_else(|_| panic!("{x}")))
            .map(ModInt998244353::new)
            .collect();

        Self {
            vec1,
            vec2,
            delay: 0,
        }
    }

    #[inline]
    #[cfg(test)]
    pub fn from_raw_slice(slice: &[u32]) -> Self {
        Self {
            vec1: slice.iter().copied().map(ModInt::new).collect(),
            vec2: slice.iter().copied().map(ModInt::new).collect(),
            delay: 0,
        }
    }
}

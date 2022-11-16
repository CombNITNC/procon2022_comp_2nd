use cast::usize;
use serde::{Deserialize, Serialize};

use self::{ntt::Ntt, pixel::Pixel};
use super::AudioVec;

pub mod mod_int;
pub mod ntt;
pub mod pixel;

/// 音声データのベクトル.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Owned {
    vec: Vec<Pixel>,
}

impl AudioVec for Owned {
    fn get(&self, index: isize) -> Pixel {
        usize(index)
            .ok()
            .and_then(|index| self.vec.get(index))
            .copied()
            .unwrap_or_default()
    }
}

impl Owned {
    #[inline]
    pub const fn new() -> Self {
        Self { vec: vec![] }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn squared(&self) -> impl Iterator<Item = u64> + '_ {
        self.vec.iter().map(|px| px.as_u64()).map(|x| x * x)
    }

    #[inline]
    pub fn squared_norm(&self) -> u64 {
        self.squared().sum()
    }

    #[inline]
    pub fn convolution(
        &self,
        other: &Self,
        (ntt1, ntt2): (&Ntt<924844033>, &Ntt<998244353>),
    ) -> Vec<u64> {
        if self.is_empty() && other.is_empty() {
            return vec![];
        }

        let len = self.len() + other.len() - 1;
        if self.len().min(other.len()) <= 40 {
            // too tiny vectors
            let mut res = vec![0; len];
            for (i, &left) in self.vec.iter().enumerate() {
                for (j, &right) in other.vec.iter().enumerate() {
                    res[i + j] += left.convolution(right);
                }
            }
            return res;
        }

        let buf_len = len.next_power_of_two();
        let (mut self_left, mut self_right): (Vec<_>, Vec<_>) =
            self.vec.iter().map(|px| px.into_inner()).unzip();
        let (mut other_left, mut other_right): (Vec<_>, Vec<_>) =
            other.vec.iter().map(|px| px.into_inner()).unzip();

        let convolution_924844033 = {
            self_left.resize(buf_len, Default::default());
            other_left.resize(buf_len, Default::default());
            ntt1.transform(&mut self_left);
            ntt1.transform(&mut other_left);
            for (elem1, elem2) in self_left.iter_mut().zip(other_left.iter_mut()) {
                *elem1 *= *elem2;
            }
            ntt1.inverse_transform(&mut self_left);
            self_left
        };
        let convolution_998244353 = {
            self_right.resize(buf_len, Default::default());
            other_right.resize(buf_len, Default::default());
            ntt2.transform(&mut self_right);
            ntt2.transform(&mut other_right);
            for (elem1, elem2) in self_right.iter_mut().zip(other_right.iter_mut()) {
                *elem1 *= *elem2;
            }
            ntt2.inverse_transform(&mut self_right);
            self_right
        };
        convolution_924844033
            .into_iter()
            .zip(convolution_998244353)
            .map(|(a, b)|
                // SAFETY: この内部表現は同じ畳み込み演算の結果であり、整合性が保たれている。
            unsafe { Pixel::from_inner((a, b)) })
            .map(|px| px.as_u64())
            .collect()
    }
}

impl Owned {
    #[inline]
    pub fn from_pixels(pixels: impl IntoIterator<Item = Pixel>) -> Self {
        Self {
            vec: pixels.into_iter().collect(),
        }
    }

    #[inline]
    pub fn from_pcm(pcm: &[i16]) -> Self {
        Self {
            vec: pcm
                .iter()
                .map(|&x| x as i64)
                .map(Pixel::from_signed)
                .collect(),
        }
    }

    #[inline]
    #[cfg(test)]
    pub fn from_raw_slice(slice: &[u32]) -> Self {
        Self {
            vec: slice.iter().copied().map(Pixel::from_unsigned).collect(),
        }
    }
}

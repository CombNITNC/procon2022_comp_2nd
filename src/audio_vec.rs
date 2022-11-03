use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use self::mod_int::ModInt998244353;

pub mod mod_int;
pub mod ntt;

/// 音声データのベクトル.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioVec {
    pub vec: Vec<ModInt998244353>,
}

impl Index<usize> for AudioVec {
    type Output = ModInt998244353;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}
impl IndexMut<usize> for AudioVec {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl AudioVec {
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        if self.vec.len() < other.vec.len() {
            return other.add(self);
        }
        let mut cloned = self.clone();
        cloned.add_assign(other);
        cloned
    }

    #[inline]
    pub fn add_assign(&mut self, other: &Self) {
        for (write, to_add) in self.vec.iter_mut().zip(other.vec.iter()) {
            *write += *to_add;
        }
    }

    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        if self.vec.len() < other.vec.len() {
            return other.add(self);
        }
        let mut cloned = self.clone();
        cloned.sub_assign(other);
        cloned
    }

    #[inline]
    pub fn sub_assign(&mut self, other: &Self) {
        for (write, to_add) in self.vec.iter_mut().zip(other.vec.iter()) {
            *write -= *to_add;
        }
    }

    #[inline]
    pub fn squared(&self) -> impl Iterator<Item = ModInt998244353> + '_ {
        self.vec.iter().map(|&a| a * a)
    }

    #[inline]
    pub fn squared_norm(&self) -> ModInt998244353 {
        self.squared().sum()
    }

    #[inline]
    pub fn delayed(&self, time: isize) -> Self {
        if 0 <= time {
            Self {
                vec: self.vec[time as usize..].to_vec(),
            }
        } else {
            let time = (-time) as usize;
            let mut new = vec![Default::default(); time + self.vec.len()];
            new[time..].copy_from_slice(&self.vec);
            Self { vec: new }
        }
    }

    #[inline]
    pub fn flip(&mut self) {
        self.vec.reverse();
    }

    #[inline]
    pub fn clip(&mut self) {
        for elem in self.vec.iter_mut() {
            *elem = (*elem).min(ModInt998244353::new(u16::MAX as u32));
        }
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.vec.resize(len, Default::default());
    }

    #[inline]
    pub fn convolution(&self, other: &Self, ntt: &ntt::Ntt) -> Vec<ModInt998244353> {
        if self.vec.is_empty() && other.vec.is_empty() {
            return vec![];
        }

        let len = self.vec.len() + other.vec.len() - 1;
        if self.vec.len().min(other.vec.len()) <= 40 {
            // too tiny vectors
            let mut res = vec![ModInt998244353::default(); len];
            for (i, &left) in self.vec.iter().enumerate() {
                for (j, &right) in other.vec.iter().enumerate() {
                    res[i + j] += left * right;
                }
            }
            return res;
        }

        let buf_len = len.next_power_of_two();
        let mut buf1 = self.clone();
        let mut buf2 = other.clone();
        buf1.resize(buf_len);
        buf2.resize(buf_len);

        // 畳み込み計算は数論変換してから乗算して逆数論変換する
        ntt.transform(&mut buf1.vec);
        ntt.transform(&mut buf2.vec);
        for (elem1, elem2) in buf1.vec.iter_mut().zip(buf2.vec.iter_mut()) {
            *elem1 *= *elem2;
        }
        ntt.inverse_transform(&mut buf1.vec);

        // 数論変換で変化した定数倍を戻す
        let inv = ModInt998244353::new(buf_len as u32).inv();
        for elem in buf1.vec.iter_mut() {
            *elem *= inv;
        }
        buf1.vec
    }
}

impl AudioVec {
    #[inline]
    pub fn from_pcm(path: &[i16]) -> Self {
        let emphasized = path
            .iter()
            // 折り返し
            .map(|&x| {
                if 0 <= x {
                    x as i64
                } else {
                    (x as i64) + ModInt998244353::N as i64
                }
            })
            .map(|x| x.try_into().unwrap_or_else(|_| panic!("{x}")))
            .map(ModInt998244353::new);

        Self {
            vec: emphasized.collect(),
        }
    }
}

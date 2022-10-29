use std::ops::{Index, IndexMut};

use wide::{u32x4, u32x8};

use self::mod_int::ModInt998244353;

pub mod mod_int;
pub mod ntt;

/// 音声データのベクトル.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct AudioVec {
    vec: Vec<ModInt998244353>,
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
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ModInt998244353> {
        self.vec.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ModInt998244353> {
        self.vec.iter_mut()
    }

    #[inline]
    pub fn get_u32x4(&self, idx: usize) -> u32x4 {
        u32x4::new([
            self.vec[idx].as_u32(),
            self.vec[idx + 1].as_u32(),
            self.vec[idx + 2].as_u32(),
            self.vec[idx + 3].as_u32(),
        ])
    }

    #[inline]
    pub fn set_u32x4(&mut self, idx: usize, val: u32x4) {
        let arr = val.to_array().map(ModInt998244353::new);
        self.vec[idx..idx + 4].copy_from_slice(&arr)
    }

    #[inline]
    pub fn get_u32x8(&self, idx: usize) -> u32x8 {
        u32x8::new([
            self.vec[idx].as_u32(),
            self.vec[idx + 1].as_u32(),
            self.vec[idx + 2].as_u32(),
            self.vec[idx + 3].as_u32(),
            self.vec[idx + 4].as_u32(),
            self.vec[idx + 5].as_u32(),
            self.vec[idx + 6].as_u32(),
            self.vec[idx + 7].as_u32(),
        ])
    }

    #[inline]
    pub fn set_u32x8(&mut self, idx: usize, val: u32x8) {
        let arr = val.to_array().map(ModInt998244353::new);
        self.vec[idx..idx + 8].copy_from_slice(&arr)
    }
}

impl AudioVec {
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        if self.len() < other.len() {
            return other.add(self);
        }
        let mut cloned = self.clone();
        cloned.add_assign(other);
        cloned
    }

    #[inline]
    pub fn add_assign(&mut self, other: &Self) {
        for (write, to_add) in self.iter_mut().zip(other.iter()) {
            *write += *to_add;
        }
    }

    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        if self.len() < other.len() {
            return other.add(self);
        }
        let mut cloned = self.clone();
        cloned.sub_assign(other);
        cloned
    }

    #[inline]
    pub fn sub_assign(&mut self, other: &Self) {
        for (write, to_add) in self.iter_mut().zip(other.iter()) {
            *write -= *to_add;
        }
    }

    #[inline]
    pub fn squared(&self) -> impl Iterator<Item = ModInt998244353> + '_ {
        self.iter().map(|&a| a * a)
    }

    #[inline]
    pub fn inner_cross(&self, other: &Self) -> ModInt998244353 {
        self.iter().zip(other.iter()).map(|(&a, &b)| a * b).sum()
    }

    #[inline]
    pub fn squared_norm(&self) -> ModInt998244353 {
        self.squared().sum()
    }

    #[inline]
    pub fn delayed(&self, time: isize) -> Self {
        if 0 <= time {
            let time = time as usize;
            let mut new = vec![Default::default(); time + self.vec.len()];
            new[time..].copy_from_slice(&self.vec);
            Self { vec: new }
        } else {
            Self {
                vec: self.vec[(-time) as usize..].to_vec(),
            }
        }
    }

    #[inline]
    pub fn flip(&mut self) {
        self.vec.reverse();
    }

    #[inline]
    pub fn clip(&mut self) {
        for elem in self.iter_mut() {
            *elem = ModInt998244353::new(
                (elem.as_u32() as i32).clamp(i16::MIN as i32, i16::MAX as i32) as u32,
            )
        }
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.vec.resize(len, Default::default());
    }

    #[inline]
    pub fn convolution(&self, other: &Self, ntt: &ntt::Ntt) -> Self {
        if self.is_empty() && other.is_empty() {
            return Self::default();
        }

        let len = self.len() + other.len() - 1;
        if self.len().min(other.len()) <= 40 {
            // too tiny vectors
            let mut res = vec![ModInt998244353::default(); len];
            for (i, &left) in self.iter().enumerate() {
                for (j, &right) in other.iter().enumerate() {
                    res[i + j] += left * right;
                }
            }
            return Self { vec: res };
        }

        let buf_len = len.next_power_of_two();
        let mut buf1 = self.clone();
        let mut buf2 = other.clone();
        buf1.resize(buf_len);
        buf2.resize(buf_len);

        // 畳み込み計算は数論変換してから乗算して逆数論変換する
        ntt.transform(&mut buf1);
        ntt.transform(&mut buf2);
        for (elem1, elem2) in buf1.iter_mut().zip(buf2.iter_mut()) {
            *elem1 *= *elem2;
        }
        ntt.inverse_transform(&mut buf1);

        // 数論変換で変化した定数倍を戻す
        let inv = ModInt998244353::new(buf_len as u32).inv();
        for elem in buf1.iter_mut() {
            *elem *= inv;
        }
        buf1
    }
}

impl AudioVec {
    #[inline]
    pub fn from_pcm(path: &[i16]) -> Self {
        // プリエンファシス
        let emphasized = path.windows(2).map(|win| {
            ModInt998244353::new(
                ((win[0] as i32 + 40 * (win[1] as i32 - win[0] as i32)) + i16::MAX as i32) as u32,
            )
        });

        Self {
            vec: emphasized.collect(),
        }
    }
}

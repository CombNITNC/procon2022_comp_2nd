use self::mod_int::ModInt998244353;

mod mod_int;
mod ntt;

/// 音声データのベクトル.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct AudioVec {
    /// 最初の時刻が一番最後の要素になるように格納される.
    vec: Vec<ModInt998244353>,
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
    pub fn inner_cross(&self, other: &Self) -> ModInt998244353 {
        self.iter().zip(other.iter()).map(|(&a, &b)| a * b).sum()
    }

    #[inline]
    pub fn squared_norm(&self) -> ModInt998244353 {
        self.inner_cross(self)
    }

    #[inline]
    pub fn delay(&mut self, time: usize) {
        self.vec.append(&mut vec![Default::default(); time]);
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
    pub fn convolution(&self, other: &Self) -> Self {
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
        ntt::ntt(&mut buf1.vec);
        ntt::ntt(&mut buf2.vec);
        for (elem1, elem2) in buf1.iter_mut().zip(buf2.iter_mut()) {
            *elem1 *= *elem2;
        }
        ntt::inverse_ntt(&mut buf1.vec);

        // 数論変換で変化した定数倍を戻す
        let inv = ModInt998244353::new(buf_len as u32).inv();
        for elem in buf1.iter_mut() {
            *elem *= inv;
        }
        buf1
    }
}

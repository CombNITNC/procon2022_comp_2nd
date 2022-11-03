use itertools::izip;
use num::traits::Pow;

use super::mod_int::ModInt998244353;

#[cfg(test)]
mod tests;

/// 変換可能な成分の最高次数.
const LEVEL: usize = (ModInt998244353::N - 1).trailing_zeros() as usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ntt {
    primitive_root: ModInt998244353,
    d_w: [ModInt998244353; LEVEL],
    d_inv_w: [ModInt998244353; LEVEL],
}

impl Ntt {
    pub fn new() -> Self {
        let modulo = ModInt998244353::N;
        let primitive_root = primitive_root(modulo);

        let mut w = [ModInt998244353::default(); LEVEL];
        let mut d_w = [ModInt998244353::default(); LEVEL];
        let mut inv_w = [ModInt998244353::default(); LEVEL];
        let mut d_inv_w = [ModInt998244353::default(); LEVEL];

        w[LEVEL - 1] = primitive_root.pow((modulo - 1) / (1 << LEVEL));
        inv_w[LEVEL - 1] = w[LEVEL - 1].inv();

        for i in (0..LEVEL - 1).rev() {
            w[i] = w[i + 1] * w[i + 1];
            inv_w[i] = inv_w[i + 1] * inv_w[i + 1];
        }

        d_w[0] = w[1] * w[1];
        d_inv_w[0] = d_w[0];
        d_w[1] = w[1];
        d_inv_w[1] = inv_w[1];
        d_w[2] = w[2];
        d_inv_w[2] = w[2];

        for i in 3..LEVEL {
            d_w[i] = d_w[i - 1] * inv_w[i - 2] * w[i];
            d_inv_w[i] = d_inv_w[i - 1] * w[i - 2] * inv_w[i];
        }

        Self {
            primitive_root,
            d_w,
            d_inv_w,
        }
    }

    pub fn transform(&self, vec: &mut [ModInt998244353]) {
        if vec.is_empty() {
            return;
        }
        let k = vec.len().trailing_zeros();
        if k == 1 {
            let a = vec[1];
            vec[1] = vec[0] - vec[1];
            vec[0] += a;
            return;
        }

        if k % 2 != 0 {
            let v = 1 << (k - 1);
            let (left, right) = vec.split_at_mut(v);
            for (l, r) in left.iter_mut().zip(right) {
                let at_l = *l;
                let at_r = *r;
                *l = at_l + at_r;
                *r = at_l - at_r;
            }
        }

        let mut v = 1 << (k - 2 - (k % 2));
        let one = ModInt998244353::new(1);
        let im = self.d_w[1];

        while v != 0 {
            let mut xx = one;
            for (i, chunk) in vec.chunks_exact_mut(4 * v).enumerate() {
                let ww = xx * xx;
                let wx = ww * xx;
                {
                    let (chunk0, chunk123) = chunk.split_at_mut(v);
                    let (chunk1, chunk23) = chunk123.split_at_mut(v);
                    let (chunk2, chunk3) = chunk23.split_at_mut(v);
                    for (j0, j1, j2, j3) in izip!(chunk0, chunk1, chunk2, chunk3) {
                        let t0 = *j0;
                        let t1 = *j1 * xx;
                        let t2 = *j2 * ww;
                        let t3 = *j3 * wx;
                        let t0p2 = t0 + t2;
                        let t1p3 = t1 + t3;
                        let t0m2 = t0 - t2;
                        let t1m3 = (t1 - t3) * im;
                        *j0 = t0p2 + t1p3;
                        *j1 = t0p2 - t1p3;
                        *j2 = t0m2 + t1m3;
                        *j3 = t0m2 - t1m3;
                    }
                }
                xx *= self.d_w[((i + 1) * 4).trailing_zeros() as usize];
            }
            v >>= 2;
        }
    }

    pub fn inverse_transform(&self, vec: &mut [ModInt998244353]) {
        if vec.is_empty() {
            return;
        }

        let k = vec.len().trailing_zeros();
        if k == 1 {
            let a1 = vec[1];
            vec[1] = vec[0] - vec[1];
            vec[0] += a1;
            return;
        }

        let mut u = 1 << (k - 2);
        let mut v = 1;
        let one = ModInt998244353::new(1);
        let im = self.d_inv_w[1];

        while u != 0 {
            let mut xx = one;
            for (i, chunk) in vec.chunks_exact_mut(4 * v).enumerate() {
                let ww = xx * xx;
                let yy = xx * im;
                let (chunk0, chunk123) = chunk.split_at_mut(v);
                let (chunk1, chunk23) = chunk123.split_at_mut(v);
                let (chunk2, chunk3) = chunk23.split_at_mut(v);
                for (j0, j1, j2, j3) in izip!(chunk0, chunk1, chunk2, chunk3) {
                    let t0 = *j0;
                    let t1 = *j1;
                    let t2 = *j2;
                    let t3 = *j3;
                    let t0p1 = t0 + t1;
                    let t2p3 = t2 + t3;
                    let t0m1 = (t0 - t1) * xx;
                    let t2m3 = (t2 - t3) * yy;
                    *j0 = t0p1 + t2p3;
                    *j1 = t0m1 + t2m3;
                    *j2 = (t0p1 - t2p3) * ww;
                    *j3 = (t0m1 - t2m3) * ww;
                }
                xx *= self.d_inv_w[((i + 1) * 4).trailing_zeros() as usize];
            }
            u >>= 2;
            v <<= 2;
        }

        if k % 2 != 0 {
            let u = 1 << (k - 1);
            let (left, right) = vec.split_at_mut(u);
            for (l, r) in left.iter_mut().zip(right) {
                let at_l = *l;
                let at_r = *r;
                *l = at_l + at_r;
                *r = at_l - at_r;
            }
        }
    }
}

impl Default for Ntt {
    fn default() -> Self {
        Self::new()
    }
}

fn primitive_root(modulo: u32) -> ModInt998244353 {
    if modulo == 2 {
        return ModInt998244353::new(1);
    }

    let mut divisors = vec![];
    let mut m = modulo - 1;
    for i in 2.. {
        if m < i * i {
            break;
        }
        if m % i == 0 {
            divisors.push(i as u64);
            while m % i == 0 {
                m /= i;
            }
        }
    }
    if m != 1 {
        divisors.push(m as u64);
    }

    'find: for primitive_root in 2.. {
        for divisor in &divisors {
            let mut a: u64 = primitive_root;
            let mut b: u64 = (modulo as u64 - 1) / divisor;
            let mut r: u64 = 1;
            while b != 0 {
                if b % 2 != 0 {
                    r *= a;
                    r %= modulo as u64;
                }
                a *= a;
                a %= modulo as u64;
                b /= 2;
            }
            if r == 1 {
                continue 'find;
            }
        }
        return ModInt998244353::new(primitive_root as u32);
    }
    unreachable!()
}

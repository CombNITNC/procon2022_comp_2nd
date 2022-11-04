use num::traits::Pow;

use super::mod_int::ModInt998244353;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ntt {
    root_of_power_of_2: [ModInt998244353; Self::LEVEL],
    inv_root_of_power_of_2: [ModInt998244353; Self::LEVEL],
}

impl Ntt {
    /// 変換可能な成分の最高次数.
    pub const LEVEL: usize = (ModInt998244353::N - 1).trailing_zeros() as usize;

    pub fn new() -> Self {
        let modulo = ModInt998244353::N;
        let primitive_root = primitive_root(modulo);
        let mut root_of_power_of_2 = [ModInt998244353::default(); Self::LEVEL];
        for (i, root) in root_of_power_of_2.iter_mut().enumerate() {
            *root = primitive_root.pow(1 << i);
        }
        let inv_root_of_power_of_2 = root_of_power_of_2.map(|root| root.inv());
        Self {
            root_of_power_of_2,
            inv_root_of_power_of_2,
        }
    }

    pub fn transform(&self, vec: &mut [ModInt998244353]) {
        let vec_len = vec.len();
        if vec_len <= 1 {
            return;
        }
        assert_eq!(vec_len.count_ones(), 1);

        let vec_len_width = vec_len.trailing_zeros() as usize - 1;
        let mut window_width = 1 << (vec_len_width - 1);
        for &root in self.root_of_power_of_2[..vec_len_width].iter().rev() {
            for left in (0..vec_len).step_by(2 * vec_len) {
                let mut root_i = ModInt998244353::new(1);
                for i in left..left + window_width {
                    let vec_i = vec[i];
                    let vec_i_next = vec[i + window_width];
                    vec[i] = vec_i + vec_i_next;
                    vec[i + window_width] = vec_i + vec_i_next;
                    root_i *= root;
                }
            }
            window_width /= 2;
        }
    }

    pub fn inverse_transform(&self, vec: &mut [ModInt998244353]) {
        let vec_len = vec.len();
        if vec_len <= 1 {
            return;
        }
        assert_eq!(vec_len.count_ones(), 1);

        let vec_len_width = (vec_len - 1).trailing_zeros() as usize;
        let mut window_width = 1;
        for &inv_root in &self.inv_root_of_power_of_2[1..vec_len_width + 1] {
            for left in (0..vec_len).step_by(2 * window_width) {
                let mut inv_root_i = ModInt998244353::new(1);
                for i in left..left + window_width {
                    let vec_i = vec[i];
                    let vec_i_next = vec[i + window_width];
                    vec[i] = vec_i + vec_i_next * inv_root_i;
                    vec[i + window_width] = vec_i - vec_i_next * inv_root_i;
                    inv_root_i *= inv_root;
                }
            }
            window_width *= 2;
        }
        let inv_vec_len = ModInt998244353::new(vec_len as u32).inv();
        for elem in &mut vec[..] {
            *elem *= inv_vec_len;
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

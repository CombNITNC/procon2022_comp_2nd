use num::traits::Pow;

use super::mod_int::ModInt;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ntt<const MOD: u32> {
    root_of_power_of_2: Vec<ModInt<MOD>>,
    inv_root_of_power_of_2: Vec<ModInt<MOD>>,
}

impl<const MOD: u32> Ntt<MOD> {
    /// 変換可能な成分の最高次数.
    pub const LEVEL: usize = (MOD - 1).trailing_zeros() as usize;

    pub fn new() -> Self {
        let modulo = MOD;
        let primitive_root = primitive_root(modulo);
        let root_of_power_of_2: Vec<_> = (0..Self::LEVEL)
            .map(|i| primitive_root.pow(1 << i))
            .collect();
        let inv_root_of_power_of_2 = root_of_power_of_2
            .iter()
            .copied()
            .map(|root| root.inv())
            .collect();
        Self {
            root_of_power_of_2,
            inv_root_of_power_of_2,
        }
    }

    pub fn transform(&self, vec: &mut [ModInt<MOD>]) {
        let vec_len = vec.len();
        if vec_len <= 1 {
            return;
        }
        assert_eq!(vec_len.count_ones(), 1);

        let vec_len_width = vec_len.trailing_zeros() as usize - 1;
        let mut window_width = 1 << (vec_len_width - 1);
        for &root in self.root_of_power_of_2[..vec_len_width].iter().rev() {
            for left in (0..vec_len).step_by(2 * window_width) {
                let mut root_i = ModInt::new(1);
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

    pub fn inverse_transform(&self, vec: &mut [ModInt<MOD>]) {
        let vec_len = vec.len();
        if vec_len <= 1 {
            return;
        }
        assert_eq!(vec_len.count_ones(), 1);

        let vec_len_width = (vec_len - 1).trailing_zeros() as usize;
        let mut window_width = 1;
        for &inv_root in &self.inv_root_of_power_of_2[1..vec_len_width + 1] {
            for left in (0..vec_len).step_by(2 * window_width) {
                let mut inv_root_i = ModInt::new(1);
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
        let inv_vec_len = ModInt::new(vec_len as u64).inv();
        for elem in &mut vec[..] {
            *elem *= inv_vec_len;
        }
    }
}

impl<const MOD: u32> Default for Ntt<MOD> {
    fn default() -> Self {
        Self::new()
    }
}

fn primitive_root<const MOD: u32>(modulo: u32) -> ModInt<MOD> {
    if modulo == 2 {
        return ModInt::new(1);
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
        return ModInt::new(primitive_root);
    }
    unreachable!()
}

use wide::{u32x4, u32x8};

use super::{mod_int::ModInt998244353, AudioVec};

mod montgomery;

/// 変換可能な成分の最高次数.
const LEVEL: usize = 998244352usize.trailing_zeros() as usize;

pub struct Ntt {
    primitive_root: ModInt998244353,
    d_w: [ModInt998244353; LEVEL],
    d_inv_w: [ModInt998244353; LEVEL],
}

impl Ntt {
    pub fn new() -> Self {
        let modulo = ModInt998244353::modulo();
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
            d_w[i] = d_w[i - 1] * d_inv_w[i - 2] * w[i];
            d_inv_w[i] = d_inv_w[i - 1] * w[i - 2] * inv_w[i];
        }

        Self {
            primitive_root,
            d_w,
            d_inv_w,
        }
    }

    pub fn transform(&self, vec: &mut AudioVec) {
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
            if v < 8 {
                for j in 0..v {
                    let jv = vec[j + v];
                    vec[j + v] = vec[j] - jv;
                    vec[j] += jv;
                }
            } else {
                let m0 = u32x8::default();
                let m2 = u32x8::splat(2 * ModInt998244353::modulo());
                for j0 in (0..v).step_by(8) {
                    let j1 = j0 + v;
                    let t0 = vec.get_u32x8(j0);
                    let t1 = vec.get_u32x8(j1);
                    let naj = montgomery::add_u32x8(t0, t1, m2, m0);
                    let naj_v = montgomery::sub_u32x8(t0, t1, m2, m0);
                    vec.set_u32x8(j0, naj);
                    vec.set_u32x8(j1, naj_v);
                }
            }
        }

        let mut u = 1 << (2 + (k % 2));
        let mut v = 1 << (k - 2 - (k % 2));
        let one = ModInt998244353::new(1);
        let im = self.d_w[1];

        while v != 0 {
            if v == 1 {
                let mut xx = one;
                for jh in (0..u).step_by(4) {
                    let ww = xx * xx;
                    let wx = ww * xx;
                    let t0 = vec[jh];
                    let t1 = vec[jh + 1] * xx;
                    let t2 = vec[jh + 2] * ww;
                    let t3 = vec[jh + 3] * wx;
                    let t0p2 = t0 + t2;
                    let t1p3 = t1 + t3;
                    let t0m2 = t0 - t2;
                    let t1m3 = (t1 - t3) * im;
                    vec[jh] = t0p2 + t1p3;
                    vec[jh + 1] = t0p2 - t1p3;
                    vec[jh + 2] = t0m2 + t1m3;
                    vec[jh + 3] = t0m2 - t1m3;
                    xx *= self.d_w[jh.trailing_zeros() as usize];
                }
            } else if v == 4 {
                let mut xx = one;
                for jh in (0..u).step_by(4) {
                    let m0 = u32x4::default();
                    let m1 = u32x4::splat(ModInt998244353::modulo());
                    let m2 = u32x4::splat(2 * ModInt998244353::modulo());
                    let inv_mod = u32x4::splat(ModInt998244353::modulo_inv());
                    let im = u32x4::splat(im.as_u32());

                    if jh == 0 {
                        for j0 in (0..v).step_by(4) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;
                            let t0 = vec.get_u32x4(j0);
                            let t1 = vec.get_u32x4(j1);
                            let t2 = vec.get_u32x4(j2);
                            let t3 = vec.get_u32x4(j3);
                            let t0p2 = montgomery::add_u32x4(t0, t2, m2, m0);
                            let t1p3 = montgomery::add_u32x4(t1, t3, m2, m0);
                            let t0m2 = montgomery::sub_u32x4(t0, t2, m2, m0);
                            let t1m3 = montgomery::mul_u32x4(
                                montgomery::sub_u32x4(t1, t3, m2, m0),
                                im,
                                inv_mod,
                                m1,
                            );
                            vec.set_u32x4(j0, montgomery::add_u32x4(t0p2, t1p3, m2, m0));
                            vec.set_u32x4(j1, montgomery::sub_u32x4(t0p2, t1p3, m2, m0));
                            vec.set_u32x4(j2, montgomery::add_u32x4(t0m2, t1m3, m2, m0));
                            vec.set_u32x4(j3, montgomery::sub_u32x4(t0m2, t1m3, m2, m0));
                        }
                    } else {
                        let ww = xx * xx;
                        let wx = ww * xx;
                        let ww = u32x4::splat(ww.as_u32());
                        let wx = u32x4::splat(wx.as_u32());
                        let xx = u32x4::splat(xx.as_u32());

                        for j0 in (jh * v..(jh + 1) * v).step_by(4) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x4(j0);
                            let t1 = vec.get_u32x4(j1);
                            let t2 = vec.get_u32x4(j2);
                            let t3 = vec.get_u32x4(j3);
                            let mt1 = montgomery::mul_u32x4(t1, xx, inv_mod, m1);
                            let mt2 = montgomery::mul_u32x4(t2, ww, inv_mod, m1);
                            let mt3 = montgomery::mul_u32x4(t3, wx, inv_mod, m1);
                            let t0p2 = montgomery::add_u32x4(t0, mt2, m2, m0);
                            let t1p3 = montgomery::add_u32x4(mt1, mt3, m2, m0);
                            let t0m2 = montgomery::add_u32x4(t0, mt2, m2, m0);
                            let t1m3 = montgomery::mul_u32x4(
                                montgomery::sub_u32x4(mt1, mt3, m2, m0),
                                im,
                                inv_mod,
                                m1,
                            );
                            vec.set_u32x4(j0, montgomery::add_u32x4(t0p2, t1p3, m2, m0));
                            vec.set_u32x4(j1, montgomery::sub_u32x4(t0p2, t1p3, m2, m0));
                            vec.set_u32x4(j2, montgomery::add_u32x4(t0m2, t1m3, m2, m0));
                            vec.set_u32x4(j2, montgomery::sub_u32x4(t0m2, t1m3, m2, m0));
                        }
                    }
                    xx *= self.d_w[(jh + 4).trailing_zeros() as usize];
                }
            } else {
                let m0 = u32x8::default();
                let m1 = u32x8::splat(ModInt998244353::modulo());
                let m2 = u32x8::splat(2 * ModInt998244353::modulo());
                let inv_mod = u32x8::splat(ModInt998244353::modulo_inv());
                let im = u32x8::splat(im.as_u32());

                let mut xx = one;
                for jh in (0..u).step_by(4) {
                    if jh == 0 {
                        for j0 in (0..v).step_by(8) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x8(j0);
                            let t1 = vec.get_u32x8(j1);
                            let t2 = vec.get_u32x8(j2);
                            let t3 = vec.get_u32x8(j3);
                            let t0p2 = montgomery::add_u32x8(t0, t1, m2, m0);
                            let t1p3 = montgomery::add_u32x8(t1, t3, m2, m0);
                            let t0m2 = montgomery::sub_u32x8(t0, t2, m2, m0);
                            let t1m3 = montgomery::mul_u32x8(
                                montgomery::sub_u32x8(t1, t3, m2, m0),
                                im,
                                inv_mod,
                                m1,
                            );
                            vec.set_u32x8(j0, montgomery::add_u32x8(t0p2, t1p3, m2, m0));
                            vec.set_u32x8(j1, montgomery::sub_u32x8(t0p2, t1p3, m2, m0));
                            vec.set_u32x8(j2, montgomery::add_u32x8(t0m2, t1m3, m2, m0));
                            vec.set_u32x8(j3, montgomery::sub_u32x8(t0m2, t1m3, m2, m0));
                        }
                    } else {
                        let ww = xx * xx;
                        let wx = ww * xx;
                        let ww = u32x8::splat(ww.as_u32());
                        let wx = u32x8::splat(wx.as_u32());
                        let xx = u32x8::splat(xx.as_u32());

                        for j0 in (jh * v..(jh + 1) * v).step_by(8) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x8(j0);
                            let t1 = vec.get_u32x8(j1);
                            let t2 = vec.get_u32x8(j2);
                            let t3 = vec.get_u32x8(j3);
                            let mt1 = montgomery::mul_u32x8(t1, xx, inv_mod, m1);
                            let mt2 = montgomery::mul_u32x8(t2, ww, inv_mod, m1);
                            let mt3 = montgomery::mul_u32x8(t3, wx, inv_mod, m1);
                            let t0p2 = montgomery::add_u32x8(t0, mt2, m2, m0);
                            let t1p3 = montgomery::add_u32x8(mt1, mt3, m2, m0);
                            let t0m2 = montgomery::sub_u32x8(t0, mt2, m2, m0);
                            let t1m3 = montgomery::mul_u32x8(
                                montgomery::sub_u32x8(mt1, mt3, m2, m0),
                                im,
                                inv_mod,
                                m1,
                            );
                            vec.set_u32x8(j0, montgomery::add_u32x8(t0p2, t1p3, m2, m0));
                            vec.set_u32x8(j1, montgomery::sub_u32x8(t0p2, t1p3, m2, m0));
                            vec.set_u32x8(j2, montgomery::add_u32x8(t0m2, t1m3, m2, m0));
                            vec.set_u32x8(j3, montgomery::sub_u32x8(t0m2, t1m3, m2, m0));
                        }
                    }
                    xx *= self.d_w[(jh + 4).trailing_zeros() as usize];
                }
                todo!()
            }

            u <<= 2;
            v >>= 2;
        }
    }

    pub fn inverse_transform(&self, vec: &mut AudioVec) {
        if vec.is_empty() {
            return;
        }

        let k = vec.len().trailing_zeros();
        if k == 1 {
            let a1 = vec[1];
            vec[1] = vec[0] - vec[1];
            vec[0] += a1;
            vec[0] *= ModInt998244353::new(2).inv();
            vec[1] *= ModInt998244353::new(2).inv();
            return;
        }

        let mut u = 1 << (k - 2);
        let mut v = 1;
        let one = ModInt998244353::new(1);
        let im = self.d_inv_w[1];
        while u != 0 {
            if v == 1 {
                let mut xx = one;
                u <<= 2;
                for jh in (0..u).step_by(4) {
                    let ww = xx * xx;
                    let yy = xx * im;
                    let t0 = vec[jh];
                    let t1 = vec[jh + 1];
                    let t2 = vec[jh + 2];
                    let t3 = vec[jh + 3];
                    let t0p1 = t0 + t1;
                    let t2p3 = t2 + t3;
                    let t0m1 = (t0 - t1) * xx;
                    let t2m3 = (t2 - t3) * yy;
                    vec[jh] = t0p1 + t2p3;
                    vec[jh + 1] = t0m1 + t2m3;
                    vec[jh + 2] = (t0p1 - t2p3) * ww;
                    vec[jh + 3] = (t0m1 - t2m3) * ww;
                    xx *= self.d_inv_w[(jh + 4).trailing_zeros() as usize];
                }
            } else if v == 4 {
                let m0 = u32x4::default();
                let m1 = u32x4::splat(ModInt998244353::modulo());
                let m2 = u32x4::splat(2 * ModInt998244353::modulo());
                let inv_mod = u32x4::splat(ModInt998244353::modulo_inv());

                let mut xx = one;
                u <<= 2;
                for jh in (0..u).step_by(4) {
                    if jh == 0 {
                        let im = u32x4::splat(im.as_u32());

                        for j0 in (0..v).step_by(4) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x4(j0);
                            let t1 = vec.get_u32x4(j1);
                            let t2 = vec.get_u32x4(j2);
                            let t3 = vec.get_u32x4(j3);
                            let t0p1 = montgomery::add_u32x4(t0, t1, m2, m0);
                            let t2p3 = montgomery::add_u32x4(t2, t3, m2, m0);
                            let t0m1 = montgomery::sub_u32x4(t0, t1, m2, m0);
                            let t2m3 = montgomery::mul_u32x4(
                                montgomery::sub_u32x4(t2, t3, m2, m0),
                                im,
                                inv_mod,
                                m1,
                            );
                            vec.set_u32x4(j0, montgomery::add_u32x4(t0p1, t2p3, m2, m0));
                            vec.set_u32x4(j1, montgomery::add_u32x4(t0m1, t2m3, m2, m0));
                            vec.set_u32x4(j2, montgomery::sub_u32x4(t0p1, t2p3, m2, m0));
                            vec.set_u32x4(j3, montgomery::sub_u32x4(t0m1, t2m3, m2, m0));
                        }
                    } else {
                        let ww = xx * xx;
                        let yy = xx * im;
                        let ww = u32x4::splat(ww.as_u32());
                        let xx = u32x4::splat(xx.as_u32());
                        let yy = u32x4::splat(yy.as_u32());

                        for j0 in (jh * v..(jh + 1) * v).step_by(4) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x4(j0);
                            let t1 = vec.get_u32x4(j1);
                            let t2 = vec.get_u32x4(j2);
                            let t3 = vec.get_u32x4(j3);
                            let t0p1 = montgomery::add_u32x4(t0, t1, m2, m0);
                            let t2p3 = montgomery::add_u32x4(t2, t3, m2, m0);
                            let t0m1 = montgomery::mul_u32x4(
                                montgomery::sub_u32x4(t0, t1, m2, m0),
                                xx,
                                inv_mod,
                                m1,
                            );
                            let t2m3 = montgomery::mul_u32x4(
                                montgomery::sub_u32x4(t2, t3, m2, m0),
                                yy,
                                inv_mod,
                                m1,
                            );
                            vec.set_u32x4(j0, montgomery::add_u32x4(t0p1, t2p3, m2, m0));
                            vec.set_u32x4(j1, montgomery::add_u32x4(t0m1, t2m3, m2, m0));
                            vec.set_u32x4(
                                j2,
                                montgomery::mul_u32x4(
                                    montgomery::sub_u32x4(t0p1, t2p3, m2, m0),
                                    ww,
                                    inv_mod,
                                    m1,
                                ),
                            );
                            vec.set_u32x4(
                                j3,
                                montgomery::mul_u32x4(
                                    montgomery::sub_u32x4(t0m1, t2m3, m2, m0),
                                    ww,
                                    inv_mod,
                                    m1,
                                ),
                            );
                        }
                    }
                    xx *= self.d_inv_w[(jh + 4).trailing_zeros() as usize];
                }
            } else {
                let m0 = u32x8::default();
                let m1 = u32x8::splat(ModInt998244353::modulo());
                let m2 = u32x8::splat(ModInt998244353::modulo());
                let mod_inv = u32x8::splat(ModInt998244353::modulo_inv());

                let mut xx = one;
                u <<= 2;
                for jh in (0..u).step_by(4) {
                    if jh == 0 {
                        let im = u32x8::splat(im.as_u32());

                        for j0 in (0..v).step_by(8) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x8(j0);
                            let t1 = vec.get_u32x8(j1);
                            let t2 = vec.get_u32x8(j2);
                            let t3 = vec.get_u32x8(j3);
                            let t0p1 = montgomery::add_u32x8(t0, t1, m2, m0);
                            let t2p3 = montgomery::add_u32x8(t2, t3, m2, m0);
                            let t0m1 = montgomery::sub_u32x8(t0, t1, m2, m0);
                            let t2m3 = montgomery::mul_u32x8(
                                montgomery::sub_u32x8(t2, t3, m2, m0),
                                im,
                                mod_inv,
                                m1,
                            );
                            vec.set_u32x8(j0, montgomery::add_u32x8(t0p1, t2p3, m2, m0));
                            vec.set_u32x8(j1, montgomery::add_u32x8(t0m1, t2m3, m2, m0));
                            vec.set_u32x8(j2, montgomery::sub_u32x8(t0p1, t2p3, m2, m0));
                            vec.set_u32x8(j3, montgomery::sub_u32x8(t0m1, t2m3, m2, m0));
                        }
                    } else {
                        let ww = xx * xx;
                        let yy = xx * im;
                        let ww = u32x8::splat(ww.as_u32());
                        let xx = u32x8::splat(xx.as_u32());
                        let yy = u32x8::splat(yy.as_u32());

                        for j0 in (jh * v..(jh + 1) * v).step_by(8) {
                            let j1 = j0 + v;
                            let j2 = j1 + v;
                            let j3 = j2 + v;

                            let t0 = vec.get_u32x8(j0);
                            let t1 = vec.get_u32x8(j1);
                            let t2 = vec.get_u32x8(j2);
                            let t3 = vec.get_u32x8(j3);
                            let t0p1 = montgomery::add_u32x8(t0, t1, m2, m0);
                            let t2p3 = montgomery::add_u32x8(t2, t3, m2, m0);
                            let t0m1 = montgomery::mul_u32x8(
                                montgomery::sub_u32x8(t0, t1, m2, m0),
                                xx,
                                mod_inv,
                                m1,
                            );
                            let t2m3 = montgomery::mul_u32x8(
                                montgomery::sub_u32x8(t2, t3, m2, m0),
                                yy,
                                mod_inv,
                                m1,
                            );
                            vec.set_u32x8(j0, montgomery::add_u32x8(t0p1, t2p3, m2, m0));
                            vec.set_u32x8(j1, montgomery::add_u32x8(t0m1, t2m3, m2, m0));
                            vec.set_u32x8(
                                j2,
                                montgomery::mul_u32x8(
                                    montgomery::sub_u32x8(t0p1, t2p3, m2, m0),
                                    ww,
                                    mod_inv,
                                    m1,
                                ),
                            );
                            vec.set_u32x8(
                                j3,
                                montgomery::mul_u32x8(
                                    montgomery::sub_u32x8(t0m1, t2m3, m2, m0),
                                    ww,
                                    mod_inv,
                                    m1,
                                ),
                            );
                        }
                    }
                    xx *= self.d_inv_w[(jh + 4).trailing_zeros() as usize];
                }
            }
            u >>= 4;
            v <<= 2;
        }

        if k % 2 == 1 {
            v = 1 << (k - 1);
            if v < 8 {
                for j in 0..v {
                    let ajv = vec[j + v];
                    let aj_ajv = vec[j] - vec[j + v];
                    vec[j] += ajv;
                    vec[j + v] = aj_ajv;
                }
            } else {
                let m0 = u32x8::default();
                let m2 = u32x8::splat(2 * ModInt998244353::modulo());

                for j0 in (0..v).step_by(8) {
                    let j1 = j0 + v;

                    let t0 = vec.get_u32x8(j0);
                    let t1 = vec.get_u32x8(j1);
                    let naj = montgomery::add_u32x8(t0, t1, m2, m0);
                    let naj_v = montgomery::sub_u32x8(t0, t1, m2, m0);
                    vec.set_u32x8(j0, naj);
                    vec.set_u32x8(j1, naj_v);
                }
            }
        }
        let inv_len = ModInt998244353::new(vec.len() as u32).inv();
        for val in vec.iter_mut() {
            *val *= inv_len;
        }
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
            divisors.push(i);
            while m % i == 0 {
                m /= i;
            }
        }
    }
    if m != 1 {
        divisors.push(m);
    }

    'find: for primitive_root in 2.. {
        for divisor in &divisors {
            let mut a = primitive_root;
            let mut b = (modulo - 1) / divisor;
            let mut r = 1;
            while b != 0 {
                if b % 2 != 0 {
                    r *= a;
                    r %= modulo;
                }
                a *= a;
                a %= modulo;
                b /= 2;
            }
            if r == 1 {
                continue 'find;
            }
        }
        return ModInt998244353::new(primitive_root);
    }
    unreachable!()
}

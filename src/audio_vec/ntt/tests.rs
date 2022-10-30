//! From: https://judge.yosupo.jp/problem/convolution_mod

use crate::audio_vec::{mod_int::ModInt998244353, AudioVec};

use super::Ntt;

#[test]
fn convolution1() {
    let a: Vec<_> = [1, 2, 3, 4].into_iter().map(ModInt998244353::new).collect();
    let b: Vec<_> = [5, 6, 7, 8, 9]
        .into_iter()
        .map(ModInt998244353::new)
        .collect();

    let a_audio = AudioVec { vec: a };
    let b_audio = AudioVec { vec: b };
    let ntt = Ntt::new();
    let out = a_audio.convolution(&b_audio, &ntt);

    let expected: Vec<_> = [5, 16, 34, 60, 70, 70, 59, 36]
        .into_iter()
        .map(ModInt998244353::new)
        .collect();
    assert_eq!(out, expected);
}

#[test]
fn convolution2() {
    let a: Vec<_> = [10000000].into_iter().map(ModInt998244353::new).collect();
    let b: Vec<_> = [10000000].into_iter().map(ModInt998244353::new).collect();

    let a_audio = AudioVec { vec: a };
    let b_audio = AudioVec { vec: b };
    let ntt = Ntt::new();
    let out = a_audio.convolution(&b_audio, &ntt);

    let expected: Vec<_> = [871938225].into_iter().map(ModInt998244353::new).collect();
    assert_eq!(out, expected);
}

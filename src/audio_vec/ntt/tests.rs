//! From: https://judge.yosupo.jp/problem/convolution_mod

use crate::audio_vec::AudioVec;

use super::Ntt;

#[test]
fn convolution1() {
    let a = [1, 2, 3, 4];
    let b = [5, 6, 7, 8, 9];

    let a_audio = AudioVec::from_raw_slice(&a);
    let b_audio = AudioVec::from_raw_slice(&b);
    let ntt1 = Ntt::new();
    let ntt2 = Ntt::new();
    let out = a_audio.convolution(&b_audio, (&ntt1, &ntt2));

    let expected = vec![5, 16, 34, 60, 70, 70, 59, 36];
    assert_eq!(out, expected);
}

#[test]
fn convolution2() {
    let a = [10000000];
    let b = [10000000];

    let a_audio = AudioVec::from_raw_slice(&a);
    let b_audio = AudioVec::from_raw_slice(&b);
    let ntt1 = Ntt::new();
    let ntt2 = Ntt::new();
    let out = a_audio.convolution(&b_audio, (&ntt1, &ntt2));

    let expected = vec![100000000000000];
    assert_eq!(out, expected);
}

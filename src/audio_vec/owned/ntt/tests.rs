//! From: https://judge.yosupo.jp/problem/convolution_mod

use crate::audio_vec::owned::Owned;

use super::Ntt;

#[test]
fn convolution1() {
    let a = [1, 2, 3, 4];
    let b = [5, 6, 7, 8, 9];

    let a_audio = Owned::from_raw_slice(&a);
    let b_audio = Owned::from_raw_slice(&b);
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

    let a_audio = Owned::from_raw_slice(&a);
    let b_audio = Owned::from_raw_slice(&b);
    let ntt1 = Ntt::new();
    let ntt2 = Ntt::new();
    let out = a_audio.convolution(&b_audio, (&ntt1, &ntt2));

    let expected = vec![100000000000000];
    assert_eq!(out, expected);
}

#[test]
fn convolution3() {
    let a = [
        2452245, 25262467, 245594, 13401, 1341, 1349, 31459, 568249, 13843013, 46585340, 3434, 0,
        0, 0, 24573, 15134, 78943510, 9847, 58183745, 1846, 17043, 710,
    ];
    let b = [
        789708, 780967, 67670, 5656, 12134, 5, 656321, 0, 54580, 13, 2433435, 3823, 35548034,
        23894, 89708, 878659, 867970, 978, 60, 8, 8706850, 348, 69407, 68,
    ];

    let a_audio = Owned::from_raw_slice(&a);
    let b_audio = Owned::from_raw_slice(&b);
    let ntt1 = Ntt::new();
    let ntt2 = Ntt::new();
    let out = a_audio.convolution(&b_audio, (&ntt1, &ntt2));

    assert_eq!(out, ugly_convolution(&a_audio, &b_audio));
}

fn ugly_convolution(a: &Owned, b: &Owned) -> Vec<u64> {
    let len = a.len() + b.len() - 1;
    let mut res = vec![0; len];
    for (i, &left) in a.vec.iter().enumerate() {
        for (j, &right) in b.vec.iter().enumerate() {
            res[i + j] += left.convolution(right);
        }
    }
    res
}

use cast::i64;

use super::{owned::pixel::Pixel, AudioVec};

#[derive(Debug)]
pub struct Add<A, B> {
    pub(super) left: A,
    pub(super) right: B,
}

impl<A: AudioVec, B: AudioVec> AudioVec for Add<A, B> {
    fn get(&self, index: isize) -> Pixel {
        self.left.get(index) + self.right.get(index)
    }
}

#[derive(Debug)]
pub struct Sub<A, B> {
    pub(super) left: A,
    pub(super) right: B,
}

impl<A: AudioVec, B: AudioVec> AudioVec for Sub<A, B> {
    fn get(&self, index: isize) -> Pixel {
        self.left.get(index) - self.right.get(index)
    }
}

#[derive(Debug)]
pub struct Delayed<T> {
    pub(super) vec: T,
    /// 要素にアクセスするときに添字にこの値を足す.
    pub(super) delay: isize,
}

impl<T: AudioVec> AudioVec for Delayed<T> {
    fn get(&self, index: isize) -> Pixel {
        self.vec.get(index + self.delay)
    }
}

#[derive(Debug)]
pub struct Flipped<T> {
    pub(super) vec: T,
}

impl<T: AudioVec> AudioVec for Flipped<T> {
    fn get(&self, index: isize) -> Pixel {
        self.vec.get(-index)
    }
}

#[derive(Debug)]
pub struct Clipped<T> {
    pub(super) vec: T,
}

impl<T: AudioVec> AudioVec for Clipped<T> {
    fn get(&self, index: isize) -> Pixel {
        self.vec.get(index).clamp(
            Pixel::from_signed(i64(i16::MIN)),
            Pixel::from_signed(i64(i16::MAX)),
        )
    }
}

use cast::isize;

use self::owned::{pixel::Pixel, Owned};

pub mod composite;
pub mod owned;

pub trait AudioVec {
    fn get(&self, index: isize) -> Option<Pixel>;

    fn add<B>(self, other: B) -> composite::Add<Self, B>
    where
        Self: Sized,
        B: AudioVec,
    {
        composite::Add {
            left: self,
            right: other,
        }
    }

    fn sub<B>(self, other: B) -> composite::Sub<Self, B>
    where
        Self: Sized,
        B: AudioVec,
    {
        composite::Sub {
            left: self,
            right: other,
        }
    }

    fn delay(self, delay: isize) -> composite::Delayed<Self>
    where
        Self: Sized,
    {
        composite::Delayed { vec: self, delay }
    }

    fn flip(self) -> composite::Flipped<Self>
    where
        Self: Sized,
    {
        composite::Flipped { vec: self }
    }

    fn clip(self) -> composite::Clipped<Self>
    where
        Self: Sized,
    {
        composite::Clipped { vec: self }
    }

    fn to_owned(&self, len: usize) -> Owned {
        Owned::from_pixels((0..len).flat_map(|index| self.get(isize(index).unwrap())))
    }
}

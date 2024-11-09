#![allow(incomplete_features)]

extern crate alloc;

use alloc::vec::Vec;
use core::simd::{cmp::*, *};

use crate::Error;

pub trait ReprBytes: Sized + PartialEq {
    const N: usize;

    fn from_bytes(input: [u8; Self::N]) -> Self;
    fn as_bytes(&self) -> [u8; Self::N];

    #[inline(always)]
    fn zero() -> Self
    where
        [(); Self::N]:,
    {
        Self::from_bytes([0u8; Self::N])
    }

    #[inline]
    fn equals(&self, other: &Self) -> bool {
        self == other
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8>
    where
        [(); Self::N]:,
    {
        self.as_bytes().to_vec()
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self, Error>
    where
        [(); Self::N]:,
    {
        Ok(Self::from_bytes(input.try_into()?))
    }
}

macro_rules! impl_repr_array {
    ($size:expr) => {
        impl ReprBytes for [u8; $size] {
            const N: usize = $size;

            #[inline(always)]
            fn from_bytes(input: [u8; Self::N]) -> Self {
                input
            }

            #[inline(always)]
            fn as_bytes(&self) -> [u8; Self::N] {
                *self
            }

            #[inline]
            fn equals(&self, other: &Self) -> bool
            where
                [(); Self::N]:,
            {
                const LANES: usize = 16;

                for (a, b) in self.chunks_exact(LANES).zip(other.chunks_exact(LANES)) {
                    let a: Simd<u8, LANES> = Simd::from_slice(a);
                    let b: Simd<u8, LANES> = Simd::from_slice(b);
                    if a.simd_ne(b).any() {
                        return false;
                    }
                }

                true
            }
        }
    };
}

macro_rules! impl_repr_num {
    ($type:ty) => {
        impl ReprBytes for $type {
            const N: usize = core::mem::size_of::<$type>();

            #[inline(always)]
            fn from_bytes(input: [u8; Self::N]) -> Self {
                <$type>::from_le_bytes(input)
            }

            #[inline(always)]
            fn as_bytes(&self) -> [u8; Self::N] {
                self.to_le_bytes()
            }

            #[inline]
            fn equals(&self, other: &Self) -> bool
            where
                [(); Self::N]:,
            {
                self == other
            }
        }
    };
}

impl_repr_array!(1);
impl_repr_array!(2);
impl_repr_array!(4);
impl_repr_array!(8);
impl_repr_array!(16);
impl_repr_array!(32);
impl_repr_array!(64);
impl_repr_array!(128);
impl_repr_array!(256);
impl_repr_array!(512);
impl_repr_array!(1024);
impl_repr_array!(2048);
impl_repr_array!(5192);

impl_repr_num!(u8);
impl_repr_num!(u16);
impl_repr_num!(u32);
impl_repr_num!(u64);
impl_repr_num!(u128);
impl_repr_num!(i8);
impl_repr_num!(i16);
impl_repr_num!(i32);
impl_repr_num!(i64);
impl_repr_num!(i128);

#[cfg(test)]
mod tests {
    extern crate alloc;

    use proptest::{collection::vec, prelude::*};

    use super::*;

    macro_rules! test_repr_array {
        ($size:expr) => {
            paste::paste! {
                #[test_strategy::proptest]
                fn [<test_bytes_roundtrip_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let input: [u8; $size] = input.try_into().unwrap();
                    prop_assert_eq!(<[u8; $size]>::from_bytes(input.as_bytes()), input);
                }
            }
        };
    }

    test_repr_array!(16);
    test_repr_array!(32);
    test_repr_array!(64);
    test_repr_array!(128);
    test_repr_array!(256);
    test_repr_array!(512);
    test_repr_array!(1024);
    test_repr_array!(2048);
}

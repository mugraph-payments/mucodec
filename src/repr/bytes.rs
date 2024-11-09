#![allow(incomplete_features)]

extern crate alloc;

use alloc::vec::Vec;

use crate::Error;

pub trait ReprBytes<const N: usize>: Sized {
    fn from_bytes(input: [u8; N]) -> Self;
    fn as_bytes(&self) -> [u8; N];

    #[inline(always)]
    fn zero() -> Self {
        Self::from_bytes([0u8; N])
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self, Error>
    where
        [(); N]:,
    {
        Ok(Self::from_bytes(input.try_into()?))
    }
}

macro_rules! impl_repr_num {
    ($type:ty, $size:expr) => {
        impl ReprBytes<$size> for $type {
            #[inline(always)]
            fn from_bytes(input: [u8; $size]) -> Self {
                <$type>::from_le_bytes(input)
            }

            #[inline(always)]
            fn as_bytes(&self) -> [u8; $size] {
                self.to_le_bytes()
            }
        }
    };
}

impl_repr_num!(u8, 1);
impl_repr_num!(u16, 2);
impl_repr_num!(u32, 4);
impl_repr_num!(u64, 8);
impl_repr_num!(u128, 16);
impl_repr_num!(i8, 1);
impl_repr_num!(i16, 2);
impl_repr_num!(i32, 4);
impl_repr_num!(i64, 8);
impl_repr_num!(i128, 16);

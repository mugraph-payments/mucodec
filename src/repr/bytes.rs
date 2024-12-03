#![allow(incomplete_features)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;

use crate::Error;

pub trait ReprBytes<const N: usize>: Sized + Debug {
    const SIZE: usize = N;

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
        if input.len() != N {
            return Err(Error::InvalidDataSize {
                expected: N,
                got: input.len(),
            });
        }

        Ok(Self::from_bytes(input.try_into()?))
    }
}

macro_rules! impl_repr_num {
    ($type:ty) => {
        impl ReprBytes<{ core::mem::size_of::<$type>() }> for $type {
            const SIZE: usize = core::mem::size_of::<$type>();

            #[inline(always)]
            fn from_bytes(input: [u8; core::mem::size_of::<$type>()]) -> Self {
                <$type>::from_le_bytes(input)
            }

            #[inline(always)]
            fn as_bytes(&self) -> [u8; core::mem::size_of::<$type>()] {
                self.to_le_bytes()
            }
        }
    };
}

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
impl_repr_num!(usize);
impl_repr_num!(isize);

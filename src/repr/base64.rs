#![allow(incomplete_features)]

extern crate alloc;

use alloc::string::String;

use crate::{Bytes, Error, ReprBytes};

pub trait ReprBase64<const N: usize>: ReprBytes<N> {
    fn to_base64(&self) -> String;
    fn from_base64(input: &str) -> Result<Self, Error>;
}

macro_rules! impl_repr_num {
    ($type:ty, $size:expr) => {
        impl ReprBase64<$size> for $type {
            fn to_base64(&self) -> String {
                Bytes::from_bytes(self.to_le_bytes()).to_base64()
            }

            fn from_base64(input: &str) -> Result<Self, Error> {
                let bytes = Bytes::<$size>::from_base64(input)?;
                Ok(<$type>::from_le_bytes(*bytes))
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

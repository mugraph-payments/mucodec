#![allow(incomplete_features)]

extern crate alloc;

use alloc::string::String;

use crate::{Bytes, Error, ReprBytes};

pub trait ReprBase64<const N: usize>: ReprBytes<N> {
    fn to_base64(&self) -> String;
    fn from_base64(input: &str) -> Result<Self, Error>;
}

macro_rules! impl_repr_num {
    ($type:ty) => {
        impl ReprBase64<{ core::mem::size_of::<$type>() }> for $type {
            fn to_base64(&self) -> String {
                Bytes::from_bytes(self.to_le_bytes()).to_base64()
            }

            fn from_base64(input: &str) -> Result<Self, Error> {
                let bytes = Bytes::<{ core::mem::size_of::<$type>() }>::from_base64(input)?;
                Ok(<$type>::from_le_bytes(*bytes))
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

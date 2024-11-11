#![allow(incomplete_features)]

use alloc::{string::String, vec::Vec};

use crate::{Error, ReprBytes};

pub trait ReprHex<const N: usize>: Sized + ReprBytes<N> {
    // Add this constant - it will always be N * 2 for hex encoding
    const HEX_SIZE: usize = N * 2;

    fn to_hex(&self) -> String;
    fn from_hex(input: &str) -> Result<Self, Error>;
}

macro_rules! impl_repr_num {
    ($type:ty) => {
        impl ReprHex<{ core::mem::size_of::<$type>() }> for $type {
            #[inline]
            fn to_hex(&self) -> String {
                const LOOKUP: [u8; 16] = *b"0123456789abcdef";
                let bytes = self.as_bytes();
                let mut result = Vec::with_capacity(Self::HEX_SIZE);

                for byte in bytes {
                    result.push(LOOKUP[(byte >> 4) as usize]);
                    result.push(LOOKUP[(byte & 0xf) as usize]);
                }

                // Safe because we only used valid ASCII hex digits
                unsafe { String::from_utf8_unchecked(result) }
            }

            #[inline]
            fn from_hex(input: &str) -> Result<Self, Error> {
                if input.len() != Self::HEX_SIZE {
                    return Err(Error::InvalidDataSize {
                        expected: Self::HEX_SIZE,
                        got: input.len(),
                    });
                }

                let input = input.as_bytes();
                let mut bytes = [0u8; { core::mem::size_of::<$type>() }];

                for (i, chunk) in input.chunks_exact(2).enumerate() {
                    let hi = from_hex_digit(chunk[0])?;
                    let lo = from_hex_digit(chunk[1])?;
                    bytes[i] = (hi << 4) | lo;
                }

                Ok(Self::from_bytes(bytes))
            }
        }
    };
}

pub(crate) fn from_hex_digit(digit: u8) -> Result<u8, Error> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(Error::InvalidHexDigit(digit as char)),
    }
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

#![allow(incomplete_features)]

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::simd::{cmp::*, num::*, *};

use crate::{Error, ReprBytes};

pub trait ReprHex: ReprBytes {
    fn to_hex(&self) -> String;

    fn from_hex(input: &str) -> Result<Self, Error>
    where
        [(); Self::N]:;
}

fn from_hex_digit(digit: u8) -> Result<u8, Error> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(Error::InvalidData(format!(
            "Invalid hex digit: {}",
            digit as char
        ))),
    }
}

macro_rules! impl_repr_array {
    ($size:expr) => {
        impl ReprHex for [u8; $size] {
            #[inline]
            fn to_hex(&self) -> String
            where
                [(); Self::N]:,
            {
                const LOOKUP: [u8; 16] = *b"0123456789abcdef";
                let mut result = Vec::with_capacity($size * 2);

                // Process full chunks of 16 bytes
                for chunk in self.chunks_exact(16) {
                    let v: Simd<u8, 16> = Simd::from_slice(chunk);

                    // Extract high and low nibbles
                    let hi = v >> 4;
                    let lo = v & Simd::splat(0x0f);

                    // Convert to usize for indexing
                    let hi_indices = hi.cast::<usize>();
                    let lo_indices = lo.cast::<usize>();

                    // Lookup hex digits
                    let hi_chars = Simd::gather_or_default(&LOOKUP, hi_indices);
                    let lo_chars = Simd::gather_or_default(&LOOKUP, lo_indices);

                    // Interleave high and low digits
                    for i in 0..16 {
                        result.push(hi_chars.as_array()[i]);
                        result.push(lo_chars.as_array()[i]);
                    }
                }

                // Handle remaining bytes
                let remainder = self.chunks_exact(16).remainder();
                for &byte in remainder {
                    result.push(LOOKUP[(byte >> 4) as usize]);
                    result.push(LOOKUP[(byte & 0xf) as usize]);
                }

                // Safe because we only used valid ASCII hex digits
                unsafe { String::from_utf8_unchecked(result) }
            }

            #[inline]
            fn from_hex(input: &str) -> Result<Self, Error>
            where
                [(); Self::N]:,
            {
                if input.len() != Self::N * 2 {
                    return Err(Error::InvalidData(format!(
                        "Invalid hex string length: expected {}, got {}",
                        Self::N * 2,
                        input.len()
                    )));
                }

                let input = input.as_bytes();
                let mut result = [0u8; $size];

                // Process 32 hex chars (16 bytes) at a time using SIMD
                for (chunk_idx, chunk) in input.chunks_exact(32).enumerate() {
                    let v: Simd<u8, 32> = Simd::from_slice(chunk);

                    // Check which chars are digits (0-9) vs letters (a-f)
                    let is_digit = v.simd_ge(Simd::splat(b'0')) & v.simd_le(Simd::splat(b'9'));
                    let is_alpha = v.simd_ge(Simd::splat(b'a')) & v.simd_le(Simd::splat(b'f'));

                    // Validate that all input chars were valid hex digits
                    if !(is_digit | is_alpha).all() {
                        return Err(Error::InvalidData("Invalid hex digit".to_string()));
                    }

                    // Convert ASCII hex to values
                    let values = is_digit.select(
                        v - Simd::splat(b'0'),
                        v - Simd::splat(b'a') + Simd::splat(10),
                    );

                    // Split into high and low nibbles
                    let values_arr = values.to_array();
                    let out_idx = chunk_idx * 16;

                    // Process pairs of hex digits
                    for i in 0..16 {
                        let hi = values_arr[i * 2]; // First digit of pair
                        let lo = values_arr[i * 2 + 1]; // Second digit of pair
                        result[out_idx + i] = (hi << 4) | lo;
                    }
                }

                // Handle remaining bytes with standard method
                let remainder = input.chunks_exact(32).remainder();
                if !remainder.is_empty() {
                    let out_idx = (input.len() - remainder.len()) / 2;

                    for (i, chunk) in remainder.chunks_exact(2).enumerate() {
                        let hi = from_hex_digit(chunk[0])?;
                        let lo = from_hex_digit(chunk[1])?;
                        result[out_idx + i] = (hi << 4) | lo;
                    }
                }

                Ok(result)
            }
        }
    };
}

macro_rules! impl_repr_num {
    ($type:ty) => {
        impl ReprHex for $type {
            #[inline]
            fn to_hex(&self) -> String {
                const LOOKUP: [u8; 16] = *b"0123456789abcdef";
                let bytes = self.as_bytes();
                let mut result = Vec::with_capacity(Self::N * 2);

                for byte in bytes {
                    result.push(LOOKUP[(byte >> 4) as usize]);
                    result.push(LOOKUP[(byte & 0xf) as usize]);
                }

                // Safe because we only used valid ASCII hex digits
                unsafe { String::from_utf8_unchecked(result) }
            }

            #[inline]
            fn from_hex(input: &str) -> Result<Self, Error> {
                if input.len() != Self::N * 2 {
                    return Err(Error::InvalidData(format!(
                        "Invalid hex string length: expected {}, got {}",
                        Self::N * 2,
                        input.len()
                    )));
                }

                let input = input.as_bytes();
                let mut bytes = [0u8; Self::N];

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
                fn [<test_encoding_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let mut arr = [0u8; $size];
                    arr.copy_from_slice(&input);
                    prop_assert_eq!(arr.to_hex(), hex::encode(arr));
                }

                #[test_strategy::proptest]
                fn [<test_roundtrip_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let input: [u8; $size] = input.try_into().unwrap();
                    prop_assert_eq!(<[u8; $size]>::from_hex(&input.to_hex())?, input);
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

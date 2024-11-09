#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::simd::{cmp::*, num::*, *};

mod error;
pub use error::Error;

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

pub trait ReprHex: ReprBytes {
    fn to_hex(&self) -> String;

    fn from_hex(input: &str) -> Result<Self, Error>
    where
        [(); Self::N]:;
}

#[inline]
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

pub trait ReprBase64: ReprBytes {
    fn to_base64(&self) -> String;

    fn from_base64(input: &str) -> Result<Self, Error>
    where
        [(); Self::N]:;
}

macro_rules! impl_repr_bytes_array {
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

        impl ReprBase64 for [u8; $size] {
            #[inline]
            fn to_base64(&self) -> String {
                const BASE64_CHARS: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                let mut result = String::with_capacity((Self::N + 2) / 3 * 4);
                let mut output = [0u8; 32]; // Fixed buffer for SIMD output
                let chunks = self.chunks_exact(24);
                let remainder = chunks.remainder();
                let remainder_len = remainder.len();

                // Process full chunks with SIMD
                for chunk in chunks {
                    // Create a padded 32-byte buffer
                    let mut padded = [0u8; 32];
                    padded[..24].copy_from_slice(chunk);

                    let input_simd = Simd::<u8, 32>::from_array(padded);
                    let reshuffled = enc_reshuffle(input_simd);
                    let encoded = enc_translate(reshuffled);

                    // Copy to our fixed output buffer
                    output.copy_from_slice(&encoded.to_array());

                    // Safe because we know the output only contains valid base64 characters
                    unsafe {
                        result.push_str(core::str::from_utf8_unchecked(&output[..32]));
                    }
                }

                // Handle remaining bytes without branching on the outer loop
                if remainder_len > 0 {
                    let mut i = 0;
                    let mut temp = [0u8; 4];

                    while i < remainder_len {
                        let b0 = remainder[i];
                        let b1 = *remainder.get(i + 1).unwrap_or(&0);
                        let b2 = *remainder.get(i + 2).unwrap_or(&0);

                        temp[0] = BASE64_CHARS[(b0 >> 2) as usize];
                        temp[1] = BASE64_CHARS[((b0 & 0x03) << 4 | b1 >> 4) as usize];
                        temp[2] = if i + 1 < remainder_len {
                            BASE64_CHARS[((b1 & 0x0f) << 2 | b2 >> 6) as usize]
                        } else {
                            b'='
                        };
                        temp[3] = if i + 2 < remainder_len {
                            BASE64_CHARS[(b2 & 0x3f) as usize]
                        } else {
                            b'='
                        };

                        // Safe because we know temp only contains valid base64 characters
                        unsafe {
                            result.push_str(core::str::from_utf8_unchecked(&temp));
                        }

                        i += 3;
                    }
                }

                result
            }

            #[inline]
            fn from_base64(input: &str) -> Result<Self, Error>
            where
                [(); Self::N]:,
            {
                // Calculate expected base64 length (including padding)
                let expected_len = (Self::N + 2) / 3 * 4;
                if input.len() != expected_len {
                    return Err(Error::InvalidData(format!(
                        "Invalid base64 string length: expected {}, got {}",
                        expected_len,
                        input.len()
                    )));
                }

                let input = input.as_bytes();
                let mut result = [0u8; $size];
                let mut chunks = input.chunks_exact(32);
                let mut out_idx = 0;

                // Process full chunks with SIMD
                while let Some(chunk) = chunks.next() {
                    if out_idx + 24 > $size {
                        break;
                    }

                    let input_simd = Simd::<u8, 32>::from_slice(chunk);
                    let decoded = dec_translate(input_simd)?;
                    let reshuffled = dec_reshuffle(decoded);

                    // Copy valid bytes to result
                    result[out_idx..out_idx + 24].copy_from_slice(&reshuffled.to_array()[..24]);
                    out_idx += 24;
                }

                // Handle remaining bytes manually
                let remainder = chunks.remainder();
                if !remainder.is_empty() {
                    let mut i = 0;
                    while i < remainder.len() {
                        if out_idx >= $size {
                            break;
                        }

                        let b0 = dec_byte(remainder[i])?;
                        let b1 = dec_byte(remainder[i + 1])?;
                        let b2 = if remainder[i + 2] == b'=' {
                            0
                        } else {
                            dec_byte(remainder[i + 2])?
                        };
                        let b3 = if remainder[i + 3] == b'=' {
                            0
                        } else {
                            dec_byte(remainder[i + 3])?
                        };

                        result[out_idx] = (b0 << 2) | (b1 >> 4);
                        if remainder[i + 2] != b'=' {
                            result[out_idx + 1] = (b1 << 4) | (b2 >> 2);
                        }
                        if remainder[i + 3] != b'=' {
                            result[out_idx + 2] = (b2 << 6) | b3;
                        }

                        i += 4;
                        out_idx += 3;
                    }
                }

                Ok(result)
            }
        }
    };
}

macro_rules! impl_repr_bytes_numeric {
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

impl_repr_bytes_array!(1);
impl_repr_bytes_array!(2);
impl_repr_bytes_array!(4);
impl_repr_bytes_array!(8);
impl_repr_bytes_array!(16);
impl_repr_bytes_array!(32);
impl_repr_bytes_array!(64);
impl_repr_bytes_array!(128);
impl_repr_bytes_array!(256);
impl_repr_bytes_array!(512);
impl_repr_bytes_array!(1024);
impl_repr_bytes_array!(2048);
impl_repr_bytes_array!(5192);

impl_repr_bytes_numeric!(u8);
impl_repr_bytes_numeric!(u16);
impl_repr_bytes_numeric!(u32);
impl_repr_bytes_numeric!(u64);
impl_repr_bytes_numeric!(u128);
impl_repr_bytes_numeric!(i8);
impl_repr_bytes_numeric!(i16);
impl_repr_bytes_numeric!(i32);
impl_repr_bytes_numeric!(i64);
impl_repr_bytes_numeric!(i128);

#[inline(always)]
fn enc_reshuffle(input: Simd<u8, 32>) -> Simd<u8, 32> {
    let mut result = Simd::splat(0u8);

    // Process 3 input bytes into 4 output bytes
    for i in 0..8 {
        // Process 24 bytes in groups of 3
        let base = i * 3;
        let out_base = i * 4;

        if base + 2 < 32 {
            let b0 = input[base];
            let b1 = input[base + 1];
            let b2 = input[base + 2];

            result[out_base] = b0 >> 2;
            result[out_base + 1] = ((b0 & 0x03) << 4) | (b1 >> 4);
            result[out_base + 2] = ((b1 & 0x0f) << 2) | (b2 >> 6);
            result[out_base + 3] = b2 & 0x3f;
        }
    }

    result
}

#[inline(always)]
fn enc_translate(input: Simd<u8, 32>) -> Simd<u8, 32> {
    // Base64 translation table
    const LUT: [u8; 64] = [
        b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
        b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd',
        b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's',
        b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'+', b'/',
    ];

    let mut result = Simd::splat(0u8);
    for i in 0..32 {
        let idx = (input[i] & 0x3f) as usize; // Ensure index is within bounds
        result[i] = LUT[idx];
    }
    result
}

#[inline(always)]
fn dec_translate(input: Simd<u8, 32>) -> Result<Simd<u8, 32>, Error> {
    let mut result = Simd::splat(0u8);

    for i in 0..32 {
        result[i] = match input[i] {
            b'A'..=b'Z' => input[i] - b'A',
            b'a'..=b'z' => input[i] - b'a' + 26,
            b'0'..=b'9' => input[i] - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 0,
            _ => {
                return Err(Error::InvalidData(format!(
                    "Invalid base64 character: {}",
                    input[i] as char
                )))
            }
        };
    }

    Ok(result)
}

#[inline(always)]
fn dec_reshuffle(input: Simd<u8, 32>) -> Simd<u8, 32> {
    let mut result = Simd::splat(0u8);

    for i in 0..8 {
        let base = i * 4;
        let out_base = i * 3;

        if out_base + 2 < 32 {
            let b0 = input[base];
            let b1 = input[base + 1];
            let b2 = input[base + 2];
            let b3 = input[base + 3];

            result[out_base] = (b0 << 2) | (b1 >> 4);
            result[out_base + 1] = (b1 << 4) | (b2 >> 2);
            result[out_base + 2] = (b2 << 6) | b3;
        }
    }

    result
}

#[inline]
fn dec_byte(input: u8) -> Result<u8, Error> {
    match input {
        b'A'..=b'Z' => Ok(input - b'A'),
        b'a'..=b'z' => Ok(input - b'a' + 26),
        b'0'..=b'9' => Ok(input - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(Error::InvalidData(format!(
            "Invalid base64 character: {}",
            input as char
        ))),
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use base64::Engine;
    use proptest::{collection::vec, prelude::*};

    use super::*;

    macro_rules! test_repr_bytes_array {
        ($size:expr) => {
            paste::paste! {
                #[test_strategy::proptest]
                fn [<test_hex_encoding_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let mut arr = [0u8; $size];
                    arr.copy_from_slice(&input);
                    prop_assert_eq!(arr.to_hex(), hex::encode(arr));
                }

                #[test_strategy::proptest]
                fn [<test_hex_roundtrip_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let input: [u8; $size] = input.try_into().unwrap();
                    prop_assert_eq!(<[u8; $size]>::from_hex(&input.to_hex())?, input);
                }

                #[test_strategy::proptest]
                fn [<test_base64_encoding_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let mut arr = [0u8; $size];
                    arr.copy_from_slice(&input);
                    prop_assert_eq!(arr.to_base64(), base64::engine::general_purpose::STANDARD.encode(arr));
                }

                #[test_strategy::proptest]
                fn [<test_base64_roundtrip_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let mut arr = [0u8; $size];
                    arr.copy_from_slice(&input);
                    prop_assert_eq!(<[u8; $size]>::from_base64(&arr.to_base64())?, arr);
                }

                #[test_strategy::proptest]
                fn [<test_equals_ $size>](
                    #[strategy(vec(any::<u8>(), $size))] input1: Vec<u8>,
                    #[strategy(vec(any::<u8>(), $size))] input2: Vec<u8>
                ) {
                    let mut arr1 = [0u8; $size];
                    let mut arr2 = [0u8; $size];
                    arr1.copy_from_slice(&input1);
                    arr2.copy_from_slice(&input2);

                    prop_assert_eq!(arr1.equals(&arr2), arr1 == arr2);
                }
            }
        };
    }

    test_repr_bytes_array!(16);
    test_repr_bytes_array!(32);
    test_repr_bytes_array!(64);
    test_repr_bytes_array!(128);
    test_repr_bytes_array!(256);
    test_repr_bytes_array!(512);
    test_repr_bytes_array!(1024);
    test_repr_bytes_array!(2048);
}

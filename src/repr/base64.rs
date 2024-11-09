#![allow(incomplete_features)]

extern crate alloc;

use alloc::{format, string::String};
use core::simd::*;

use crate::{Error, ReprBytes};

pub trait ReprBase64: ReprBytes {
    fn to_base64(&self) -> String;

    fn from_base64(input: &str) -> Result<Self, Error>
    where
        [(); Self::N]:;
}

macro_rules! impl_repr_array {
    ($size:expr) => {
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

macro_rules! impl_repr_num {
    ($type:ty) => {
        impl ReprBase64 for $type {
            fn to_base64(&self) -> String {
                self.to_le_bytes().to_base64()
            }

            fn from_base64(input: &str) -> Result<Self, Error> {
                let bytes = <[u8; core::mem::size_of::<$type>()]>::from_base64(input)?;
                Ok(<$type>::from_le_bytes(bytes))
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
    // Create a static lookup table where valid chars map to their values, invalid chars map to 255
    static DECODE_TABLE: [u8; 256] = {
        let mut table = [255u8; 256];
        let mut i = 0u8;
        while i < 26 {
            table[b'A' as usize + i as usize] = i;
            table[b'a' as usize + i as usize] = i + 26;
            i += 1;
        }
        let mut i = 0u8;
        while i < 10 {
            table[b'0' as usize + i as usize] = i + 52;
            i += 1;
        }
        table[b'+' as usize] = 62;
        table[b'/' as usize] = 63;
        table[b'=' as usize] = 0;
        table
    };

    let mut result = Simd::splat(0u8);
    for i in 0..32 {
        let decoded = DECODE_TABLE[input[i] as usize];
        if decoded == 255 {
            return Err(Error::InvalidData(format!(
                "Invalid base64 character: {}",
                input[i] as char
            )));
        }
        result[i] = decoded;
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
    static DECODE_TABLE: [u8; 256] = {
        let mut table = [255u8; 256];
        let mut i = 0u8;
        while i < 26 {
            table[b'A' as usize + i as usize] = i;
            table[b'a' as usize + i as usize] = i + 26;
            i += 1;
        }
        let mut i = 0u8;
        while i < 10 {
            table[b'0' as usize + i as usize] = i + 52;
            i += 1;
        }
        table[b'+' as usize] = 62;
        table[b'/' as usize] = 63;
        table[b'=' as usize] = 0;
        table
    };

    let decoded = DECODE_TABLE[input as usize];
    if decoded == 255 {
        return Err(Error::InvalidData(format!(
            "Invalid base64 character: {}",
            input as char
        )));
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use base64::Engine;
    use proptest::{collection::vec, prelude::*};

    use super::*;

    macro_rules! test_repr_array {
        ($size:expr) => {
            paste::paste! {
                #[test_strategy::proptest]
                fn [<test_encoding_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
                    let mut arr = [0u8; $size];
                    arr.copy_from_slice(&input);
                    prop_assert_eq!(arr.to_base64(), base64::engine::general_purpose::STANDARD.encode(arr));
                }

                #[test_strategy::proptest]
                fn [<test_roundtrip_ $size>](#[strategy(vec(any::<u8>(), $size))] input: Vec<u8>) {
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

    test_repr_array!(16);
    test_repr_array!(32);
    test_repr_array!(64);
    test_repr_array!(128);
    test_repr_array!(256);
    test_repr_array!(512);
    test_repr_array!(1024);
    test_repr_array!(2048);
}

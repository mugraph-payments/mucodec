use alloc::{string::String, vec::Vec};
use core::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    simd::{cmp::*, num::*, *},
};

use crate::{from_hex_digit, Error, ReprBase64, ReprBytes, ReprHex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Bytes<const N: usize>([u8; N]);

impl<const N: usize> Hash for Bytes<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<const N: usize> Default for Bytes<N> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize> Bytes<N> {
    #[cfg(feature = "rand")]
    pub fn random<R: rand::prelude::Rng>(rng: &mut R) -> Self {
        let mut out = [0u8; N];
        rng.fill_bytes(&mut out);
        Self(out)
    }
}

#[cfg(feature = "blake3")]
impl From<blake3::Hash> for Bytes<{ blake3::OUT_LEN }> {
    fn from(hash: blake3::Hash) -> Self {
        Self(*hash.as_bytes())
    }
}

impl<const N: usize> fmt::Debug for Bytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(any(test, feature = "proptest"))]
impl<const N: usize> proptest::prelude::Arbitrary for Bytes<N> {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::{prop::collection::vec, *};
        vec(any::<u8>(), N)
            .prop_map(|x| x.try_into().unwrap())
            .prop_map(Self)
            .boxed()
    }
}

impl<const N: usize> fmt::Display for Bytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl<const N: usize> Deref for Bytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> AsRef<[u8]> for Bytes<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> ReprBytes<N> for Bytes<N> {
    #[inline(always)]
    fn from_bytes(input: [u8; N]) -> Self {
        Self(input)
    }

    #[inline(always)]
    fn as_bytes(&self) -> [u8; N] {
        self.0
    }
}

impl<const N: usize> ReprHex<N> for Bytes<N> {
    #[inline]
    fn to_hex(&self) -> String {
        const LOOKUP: [u8; 16] = *b"0123456789abcdef";
        let mut result = Vec::with_capacity(N * 2);

        // Process full chunks of 16 bytes
        for chunk in self.0.chunks_exact(16) {
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
        let remainder = self.0.chunks_exact(16).remainder();
        for &byte in remainder {
            result.push(LOOKUP[(byte >> 4) as usize]);
            result.push(LOOKUP[(byte & 0xf) as usize]);
        }

        // Safe because we only used valid ASCII hex digits
        unsafe { String::from_utf8_unchecked(result) }
    }

    #[inline]
    fn from_hex(input: &str) -> Result<Self, Error> {
        if input.len() != N * 2 {
            return Err(Error::InvalidDataSize {
                expected: N * 2,
                got: input.len(),
            });
        }

        let input = input.as_bytes();
        let mut result = [0u8; N];

        // Process 32 hex chars (16 bytes) at a time using SIMD
        for (chunk_idx, chunk) in input.chunks_exact(32).enumerate() {
            let v: Simd<u8, 32> = Simd::from_slice(chunk);

            // Check which chars are digits (0-9) vs letters (a-f)
            let is_digit = v.simd_ge(Simd::splat(b'0')) & v.simd_le(Simd::splat(b'9'));
            let is_alpha = v.simd_ge(Simd::splat(b'a')) & v.simd_le(Simd::splat(b'f'));

            // Validate that all input chars were valid hex digits
            if !(is_digit | is_alpha).all() {
                // Use first invalid character found for the error
                let chars = v.to_array();
                for &c in &chars {
                    if !(c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
                        return Err(Error::InvalidHexDigit(c as char));
                    }
                }
                // Unreachable as we know there's an invalid char
                unreachable!()
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

        Ok(Self(result))
    }
}

impl<const N: usize> ReprBase64<N> for Bytes<N> {
    #[inline]
    fn to_base64(&self) -> String {
        const BASE64_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::with_capacity((N + 2) / 3 * 4);
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
    fn from_base64(input: &str) -> Result<Self, Error> {
        if input.len() != Self::BASE64_SIZE {
            return Err(Error::InvalidDataSize {
                expected: Self::BASE64_SIZE,
                got: input.len(),
            });
        }

        let input = input.as_bytes();
        let mut result = [0u8; N];
        let mut chunks = input.chunks_exact(32);
        let mut out_idx = 0;

        // Process full chunks with SIMD
        for chunk in chunks.by_ref() {
            if out_idx + 24 > N {
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
                if out_idx >= N {
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

        Ok(Self(result))
    }
}

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
            return Err(Error::InvalidBase64Character(input[i] as char));
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
        return Err(Error::InvalidBase64Character(input as char));
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    macro_rules! test_size {
        ($size:expr) => {
            paste::paste! {
                mod [<bytes_ $size>] {
                    use proptest::prelude::*;
                    use crate::*;

                    #[test_strategy::proptest]
                    fn test_roundtrip(input: Bytes<$size>) {
                        prop_assert_eq!(Bytes::<$size>::from_bytes(input.as_bytes()), input);
                    }
                }

                mod [<hex_ $size>] {
                    use proptest::prelude::*;
                    use crate::*;

                    #[test_strategy::proptest]
                    fn test_encoding(input: Bytes<$size>) {
                        prop_assert_eq!(input.to_hex(), hex::encode(*input));
                    }

                    #[test_strategy::proptest]
                    fn test_roundtrip(input: Bytes<$size>) {
                        prop_assert_eq!(Bytes::<$size>::from_hex(&input.to_hex())?, input);
                    }
                }

                mod [<base64_ $size>] {
                    use base64::Engine;
                    use proptest::prelude::*;

                    use crate::*;

                    #[test_strategy::proptest]
                    fn test_encoding(input: Bytes<$size>) {
                        prop_assert_eq!(input.to_base64(), base64::engine::general_purpose::STANDARD.encode(*input));
                    }

                    #[test_strategy::proptest]
                    fn test_roundtrip(input: Bytes<$size>) {
                        prop_assert_eq!(Bytes::<$size>::from_base64(&input.to_base64())?, input);
                    }
                }
            }
        };
    }

    test_size!(16);
    test_size!(32);
    test_size!(64);
    test_size!(128);
    test_size!(256);
    test_size!(512);
    test_size!(1024);
    test_size!(2048);
}

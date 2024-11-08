#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![cfg_attr(feature = "simd", feature(portable_simd))]
#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
#[cfg(feature = "simd")]
use core::simd::{cmp::*, num::*, *};

use base64::Engine;

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
}

pub trait ReprBase64: ReprBytes {
    fn to_base64(&self) -> String;
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
            fn equals(&self, other: &Self) -> bool {
                self.as_bytes() == other.as_bytes()
            }
        }

        impl ReprHex for [u8; $size] {
            #[inline(always)]
            fn to_hex(&self) -> String {
                hex::encode(self.as_bytes())
            }
        }

        impl ReprBase64 for [u8; $size] {
            #[inline(always)]
            fn to_base64(&self) -> String {
                base64::engine::general_purpose::STANDARD.encode(self.as_bytes())
            }
        }
    };
}

macro_rules! impl_repr_bytes_array_simd {
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

            #[cfg(feature = "simd")]
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
            #[cfg(not(feature = "simd"))]
            #[inline(always)]
            fn to_hex(&self) -> String
            where
                [(); Self::N]:,
            {
                hex::encode(self.as_bytes())
            }

            #[cfg(feature = "simd")]
            #[inline]
            fn to_hex(&self) -> String
            where
                [(); Self::N]:,
            {
                const LOOKUP: [u8; 16] = *b"0123456789abcdef";
                let mut result = String::with_capacity(Self::N * 2);
                let result_ptr = result.as_mut_ptr();

                for (i, chunk) in self.chunks_exact(16).enumerate() {
                    let v: Simd<u8, 16> = Simd::from_slice(chunk);
                    let hi = (v >> 4) & Simd::splat(0x0f);
                    let lo = v & Simd::splat(0x0f);

                    // Convert u8 indices to usize for gather_or_default
                    let hi_indices = hi.cast::<usize>();
                    let lo_indices = lo.cast::<usize>();

                    let hi_lookup = Simd::gather_or_default(&LOOKUP, hi_indices);
                    let lo_lookup = Simd::gather_or_default(&LOOKUP, lo_indices);

                    // Copy values directly to the String's buffer
                    unsafe {
                        hi_lookup.copy_to_slice(core::slice::from_raw_parts_mut(
                            result_ptr.add(i * 32),
                            16,
                        ));
                        lo_lookup.copy_to_slice(core::slice::from_raw_parts_mut(
                            result_ptr.add(i * 32 + 16),
                            16,
                        ));
                    }
                }

                result
            }
        }

        impl ReprBase64 for [u8; $size] {
            #[cfg(not(feature = "simd"))]
            #[inline(always)]
            fn to_base64(&self) -> String
            where
                [(); Self::N]:,
            {
                base64::engine::general_purpose::STANDARD.encode(self.as_bytes())
            }

            #[inline]
            fn to_base64(&self) -> String {
                base64::engine::general_purpose::STANDARD.encode(self.as_bytes())
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
            #[inline(always)]
            fn to_hex(&self) -> String
            where
                [(); Self::N]:,
            {
                hex::encode(self.as_bytes())
            }
        }
    };
}

impl_repr_bytes_array!(1);
impl_repr_bytes_array!(2);
impl_repr_bytes_array!(4);
impl_repr_bytes_array!(8);
impl_repr_bytes_array_simd!(16);
impl_repr_bytes_array_simd!(32);
impl_repr_bytes_array_simd!(64);
impl_repr_bytes_array_simd!(128);
impl_repr_bytes_array_simd!(256);
impl_repr_bytes_array_simd!(512);
impl_repr_bytes_array_simd!(1024);
impl_repr_bytes_array_simd!(2048);
impl_repr_bytes_array_simd!(5192);

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

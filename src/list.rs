use alloc::vec::Vec;
use core::{fmt, mem::size_of, ops::Deref, simd::*};

use crate::*;

macro_rules! impl_list {
    ($type:ty, $list_type:ident, $simd_type:ty, $lanes:ty) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $list_type<const N: usize>([$type; N]);

        #[allow(dead_code)]
        impl<const N: usize> $list_type<N>
        where
            $type: SimdElement,
            $lanes: SupportedLaneCount,
        {
            #[cfg(feature = "rand")]
            pub fn random<R: rand::prelude::Rng>(rng: &mut R) -> Self {
                let mut out = [0; N];
                for i in 0..N {
                    out[i] = rng.gen::<$type>();
                }
                Self(out)
            }

            pub fn pack<const B: usize>(&self) -> Vec<u8> {
                let mut out = Vec::with_capacity(N * B / 8);
                let mask = if B < (size_of::<$type>() * 8) { ((1 << B) - 1) as $type } else { <$type>::MAX };

                for &item in self.0.iter() {
                    out.extend_from_slice(&(item & mask).to_le_bytes()[..B / 8]);
                }

                out
            }

            pub fn unpack<const B: usize>(input: &[u8]) -> Result<Self, Error> {
                if input.len() != N * B / 8 {
                    return Err(Error::InvalidDataSize {
                        expected: N * B / 8,
                        got: input.len(),
                    });
                }

                let mut out = [0; N];
                let mask = if B < (size_of::<$type>() * 8) { ((1 << B) - 1) as $type } else { <$type>::MAX };

                for (i, chunk) in input.chunks_exact(B / 8).enumerate() {
                    let mut bytes = [0u8; size_of::<$type>()];
                    bytes[..B / 8].copy_from_slice(chunk);
                    let value = <$type>::from_le_bytes(bytes);

                    out[i] = value & mask;
                }
                Ok(Self(out))
            }

            pub const fn pack_len(bit: usize) -> usize {
                bit / 8
            }
        }

        impl<const N: usize> fmt::Debug for $list_type<N> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$list_type:snake _tests>] {
                use super::*;
                use proptest::prelude::*;

                fn list<const N: usize>(max_value: $type) -> impl Strategy<Value = $list_type<N>> {
                    prop::collection::vec(0..=max_value, N)
                        .prop_map(|v| $list_type(v.try_into().unwrap()))
                }

                #[test_strategy::proptest]
                fn test_pack_roundtrip_8bit(#[strategy(list::<8>(((1u128 << 8) - 1).min(<$type>::MAX.into()) as $type))] input: $list_type<8>) {
                    prop_assert_eq!($list_type::<8>::unpack::<8>(&input.pack::<8>())?, input);
                }

                #[test_strategy::proptest]
                fn test_pack_roundtrip_16bit(#[strategy(list::<16>(((1u128 << 16) - 1).min(<$type>::MAX.into()) as $type))] input: $list_type<16>) {
                    prop_assert_eq!($list_type::<16>::unpack::<16>(&input.pack::<16>())?, input);
                }

                #[test_strategy::proptest]
                fn test_roundtrip_8bit(#[strategy(list::<16>(<$type>::MAX))] input: $list_type<16>) {
                    prop_assert_eq!(<$list_type<16>>::from_bytes(input.as_bytes()), input);
                }

                #[test_strategy::proptest]
                fn test_roundtrip_16bit(#[strategy(list::<16>(<$type>::MAX))] input: $list_type<16>) {
                    prop_assert_eq!(<$list_type<16>>::from_bytes(input.as_bytes()), input);
                }
            }
        }

        #[cfg(any(test, feature = "proptest"))]
        impl<const N: usize> proptest::arbitrary::Arbitrary for $list_type<N> {
            type Parameters = ();
            type Strategy = proptest::strategy::BoxedStrategy<Self>;

            fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
                use proptest::prelude::*;
                prop::collection::vec(any::<$type>(), N)
                    .prop_map(|v| Self(v.try_into().unwrap()))
                    .boxed()
            }
        }

        impl<const N: usize> fmt::Display for $list_type<N> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }

        impl<const N: usize> Deref for $list_type<N> {
            type Target = [$type; N];

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<const N: usize> AsRef<[$type]> for $list_type<N> {
            fn as_ref(&self) -> &[$type] {
                &self.0
            }
        }

        impl<const N: usize> ReprBytes<{ size_of::<$type>() * N }> for $list_type<N> {
            fn from_bytes(input: [u8; { size_of::<$type>() * N }]) -> Self {
                let mut out = [0; N];
                let mut input_chunks = input.chunks_exact(size_of::<$type>());
                for i in 0..N {
                    let chunk = input_chunks.next().unwrap();
                    out[i] = <$type>::from_le_bytes(chunk.try_into().unwrap());
                }
                Self(out)
            }

            fn as_bytes(&self) -> [u8; { size_of::<$type>() * N }] {
                let mut out = [0u8; { size_of::<$type>() * N }];

                for (i, val) in self.0.iter().enumerate() {
                    let start = i * size_of::<$type>();
                    let end = start + size_of::<$type>();

                    out[start..end].copy_from_slice(&val.to_le_bytes());
                }
                out
            }
        }
    };
}

impl_list!(u16, ListU16, u16x4, LaneCount<4>);
impl_list!(u32, ListU32, u32x4, LaneCount<4>);
impl_list!(u64, ListU64, u64x2, LaneCount<2>);

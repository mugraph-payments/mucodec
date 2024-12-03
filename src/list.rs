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
            pub fn zero() -> Self {
                Self([0; N])
            }

            #[cfg(feature = "rand")]
            pub fn random<R: rand::prelude::Rng>(rng: &mut R) -> Self {
                let mut out = [0; N];

                for i in 0..N {
                    out[i] = rng.gen::<$type>();
                }

                Self(out)
            }
        }

        impl<const N: usize> ReprPacked for $list_type<N>
        where
            $type: SimdElement,
            $lanes: SupportedLaneCount,
            Self: ReprBytes<{ size_of::<$type>() * N + 1 }>,
        {
            fn pack(&self) -> (usize, Vec<u8>) {
                // Find maximum value to determine required bits
                let max_val = self.0.iter().copied().max().unwrap_or(0);
                let bit_width = if max_val == 0 {
                    0
                } else {
                    (core::mem::size_of::<$type>() * 8) - max_val.leading_zeros() as usize
                };

                // Calculate packed size in bytes (rounding up)
                let byte_size = (N * bit_width + 7) / 8;
                let mut out = Vec::with_capacity(byte_size);
                let mask = if bit_width < (<$type>::BITS as usize) {
                    ((1 as $type) << bit_width) - 1
                } else {
                    <$type>::MAX
                };

                // Pack values using determined bit width
                for &item in self.0.iter() {
                    let bytes = (item & mask).to_le_bytes();
                    out.extend_from_slice(&bytes[..((bit_width + 7) / 8)]);
                }

                // Trim any excess capacity
                out.truncate(byte_size);

                (bit_width, out)
            }

            fn unpack(bit_width: usize, input: &[u8]) -> Result<Self, Error> {
                // Special case: if bit_width is 0, all values are 0
                if bit_width == 0 {
                    return Ok(Self([0; N]));
                }

                let expected_size = (N * bit_width + 7) / 8;
                if input.len() != expected_size {
                    return Err(Error::InvalidDataSize {
                        expected: expected_size,
                        got: input.len(),
                    });
                }

                let mut out = [0; N];
                let mask = if bit_width < (<$type>::BITS as usize) {
                    ((1 as $type) << bit_width) - 1
                } else {
                    <$type>::MAX
                };

                let bytes_per_value = (bit_width + 7) / 8;
                for (i, chunk) in input.chunks(bytes_per_value).enumerate() {
                    if i >= N {
                        break;
                    }

                    let mut bytes = [0u8; size_of::<$type>()];
                    bytes[..chunk.len()].copy_from_slice(chunk);
                    let value = <$type>::from_le_bytes(bytes);

                    out[i] = value & mask;
                }

                Ok(Self(out))
            }
        }

        impl<const N: usize> fmt::Debug for $list_type<N> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "List<{N}>")
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

        impl<const N: usize> Default for $list_type<N> {
            fn default() -> Self {
                Self::zero()
            }
        }

        impl<const N: usize> ReprBytes<{ size_of::<$type>() * N + 1 }> for $list_type<N> {
            fn from_bytes(input: [u8; { size_of::<$type>() * N + 1 }]) -> Self {
                // First byte contains the bit width
                let bit_width = input[0] as usize;
                let data_size = (N * bit_width + 7) / 8;

                // Extract the packed data
                let packed = &input[1..1 + data_size];

                // Unpack using the stored bit width
                Self::unpack(bit_width, packed).unwrap()
            }

            fn as_bytes(&self) -> [u8; { size_of::<$type>() * N + 1 }] {
                let mut out = [0u8; { size_of::<$type>() * N + 1 }];

                // Pack the data and get bit width
                let (bit_width, packed) = self.pack();
                let data_size = (N * bit_width + 7) / 8;

                // Store bit width in first byte
                out[0] = bit_width as u8;

                // Copy packed data
                out[1..1 + data_size].copy_from_slice(&packed);

                // Only add padding and sentinel if there's room
                let total = out.len();
                if 1 + data_size < total - 1 {
                    // Fill remaining bytes with zeros
                    out[1 + data_size..total - 1].fill(0);
                    // Set last byte to 0xFF as sentinel
                    out[total - 1] = 0xFF;
                }

                out
            }
        }
    };
}

impl_list!(u16, ListU16, u16x4, LaneCount<4>);
impl_list!(u32, ListU32, u32x4, LaneCount<4>);
impl_list!(u64, ListU64, u64x2, LaneCount<2>);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    macro_rules! generate_tests {
        ($list_type:ident, $type:ty, $($size:expr),+) => {
            paste::paste! {
                mod [<$list_type:snake _tests>] {
                    use super::*;

                    fn list<const N: usize>(max_value: $type) -> impl Strategy<Value = $list_type<N>> {
                        prop::collection::vec(0..=max_value, N)
                            .prop_map(|v| $list_type(v.try_into().unwrap()))
                    }

                    $(
                        #[test]
                        fn [<test_pack_zero_ $size>]() {
                            let input =  $list_type::<$size>::zero();
                            let (bit_width, packed) = input.pack();

                            assert_eq!($list_type::<$size>::unpack(bit_width, &packed).unwrap(), input);
                        }

                        #[test_strategy::proptest]
                        fn [<test_pack_roundtrip_ $size>](
                            #[strategy(list::<$size>(((1u128 << 16) - 1).min(<$type>::MAX.into()) as $type))]
                            input: $list_type<$size>
                        ) {
                            let (bit_width, packed) = input.pack();
                            prop_assert_eq!($list_type::<$size>::unpack(bit_width, &packed).unwrap(), input);
                        }

                        #[test_strategy::proptest]
                        fn [<test_pack_compression_ $size>](
                            #[strategy(list::<$size>(((1u128 << 16) - 1).min(<$type>::MAX.into()) as $type))]
                            input: $list_type<$size>
                        ) {
                            let (bit_width, packed) = input.pack();
                            let expected_packed_size = ($size * bit_width + 7) / 8;
                            prop_assert_eq!(packed.len(), expected_packed_size);
                        }

                        #[test_strategy::proptest]
                        fn [<test_roundtrip_ $size>](
                            #[strategy(list::<$size>(<$type>::MAX))]
                            input: $list_type<$size>
                        ) {
                            prop_assert_eq!(<$list_type<$size>>::from_bytes(input.as_bytes()), input);
                        }

                        #[test_strategy::proptest]
                        fn [<test_padding_ $size>](list: $list_type<$size>) {
                            let bytes = list.as_bytes();
                            let (bit_width, packed) = list.pack();

                            // Check that the first byte contains the correct bit width
                            prop_assert_eq!(bytes[0] as usize, bit_width);

                            // Check that the actual data is present
                            prop_assert_eq!(&bytes[1..1 + packed.len()], packed.as_slice());

                            // Only verify padding if there's enough space for both padding and sentinel
                            if 1 + packed.len() < bytes.len() - 1 {
                                // Check that the padding is all zeros except the last byte
                                prop_assert!(bytes[1 + packed.len()..bytes.len() - 1].iter().all(|&b| b == 0));
                                prop_assert_eq!(*bytes.last().unwrap(), 0xFF);
                            }
                        }
                    )*
                }
            }
        };
    }

    generate_tests!(ListU16, u16, 64, 128);
    generate_tests!(ListU32, u32, 64, 128);
    generate_tests!(ListU64, u64, 64, 128);
}

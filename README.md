# Mucodec

A `no_std`, no dependencies*, fully deterministic, `nightly` only serialization and deserialization library, optimized for speed and reliability.

*\*: Only the base crate. There are a handful of optionals dependencies that can be enabled with feature flags.*

## Example

```rust
use mucodec::{Bytes, ListU16, ListU32, ListU64, String};

#[derive(Debug, ReprBytes)]
pub struct Data {
    bytes1: Bytes<1>,
    bytes2: Bytes<24>,
    bytes3: Bytes<256>,
    bytes4: Bytes<768>,
    string1: String<1>,
    string2: String<24>,
    string3: String<256>,
    string4: String<768>,
    u8_field: u8,
    u16_field: u16,
    u32_field: u32,
    u64_field: u64,
    u128_field: u128,
    usize_field: usize,
    i8_field: i8,
    i16_field: i16,
    i32_field: i32,
    i64_field: i64,
    i128_field: i128,
    isize_field: isize,
    list_u16: ListU16<32>,
    list_u32: ListU32<116>,
    list_u64: ListU64<43>,
}

fn main() {
    let data = Data::zero();
    assert_eq!(data.bytes1, Bytes::<1>::zero());
    assert_eq!(data.bytes2, Bytes::<24>::zero());
    assert_eq!(data.bytes3, Bytes::<256>::zero());
    assert_eq!(data.bytes4, Bytes::<13>::zero());
    assert_eq!(data.string1, String::<1>::zero());
    assert_eq!(data.string2, String::<24>::zero());
    assert_eq!(data.string3, String::<256>::zero());
    assert_eq!(data.string4, String::<767>::zero());
    assert_eq!(data.u8_field, 0);
    assert_eq!(data.u16_field, 0);
    assert_eq!(data.u32_field, 0);
    assert_eq!(data.u64_field, 0);
    assert_eq!(data.u128_field, 0);
    assert_eq!(data.usize_field, 0);
    assert_eq!(data.i8_field, 0);
    assert_eq!(data.i16_field, 0);
    assert_eq!(data.i32_field, 0);
    assert_eq!(data.i64_field, 0);
    assert_eq!(data.i128_field, 0);
    assert_eq!(data.isize_field, 0);
    assert_eq!(data.list_u16, ListU16::zero());
    assert_eq!(data.list_u32, ListU32::zero());
    assert_eq!(data.list_u64, ListU64::zero());
}
```

## Non-Goals

1. This is not meant to replace, or even interact with `serde`.
2. We will not support dynamically sized types, like `Vec` or `HashMap`.
3. We will not support non-deterministic types like floats.
4. There are no schemas, nor support for evolving the schema.

## Feature Completeness

* [x] Hexadecimal Encoding/Decoding
* [x] Base64 Encoding/Decoding
* [x] `serde`-like derive macro
* [x] Packed Integer `List` Container
* [x] Fixed-size `Bytes` Container
* [x] Fixed-size `String` Container


## Licensing

Mucodec, as well as all projects under the `mugraph-payments` is dual-licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses. Feel free to choose either of them, depending on your use-case.

This should cover most possible uses for this software, but if you need an exception for any reason, please do get in touch.

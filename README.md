# Mucodec

A `no_std`, no dependencies*, fully deterministic, `nightly` only serialization and deserialization library, optimized for speed and reliability.

*\*: Only the base crate. There are a handful of optionals dependencies that can be enabled with feature flags.*

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

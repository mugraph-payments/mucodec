# Mucodec

A `no_std`, no dependencies*, fully deterministic, `nightly` only serialization and deserialization library, optimized for speed and reliability.

*\*: Only the base crate, if you enable the `derive` feature then we depend on [`syn`](https://docs.rs/syn).*

## Non-Goals

1. This is not meant to replace, or even interact with `serde`.
2. We will not support dynamically sized types, like `Vec` or `HashMap`.
3. We will not support non-deterministic types like floats.
4. There are no schemas, nor support for evolving the schema.

## Feature Completeness

* [x] Hexadecimal Encoding/Decoding
* [x] Base64 Encoding/Decoding
* [x] `Encode` derive macro
* [x] `Decode` derive macro
* [ ] Integer Binary Encoding

## Licensing

Mucodec, as well as all projects under the `mugraph-payments` is dual-licensed under the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses. Feel free to choose either of them, depending on your use-case.

This should cover most possible uses for this software, but if you need an exception for any reason, please do get in touch.

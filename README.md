# mucodec

A `no_std`, no dependencies, fully deterministic, `nightly` only serialization and deserialization library, optimized for speed and reliability.

## Non-Goals

1. This is not meant to replace, or even interact with `serde`.
2. We will not support dynamically sized types, like `Vec` or `HashMap`.
3. We will not support non-deterministic types like floats.
4. There are no schemas, nor support for evolving the schema.

## Feature Completeness

* [x] Hexadecimal Encoding/Decoding
* [x] Base64 Encoding/Decoding
* [ ] Integer Binary Encoding
* [ ] `Encode` derive macro
* [ ] `Decode` derive macro

use mucodec::{Bytes, ReprBytes};

#[derive(Debug, ReprBytes)]
pub struct Data {
    bytes1: Bytes<1>,
    bytes2: Bytes<24>,
    bytes3: Bytes<256>,
    bytes4: Bytes<768>,
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
}

fn main() {
    let data = Data::zero();
    assert_eq!(data.bytes1, Bytes::<1>::zero());
    assert_eq!(data.bytes2, Bytes::<24>::zero());
    assert_eq!(data.bytes3, Bytes::<256>::zero());
    assert_eq!(data.bytes4, Bytes::<768>::zero());
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
}

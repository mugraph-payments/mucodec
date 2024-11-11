use mucodec::{Bytes, ReprBytes};

#[derive(Debug, ReprBytes)]
pub struct Data {
    name: Bytes<32>,
    address1: u32,
    address2: u128,
    address3: Bytes<64>,
}

fn main() {
    let data = Data::zero();
    assert_eq!(data.name, Bytes::<32>::zero());
    assert_eq!(data.address1, 0u32);
    assert_eq!(data.address2, 0u128);
    assert_eq!(data.address3, Bytes::<64>::zero());
}

use mucodec::{Bytes, ReprBytes};

#[derive(Debug, ReprBytes)]
pub struct Data {
    name: Bytes<32>,
}

fn main() {
    let data = Data::zero();
    assert_eq!(data.name, Bytes::<32>::zero());
}

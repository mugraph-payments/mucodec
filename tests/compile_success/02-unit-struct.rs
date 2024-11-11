use mucodec::{Bytes, ReprBytes};

#[derive(Debug, ReprBytes)]
pub struct Unit(Bytes<32>);

fn main() {
    let unit = Unit(Bytes::<32>::zero());
    assert_eq!(unit.0.as_bytes(), [0u8; 32]);
}

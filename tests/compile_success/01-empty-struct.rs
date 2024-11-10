use mucodec::ReprBytes;

#[derive(Debug, PartialEq, ReprBytes)]
pub struct Empty;

fn main() {
    assert_eq!(Empty.as_bytes(), []);
}

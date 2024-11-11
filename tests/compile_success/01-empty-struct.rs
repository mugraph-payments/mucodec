use mucodec::ReprBytes;

#[derive(Debug, ReprBytes)]
pub struct Empty;

fn main() {
    assert_eq!(Empty.as_bytes(), []);
}

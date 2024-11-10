use mucodec::ReprBytes;

#[derive(ReprBytes)]
pub struct Empty;

fn main() {
    assert_eq!(Empty.as_bytes(), []);
}

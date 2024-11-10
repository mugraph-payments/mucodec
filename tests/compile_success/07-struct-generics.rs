use mucodec::ReprBytes;

#[derive(Debug, ReprBytes)]
pub struct Data<T: ReprBytes<4>> {
    inner: T,
}

fn main() {
    let data = Data::<u32>::zero();
    assert_eq!(data.inner, 0u32);
}

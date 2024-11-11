use mucodec::{Bytes, ReprBytes};

#[derive(Debug, ReprBytes)]
pub struct Data<T> {
    inner: T,
}

#[derive(Debug)]
pub struct Fail;

fn main() {
    let data = Data::<Fail>::zero();
    assert_eq!(data.inner, Fail);
}

use mucodec::{Bytes, ReprBytes};

#[derive(ReprBytes)]
pub struct A(Bytes<64>);

#[derive(ReprBytes)]
pub struct B(A);

#[derive(ReprBytes)]
pub struct C {
    a: A,
    b: B,
}

#[derive(ReprBytes)]
pub struct Data {
    a: A,
    b: B,
    c: C,
}

fn main() {
    let data = Data::zero();
    assert_eq!(data.a, A::zero());
    assert_eq!(data.b, B::zero());
    assert_eq!(data.c, C::zero());
    assert_eq!(data.c.a, A::zero());
    assert_eq!(data.c.b, B::zero());
}

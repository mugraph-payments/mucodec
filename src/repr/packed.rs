use alloc::vec::Vec;

use crate::Error;

pub trait ReprPacked: Sized {
    fn pack(&self) -> (usize, Vec<u8>);
    fn unpack(bit_width: usize, input: &[u8]) -> Result<Self, Error>;
}

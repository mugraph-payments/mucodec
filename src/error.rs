use alloc::string::{String, ToString};
use core::array::TryFromSliceError;

#[derive(onlyerror::Error, Debug)]
pub enum Error {
    #[error("Invalid data type: {0}")]
    InvalidData(String),
}

impl From<TryFromSliceError> for Error {
    fn from(value: TryFromSliceError) -> Self {
        Self::InvalidData(value.to_string())
    }
}

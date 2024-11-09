use alloc::string::{String, ToString};
use core::{array::TryFromSliceError, fmt};

pub enum Error {
    InvalidData(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData(msg) => write!(f, "InvalidData({})", msg),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData(msg) => write!(f, "Invalid data type: {}", msg),
        }
    }
}

impl core::error::Error for Error {}

impl From<TryFromSliceError> for Error {
    fn from(value: TryFromSliceError) -> Self {
        Self::InvalidData(value.to_string())
    }
}

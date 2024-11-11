use alloc::string::{String, ToString};
use core::{array::TryFromSliceError, fmt};

pub enum Error {
    InvalidDataSize { expected: usize, got: usize },
    InvalidHexDigit(char),
    InvalidBase64Character(char),
    SliceConversionError(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidDataSize { expected, got } => {
                write!(
                    f,
                    "InvalidDataSize {{ expected: {}, got: {} }}",
                    expected, got
                )
            }
            Error::InvalidHexDigit(c) => write!(f, "InvalidHexDigit({})", c),
            Error::InvalidBase64Character(c) => write!(f, "InvalidBase64Character({})", c),
            Error::SliceConversionError(msg) => write!(f, "SliceConversionError({})", msg),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidDataSize { expected, got } => {
                write!(f, "Invalid data size: expected {}, got {}", expected, got)
            }
            Error::InvalidHexDigit(c) => write!(f, "Invalid hex digit: {}", c),
            Error::InvalidBase64Character(c) => write!(f, "Invalid base64 character: {}", c),
            Error::SliceConversionError(msg) => write!(f, "Slice conversion error: {}", msg),
        }
    }
}

impl core::error::Error for Error {}

impl From<TryFromSliceError> for Error {
    fn from(value: TryFromSliceError) -> Self {
        Self::SliceConversionError(value.to_string())
    }
}

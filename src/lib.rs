#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod bytes;
mod error;
mod list;
mod repr;
mod string;

#[cfg(feature = "derive")]
extern crate mucodec_derive;

#[cfg(feature = "derive")]
pub use mucodec_derive::*;

pub use self::{bytes::Bytes, error::Error, list::*, repr::*, string::*};

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod error;
mod repr;
pub use self::{bytes::Bytes, error::Error, repr::*};

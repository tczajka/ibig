//! A big integer library.

#![no_std]

extern crate alloc;

pub use error::TryFromBigError;
pub use repr::{IBig, UBig};

#[cfg(feature = "proptest")]
pub mod proptest;

mod add;
mod bits;
mod bitwise;
mod bytes;
mod cmp;
mod convert;
mod error;
mod misc;
mod ops;
mod repr;
mod shift;
mod sign;
mod sub;

#[cfg(test)]
mod tests;

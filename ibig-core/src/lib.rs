//! Core big-integer algorithms.
//!
//! This crate implements the fundamental arithmetic algorithms — addition, subtraction,
//! multiplication and division — operating on sequences of [`Digit`]s.
//!
//! Numbers are represented as slices of [`Digit`]s in little-endian order — the
//! least-significant digit first — regardless of the platform's native byte order (so the
//! layout is the same on big-endian machines).
//!
//! Function names describe each operand explicitly. The
//! operand vocabulary is:
//!
//! * `unsigned` — a slice of digits interpreted as an unsigned (non-negative) value
//! * `signed` — a slice of digits interpreted as a signed value (always two's complement)
//! * `digit` — a single [`Digit`]
//! * `idigit` — a single [`IDigit`]
//! * `carry` — a `bool` carry (0 or 1)
//! * `icarry` — a [`IDigit`] carry (-1, 0, or 1)
//! * `borrow` — a `bool` borrow (0 or 1)

#![no_std]

mod add;
mod bits;
mod bitwise;
mod bytes;
mod len;
mod shift;
mod sign;
mod sub;

pub use add::{
    add_digit_idigit, add_idigit_idigit, add_signed_digit, add_signed_icarry, add_signed_idigit,
    add_signed_signed, add_signed_unsigned, add_unsigned_1, add_unsigned_carry, add_unsigned_digit,
    add_unsigned_icarry, add_unsigned_idigit, add_unsigned_signed, add_unsigned_unsigned,
    add_unsigned_unsigned_same_len,
};
pub use bits::{
    BitIndex, BitIndexOutOfRange, DIGIT_BITS_USIZE, bit_signed, bit_unsigned, count_ones,
    highest_one, is_power_of_two, lowest_one, lowest_zero, next_power_of_two, set_bit,
};
pub use bitwise::{bitand_same_len, bitandnot_same_len, bitor_same_len, bitxor_same_len, not};
pub use bytes::{
    from_be_bytes_signed, from_be_bytes_unsigned, from_bytes_signed, from_bytes_unsigned,
    to_bytes_signed, to_bytes_unsigned,
};
pub use len::{min_len_bytes_signed, min_len_bytes_unsigned, min_len_signed, min_len_unsigned};
pub use shift::{
    shl_small_digit, shl_small_idigit, shl_small_signed, shl_small_unsigned, shr_small_signed,
    shr_small_unsigned,
};
pub use sign::{
    abs, extend_signed, extend_signed_bytes, is_negative, neg, neg_borrow, sign_extension,
    sign_extension_byte, sign_extension_idigit,
};
pub use sub::{
    sub_digit_idigit, sub_idigit_digit, sub_idigit_idigit, sub_rev_signed_idigit,
    sub_rev_signed_signed, sub_rev_unsigned_idigit, sub_rev_unsigned_signed,
    sub_rev_unsigned_unsigned_same_len, sub_signed_digit, sub_signed_idigit, sub_signed_signed,
    sub_signed_unsigned, sub_unsigned_1, sub_unsigned_borrow, sub_unsigned_digit,
    sub_unsigned_idigit, sub_unsigned_signed, sub_unsigned_unsigned,
    sub_unsigned_unsigned_same_len,
};
use unative::{INative, UNative};

/// A single digit of a big integer.
///
/// Big integers are represented as little-endian sequences of `Digit`s. A `Digit` is the
/// platform's native unsigned integer type ([`UNative`]), chosen for efficient hardware
/// arithmetic.
pub type Digit = UNative;

/// A [`Digit`] interpreted as a signed value.
///
/// This is the platform's native signed integer type ([`INative`]), the same width as
/// [`Digit`]. It is used as the most-significant digit of a signed number, where the sign
/// bit lives.
pub type IDigit = INative;

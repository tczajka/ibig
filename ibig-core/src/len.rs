//! Minimal canonical length of a digit or byte slice.

use crate::Digit;
use crate::sign::{sign_extension_byte, sign_extension_idigit};

/// Returns the minimum number of digits needed to represent the value held in `digits`.
///
/// This is the length with the most-significant zero digits removed. It is 0 for the value
/// zero (an empty slice, or a slice of only zero digits).
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, min_len_unsigned};
/// assert_eq!(min_len_unsigned(&[Digit::from(5u8), Digit::from(2u8)]), 2);
/// assert_eq!(min_len_unsigned(&[Digit::from(5u8), Digit::ZERO]), 1);
/// assert_eq!(min_len_unsigned(&[Digit::ZERO, Digit::ZERO]), 0);
/// ```
pub const fn min_len_unsigned(digits: &[Digit]) -> usize {
    let mut len = digits.len();
    while len > 0 && digits[len - 1].const_eq(Digit::ZERO) {
        len -= 1;
    }
    len
}

/// Given non-empty signed `digits`, returns the minimum number of digits needed to
/// represent the value.
///
/// This is the length with redundant most-significant sign-extension digits removed (a top
/// digit that merely repeats the sign of the digit below it), but always at least 1 (a
/// signed value always needs at least one digit to carry its sign).
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, min_len_signed};
/// // 5 is positive in a single digit, so the redundant zero sign-extension is dropped.
/// assert_eq!(min_len_signed(&[Digit::from(5u8), Digit::ZERO]), 1);
/// // -1 is all-ones; the redundant all-ones sign-extension digits are dropped.
/// assert_eq!(min_len_signed(&[Digit::MAX, Digit::MAX]), 1);
/// // The top digit's high bit being set means the leading zero digit is needed to keep
/// // this value positive, so it is not dropped.
/// assert_eq!(min_len_signed(&[Digit::MAX, Digit::ZERO]), 2);
/// ```
pub const fn min_len_signed(digits: &[Digit]) -> usize {
    assert!(!digits.is_empty());
    let mut len = digits.len();
    while len > 1
        && digits[len - 1]
            .const_eq(sign_extension_idigit(digits[len - 2].cast_signed()).cast_unsigned())
    {
        len -= 1;
    }
    len
}

/// Given a little-endian slice of bytes, returns the minimum number of bytes needed to
/// represent the value: the length with the most-significant zero bytes removed.
///
/// This is 0 for the value zero (an empty slice, or a slice of only zero bytes). It is the
/// byte analogue of [`min_len_unsigned`].
///
/// # Examples
///
/// ```
/// # use ibig_core::min_len_bytes_unsigned;
/// assert_eq!(min_len_bytes_unsigned(&[]), 0);
/// assert_eq!(min_len_bytes_unsigned(&[0, 0]), 0);
/// assert_eq!(min_len_bytes_unsigned(&[5, 0]), 1);
/// assert_eq!(min_len_bytes_unsigned(&[0, 1]), 2);
/// ```
pub fn min_len_bytes_unsigned(bytes: &[u8]) -> usize {
    let mut len = bytes.len();
    while len > 0 && bytes[len - 1] == 0 {
        len -= 1;
    }
    len
}

/// Given a non-empty slice of signed bytes, returns the minimum
/// number of bytes needed to represent the value: the length with redundant
/// most-significant sign-extension bytes removed, but always at least 1.
///
/// This is the byte analogue of [`min_len_signed`].
///
/// # Panics
///
/// Panics if `bytes` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::min_len_bytes_signed;
/// // A redundant zero sign byte above a non-negative byte is dropped.
/// assert_eq!(min_len_bytes_signed(&[5, 0]), 1);
/// // -1 is all-ones; the redundant 0xff sign bytes are dropped.
/// assert_eq!(min_len_bytes_signed(&[0xff, 0xff]), 1);
/// // 0xc8 alone is negative, so a leading zero byte is needed to stay positive (200).
/// assert_eq!(min_len_bytes_signed(&[0xc8, 0x00]), 2);
/// ```
pub fn min_len_bytes_signed(bytes: &[u8]) -> usize {
    assert!(!bytes.is_empty());
    let mut len = bytes.len();
    while len > 1
        && bytes[len - 1] == sign_extension_byte(bytes[len - 2].cast_signed()).cast_unsigned()
    {
        len -= 1;
    }
    len
}

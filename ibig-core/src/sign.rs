//! Sign and sign-extension of signed digit and byte slices.

use crate::{Digit, IDigit, not};

/// Returns `true` if the non-empty signed `digits` represent a negative value (the
/// most-significant digit's sign bit is set).
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, is_negative};
/// assert!(is_negative(&[Digit::MAX])); // -1
/// assert!(!is_negative(&[Digit::from(5u8)])); // +5
/// assert!(!is_negative(&[Digit::MAX, Digit::ZERO])); // a positive multi-digit value
/// ```
pub const fn is_negative(digits: &[Digit]) -> bool {
    digits.last().unwrap().cast_signed().is_negative()
}

/// Negates the signed value in the non-empty `digits` in place, returning a signed carry (0 or -1).
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, neg};
/// // -1 negates to 1.
/// let mut a = [Digit::MAX];
/// let high = neg(&mut a);
/// assert_eq!(a, [Digit::from(1u8)]);
/// assert_eq!(high, IDigit::ZERO);
/// ```
pub fn neg(digits: &mut [Digit]) -> IDigit {
    assert!(!digits.is_empty(), "signed digits are empty");
    // Skip zeros.
    let Some(index) = digits.iter().position(|&d| d != Digit::ZERO) else {
        return IDigit::ZERO;
    };
    let high = &mut digits[index..];
    let extension = sign_extension(high);
    let (mid, high) = high.split_first_mut().unwrap();
    *mid = mid.wrapping_neg();
    not(high);
    !extension
}

/// Negates the signed value in the non-empty `digits` and subtracts `borrow` (0 or 1) in place,
/// returning the signed carry (0 or -1).
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, neg_borrow};
/// // -(-1) - 0 == 1.
/// let mut a = [Digit::MAX];
/// assert_eq!(neg_borrow(&mut a, false), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(1u8)]);
///
/// // -(-1) - 1 == 0.
/// let mut a = [Digit::MAX];
/// assert_eq!(neg_borrow(&mut a, true), IDigit::ZERO);
/// assert_eq!(a, [Digit::ZERO]);
///
/// // -3 - 1 == -4, a borrow out of the most-significant digit.
/// let mut a = [Digit::from(3u8)];
/// assert_eq!(neg_borrow(&mut a, true), IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX - Digit::from(3u8)]);
/// ```
pub fn neg_borrow(digits: &mut [Digit], borrow: bool) -> IDigit {
    if borrow {
        // -x - 1 == !x, which always fits in the same number of digits.
        not(digits);
        sign_extension(digits)
    } else {
        neg(digits)
    }
}

/// Replaces the non-empty signed `digits` with their absolute value in place. The result is the
/// unsigned magnitude, which always fits in the same number of digits (the magnitude of the most
/// negative value, `2^(bits-1)`, sets the top bit but stays within `bits` unsigned bits).
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, abs};
/// // |-1| == 1.
/// let mut a = [Digit::MAX];
/// abs(&mut a);
/// assert_eq!(a, [Digit::from(1u8)]);
/// // A non-negative value is unchanged.
/// let mut a = [Digit::from(5u8)];
/// abs(&mut a);
/// assert_eq!(a, [Digit::from(5u8)]);
/// ```
pub fn abs(digits: &mut [Digit]) {
    if is_negative(digits) {
        neg(digits);
    }
}

/// Sign-extends the signed value held in `digits[..len]` to fill the rest of
/// `digits` in place: every digit from index `len` onward is set to the value's sign
/// (all-ones if negative, zero otherwise).
///
/// # Panics
///
/// Panics if `len` is 0 or greater than `digits.len()`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, extend_signed};
/// // -1 occupies the low digit; extend it across the buffer.
/// let mut digits = [Digit::MAX, Digit::ZERO, Digit::ZERO];
/// extend_signed(&mut digits, 1);
/// assert_eq!(digits, [Digit::MAX, Digit::MAX, Digit::MAX]);
/// // A non-negative value extends with zeros.
/// let mut digits = [Digit::from(5u8), Digit::MAX];
/// extend_signed(&mut digits, 1);
/// assert_eq!(digits, [Digit::from(5u8), Digit::ZERO]);
/// ```
pub fn extend_signed(digits: &mut [Digit], len: usize) {
    assert!(
        len > 0 && len <= digits.len(),
        "len must be in 1..=digits.len()"
    );
    let fill = sign_extension(&digits[..len]).cast_unsigned();
    digits[len..].fill(fill);
}

/// Sign-extends the signed value held in `bytes[..len]` to fill the rest of `bytes`
/// in place: every byte from index `len` onward is set to the value's sign (all-ones if
/// negative, zero otherwise).
///
/// # Panics
///
/// Panics if `len` is 0 or greater than `bytes.len()`.
///
/// # Examples
///
/// ```
/// # use ibig_core::extend_signed_bytes;
/// // -1 occupies the low byte; extend it across the buffer.
/// let mut bytes = [0xffu8, 0, 0];
/// extend_signed_bytes(&mut bytes, 1);
/// assert_eq!(bytes, [0xff, 0xff, 0xff]);
/// // A non-negative value extends with zeros.
/// let mut bytes = [5u8, 0xff];
/// extend_signed_bytes(&mut bytes, 1);
/// assert_eq!(bytes, [5, 0]);
/// ```
pub fn extend_signed_bytes(bytes: &mut [u8], len: usize) {
    assert!(
        len > 0 && len <= bytes.len(),
        "len must be in 1..=bytes.len()"
    );
    let fill = sign_extension_byte(bytes[len - 1].cast_signed()).cast_unsigned();
    bytes[len..].fill(fill);
}

/// The sign-extension digit of the signed value in the non-empty `digits`: `-1` (all-ones) if
/// the value is negative, `0` otherwise. This is the digit that fills positions above the
/// value.
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, sign_extension};
/// assert_eq!(sign_extension(&[Digit::MAX]), IDigit::from(-1i8)); // -1
/// assert_eq!(sign_extension(&[Digit::from(5u8)]), IDigit::ZERO); // +5
/// // The sign comes from the most-significant digit.
/// assert_eq!(sign_extension(&[Digit::from(5u8), Digit::MAX]), IDigit::from(-1i8));
/// ```
pub const fn sign_extension(digits: &[Digit]) -> IDigit {
    let last = digits.last().expect("signed digits are empty");
    sign_extension_idigit(last.cast_signed())
}

/// The sign-extension digit for a signed value whose most-significant digit is
/// `high`: `-1` (all-ones) if `high` is negative, `0` otherwise.
///
/// # Examples
///
/// ```
/// # use ibig_core::{IDigit, sign_extension_idigit};
/// assert_eq!(sign_extension_idigit(IDigit::from(-2i8)), IDigit::from(-1i8));
/// assert_eq!(sign_extension_idigit(IDigit::from(5i8)), IDigit::ZERO);
/// ```
pub const fn sign_extension_idigit(high: IDigit) -> IDigit {
    // Smear the sign bit across the whole digit: arithmetic-shifting it down to every bit
    // yields all-ones for a negative `high` and all-zeros otherwise.
    high.checked_shr(IDigit::BITS - 1).unwrap()
}

/// The sign-extension byte for a signed value whose most-significant byte is `high`:
/// `-1` (all-ones) if `high` is negative, `0` otherwise.
///
/// # Examples
///
/// ```
/// # use ibig_core::sign_extension_byte;
/// assert_eq!(sign_extension_byte(-2), -1);
/// assert_eq!(sign_extension_byte(5), 0);
/// ```
pub const fn sign_extension_byte(high: i8) -> i8 {
    high >> (i8::BITS - 1)
}

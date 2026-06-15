//! Bit shifts.

use crate::{Digit, SignedDigit, sign::sign_extension_sdigit};

/// Shifts `digits` left by `shift` bits in place, returning the overflow.
///
/// `shift` must be less than [`Digit::BITS`]. The bits shifted out beyond the
/// most-significant digit are returned in the low `shift` bits of the overflow digit (its
/// high bits are zero).
///
/// # Panics
///
/// Panics if `shift >= Digit::BITS`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, shl_small_unsigned};
/// let mut a = [Digit::from(0b101u8)];
/// let overflow = shl_small_unsigned(&mut a, 2);
/// assert_eq!(a, [Digit::from(0b10100u8)]);
/// assert_eq!(overflow, Digit::ZERO);
///
/// // The top bit of the low digit is carried into the next digit.
/// let mut a = [Digit::MAX, Digit::ZERO];
/// let overflow = shl_small_unsigned(&mut a, 1);
/// assert_eq!(a, [!Digit::from(1u8), Digit::from(1u8)]);
/// assert_eq!(overflow, Digit::ZERO);
/// ```
pub fn shl_small_unsigned(digits: &mut [Digit], shift: u32) -> Digit {
    assert!(shift < Digit::BITS);
    if shift == 0 {
        return Digit::ZERO;
    }
    let mut carry = Digit::ZERO;
    for d in digits.iter_mut() {
        let new_carry = *d >> (Digit::BITS - shift);
        *d = (*d << shift) | carry;
        carry = new_carry;
    }
    carry
}

/// Shifts a single `digit` left by `shift` bits, returning the two-digit result `(low, high)`.
///
/// `shift` must be less than [`Digit::BITS`]. The bits shifted out of `digit` end up in the
/// low `shift` bits of `high` (its high bits are zero).
///
/// # Panics
///
/// Panics if `shift >= Digit::BITS`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, shl_small_digit};
/// assert_eq!(shl_small_digit(Digit::from(0b101u8), 2), (Digit::from(0b10100u8), Digit::ZERO));
///
/// // The top bit shifts into the high digit.
/// assert_eq!(shl_small_digit(Digit::MAX, 1), (!Digit::from(1u8), Digit::from(1u8)));
/// ```
pub fn shl_small_digit(digit: Digit, shift: u32) -> (Digit, Digit) {
    assert!(shift < Digit::BITS);
    if shift == 0 {
        return (digit, Digit::ZERO);
    }
    (digit << shift, digit >> (Digit::BITS - shift))
}

/// Shifts `digits` right by `shift` bits in place. The bits shifted out below the
/// least-significant digit are discarded.
///
/// `shift` must be less than [`Digit::BITS`].
///
/// # Panics
///
/// Panics if `shift >= Digit::BITS`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, shr_small_unsigned};
/// let mut a = [Digit::from(0b10100u8)];
/// shr_small_unsigned(&mut a, 2);
/// assert_eq!(a, [Digit::from(0b101u8)]);
///
/// // The high bits of the low digit are pulled down from the next digit.
/// let mut a = [Digit::ZERO, Digit::from(1u8)];
/// shr_small_unsigned(&mut a, 1);
/// assert_eq!(a, [Digit::from(1u8) << (Digit::BITS - 1), Digit::ZERO]);
/// ```
pub fn shr_small_unsigned(digits: &mut [Digit], shift: u32) {
    assert!(shift < Digit::BITS);
    if shift == 0 {
        return;
    }
    let mut carry = Digit::ZERO;
    for d in digits.iter_mut().rev() {
        let new_carry = *d << (Digit::BITS - shift);
        *d = (*d >> shift) | carry;
        carry = new_carry;
    }
}

/// Shifts the signed value in `digits` left by `shift` bits in place, returning the
/// sign-extended overflow.
///
/// `shift` must be less than [`Digit::BITS`]. The `(len + 1)`-digit signed number
/// formed by the shifted digits followed by the overflow equals the original value shifted
/// left by `shift`.
///
/// # Panics
///
/// Panics if `shift >= Digit::BITS`, or if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, SignedDigit, shl_small_signed};
/// let mut a = [Digit::MAX]; // -1
/// let overflow = shl_small_signed(&mut a, 1);
/// assert_eq!(a, [!Digit::from(1u8)]); // -2
/// assert_eq!(overflow, SignedDigit::from(-1i8)); // sign extension of -1
/// ```
pub fn shl_small_signed(digits: &mut [Digit], shift: u32) -> SignedDigit {
    let (top, low) = digits.split_last_mut().expect("digits is empty");
    // Shift the lower digits as unsigned and the top digit as signed, stitching the carry
    // into the top digit.
    let carry = shl_small_unsigned(low, shift);
    let (new_top, overflow) = shl_small_sdigit(top.cast_signed(), shift);
    *top = new_top | carry;
    overflow
}

/// Shifts a single signed `digit` left by `shift` bits, returning the two-digit result
/// `(low, high)`.
///
/// `shift` must be less than [`Digit::BITS`]. The two-digit signed number formed by
/// `low` and `high` equals `digit` shifted left by `shift`; the high `BITS - shift` bits of
/// `high` are filled with the sign of `digit`.
///
/// # Panics
///
/// Panics if `shift >= Digit::BITS`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, SignedDigit, shl_small_sdigit};
/// // -1 << 1 == -2, spanning two digits.
/// assert_eq!(
///     shl_small_sdigit(SignedDigit::from(-1i8), 1),
///     (!Digit::from(1u8), SignedDigit::from(-1i8))
/// );
/// ```
pub fn shl_small_sdigit(digit: SignedDigit, shift: u32) -> (Digit, SignedDigit) {
    assert!(shift < Digit::BITS);
    if shift == 0 {
        // The high digit is pure sign extension.
        return (digit.cast_unsigned(), sign_extension_sdigit(digit));
    }
    // The arithmetic shift sign-extends the high digit.
    (
        digit.cast_unsigned() << shift,
        digit >> (SignedDigit::BITS - shift),
    )
}

/// Shifts the signed value in `digits` right by `shift` bits in place (an
/// arithmetic shift, sign-extending). The bits shifted out below the least-significant digit
/// are discarded.
///
/// `shift` must be less than [`Digit::BITS`]. The vacated high bits of the most-significant
/// digit are filled with the sign of the value.
///
/// # Panics
///
/// Panics if `shift >= Digit::BITS`, or if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, shr_small_signed};
/// let mut a = [!Digit::from(3u8)]; // -4
/// shr_small_signed(&mut a, 1);
/// assert_eq!(a, [!Digit::from(1u8)]); // -2
/// ```
pub fn shr_small_signed(digits: &mut [Digit], shift: u32) {
    assert!(!digits.is_empty());
    assert!(shift < Digit::BITS);
    if shift == 0 {
        return;
    }
    // Shift the top digit separately: the arithmetic shift fills its vacated high bits with
    // the sign.
    let (top, low) = digits.split_last_mut().expect("digits is empty");
    let mut carry = *top << (Digit::BITS - shift);
    *top = (top.cast_signed() >> shift).cast_unsigned();
    for d in low.iter_mut().rev() {
        let new_carry = *d << (Digit::BITS - shift);
        *d = (*d >> shift) | carry;
        carry = new_carry;
    }
}

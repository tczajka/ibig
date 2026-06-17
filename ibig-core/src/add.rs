//! Addition.

use crate::sub::sub_unsigned_1;
use crate::{Digit, IDigit, sign_extension, sign_extension_idigit};

/// Adds `rhs` to `lhs` in place, returning the carry out of the most-significant digit.
/// The slices must have the same length.
///
/// # Panics
///
/// Panics if `lhs` and `rhs` have different lengths.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, add_unsigned_unsigned_same_len};
/// let mut a = [Digit::MAX, Digit::MAX];
/// let carry = add_unsigned_unsigned_same_len(&mut a, &[Digit::from(1u8), Digit::ZERO]);
/// assert_eq!(a, [Digit::ZERO, Digit::ZERO]);
/// assert!(carry);
/// ```
pub fn add_unsigned_unsigned_same_len(lhs: &mut [Digit], rhs: &[Digit]) -> bool {
    assert_eq!(lhs.len(), rhs.len());
    let mut carry = false;
    for (l, r) in lhs.iter_mut().zip(rhs) {
        let (sum, new_carry) = l.carrying_add(*r, carry);
        *l = sum;
        carry = new_carry;
    }
    carry
}

/// Adds `rhs` to `lhs` in place, returning the carry out of the most-significant digit.
///
/// `rhs` must not be longer than `lhs`.
///
/// # Panics
///
/// Panics if `rhs` is longer than `lhs`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, add_unsigned_unsigned};
/// let mut a = [Digit::MAX, Digit::from(2u8)];
/// let carry = add_unsigned_unsigned(&mut a, &[Digit::from(1u8)]);
/// assert_eq!(a, [Digit::ZERO, Digit::from(3u8)]);
/// assert!(!carry);
/// ```
pub fn add_unsigned_unsigned(lhs: &mut [Digit], rhs: &[Digit]) -> bool {
    let (low, high) = lhs.split_at_mut(rhs.len());
    let carry = add_unsigned_unsigned_same_len(low, rhs);
    add_unsigned_carry(high, carry)
}

/// Adds a single digit to the non-empty `lhs` in place, returning the carry out of the
/// most-significant digit.
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, add_unsigned_digit};
/// let mut a = [Digit::MAX, Digit::from(7u8)];
/// let carry = add_unsigned_digit(&mut a, Digit::from(1u8));
/// assert_eq!(a, [Digit::ZERO, Digit::from(8u8)]);
/// assert!(!carry);
/// ```
pub fn add_unsigned_digit(lhs: &mut [Digit], rhs: Digit) -> bool {
    let (low, high) = lhs.split_first_mut().expect("lhs is empty");
    let (sum, carry) = low.overflowing_add(rhs);
    *low = sum;
    add_unsigned_carry(high, carry)
}

/// Adds 1 to `lhs` in place, returning the carry out of the most-significant digit (`true`
/// exactly when `lhs` is all ones).
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, add_unsigned_1};
/// let mut a = [Digit::MAX, Digit::ZERO];
/// let carry = add_unsigned_1(&mut a);
/// assert_eq!(a, [Digit::ZERO, Digit::from(1u8)]);
/// assert!(!carry);
/// ```
pub fn add_unsigned_1(lhs: &mut [Digit]) -> bool {
    for d in lhs.iter_mut() {
        let (sum, overflow) = d.overflowing_add(Digit::from(1u8));
        *d = sum;
        if !overflow {
            return false;
        }
    }
    true
}

/// Adds a carry (0 or 1) to `lhs` in place, returning the carry out of the most-significant
/// digit.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, add_unsigned_carry};
/// let mut a = [Digit::MAX, Digit::ZERO];
/// let carry = add_unsigned_carry(&mut a, true);
/// assert_eq!(a, [Digit::ZERO, Digit::from(1u8)]);
/// assert!(!carry);
///
/// // Without an incoming carry nothing changes.
/// let mut a = [Digit::MAX];
/// assert!(!add_unsigned_carry(&mut a, false));
/// ```
pub fn add_unsigned_carry(lhs: &mut [Digit], carry: bool) -> bool {
    carry && add_unsigned_1(lhs)
}

/// Adds a signed carry (-1, 0, or 1) to unsigned `lhs` in place, returning the carry out of the
/// most-significant digit (-1, 0, or 1).
///
/// # Panics
///
/// Panics if `carry` is not -1, 0, or 1.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_unsigned_icarry};
/// // Adding -1 borrows through the low zero digit.
/// let mut a = [Digit::ZERO, Digit::from(1u8)];
/// assert_eq!(add_unsigned_icarry(&mut a, IDigit::from(-1i8)), IDigit::ZERO);
/// assert_eq!(a, [Digit::MAX, Digit::ZERO]);
///
/// // Adding +1 to all ones carries out.
/// let mut a = [Digit::MAX];
/// assert_eq!(add_unsigned_icarry(&mut a, IDigit::from(1i8)), IDigit::from(1i8));
/// assert_eq!(a, [Digit::ZERO]);
/// ```
pub fn add_unsigned_icarry(lhs: &mut [Digit], carry: IDigit) -> IDigit {
    if carry == IDigit::from(-1i8) {
        -IDigit::from(sub_unsigned_1(lhs))
    } else if carry == IDigit::ZERO {
        IDigit::ZERO
    } else if carry == IDigit::from(1i8) {
        IDigit::from(add_unsigned_1(lhs))
    } else {
        panic!("invalid signed carry: {carry}")
    }
}

/// Adds the signed `rhs` to the unsigned `lhs` in place, returning a signed carry (-1, 0, or 1).
/// The `(lhs.len() + 1)`-digit signed number formed by the new `lhs` digits followed by the
/// returned carry equals the original (unsigned) `lhs` plus the (signed) `rhs`.
///
/// `rhs` must be non-empty and not longer than `lhs`.
///
/// # Panics
///
/// Panics if `rhs` is empty or longer than `lhs`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_unsigned_signed};
/// // 5 + -1 == 4, no carry.
/// let mut a = [Digit::from(5u8)];
/// assert_eq!(add_unsigned_signed(&mut a, &[Digit::MAX]), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(4u8)]);
///
/// // 0 + -1 == -1: the result is negative, so the carry is -1.
/// let mut a = [Digit::ZERO];
/// assert_eq!(add_unsigned_signed(&mut a, &[Digit::MAX]), IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX]);
/// ```
pub fn add_unsigned_signed(lhs: &mut [Digit], rhs: &[Digit]) -> IDigit {
    let rhs_extension = sign_extension(rhs);
    let (low, high) = lhs.split_at_mut(rhs.len());
    let low_carry = IDigit::from(add_unsigned_unsigned_same_len(low, rhs)) + rhs_extension;
    add_unsigned_icarry(high, low_carry)
}

/// Adds the signed digit `rhs` to the non-empty unsigned `lhs` in place, returning a signed
/// carry (-1, 0, or 1).
///
/// This is the single-digit form of [`add_unsigned_signed`].
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_unsigned_idigit};
/// // 5 + -3 == 2
/// let mut a = [Digit::from(5u8)];
/// assert_eq!(add_unsigned_idigit(&mut a, IDigit::from(-3i8)), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(2u8)]);
///
/// // 0 + -1 == -1, a borrow out of the most-significant digit.
/// let mut a = [Digit::ZERO];
/// assert_eq!(add_unsigned_idigit(&mut a, IDigit::from(-1i8)), IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX]);
/// ```
pub fn add_unsigned_idigit(lhs: &mut [Digit], rhs: IDigit) -> IDigit {
    let (low, high) = lhs.split_first_mut().expect("lhs is empty");
    let (sum, carry) = low.overflowing_add(rhs.cast_unsigned());
    *low = sum;
    let low_carry = IDigit::from(carry) + sign_extension_idigit(rhs);
    add_unsigned_icarry(high, low_carry)
}

/// Adds the signed digit `rhs` to the unsigned digit `lhs`, returning the low digit and the
/// signed carry (-1, 0, or 1) above it.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_digit_idigit};
/// // 5 + -3 == 2, no carry.
/// assert_eq!(
///     add_digit_idigit(Digit::from(5u8), IDigit::from(-3i8)),
///     (Digit::from(2u8), IDigit::ZERO)
/// );
///
/// // 0 + -1 == -1: the low digit wraps and the carry is -1.
/// assert_eq!(
///     add_digit_idigit(Digit::ZERO, IDigit::from(-1i8)),
///     (Digit::MAX, IDigit::from(-1i8))
/// );
///
/// // (2^bits - 1) + 1 carries out into a +1.
/// assert_eq!(
///     add_digit_idigit(Digit::MAX, IDigit::from(1i8)),
///     (Digit::ZERO, IDigit::from(1i8))
/// );
/// ```
pub fn add_digit_idigit(lhs: Digit, rhs: IDigit) -> (Digit, IDigit) {
    let (low, carry) = lhs.overflowing_add(rhs.cast_unsigned());
    let icarry = sign_extension_idigit(rhs) + IDigit::from(carry);
    (low, icarry)
}

/// Adds the unsigned `rhs` to the signed `lhs` in place, returning the signed carry (-1, 0, or
/// 1).
///
/// `lhs` must be non-empty and `rhs` must not be longer than `lhs`.
///
/// # Panics
///
/// Panics if `lhs` is empty or `rhs` is longer than `lhs`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_signed_unsigned};
/// // -1 + 5 == 4.
/// let mut a = [Digit::MAX]; // -1
/// assert_eq!(add_signed_unsigned(&mut a, &[Digit::from(5u8)]), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(4u8)]);
///
/// // A large positive value plus a large unsigned value overflows to a +1 top digit.
/// let mut a = [Digit::MAX >> 1]; // the largest positive single digit
/// assert_eq!(add_signed_unsigned(&mut a, &[Digit::MAX]), IDigit::from(1i8));
/// assert_eq!(a, [(Digit::MAX >> 1) - Digit::from(1u8)]);
/// ```
pub fn add_signed_unsigned(lhs: &mut [Digit], rhs: &[Digit]) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    IDigit::from(add_unsigned_unsigned(lhs, rhs)) + lhs_extension
}

/// Adds the signed `rhs` to the signed `lhs` in place, returning the signed carry (0 or -1).
///
/// `rhs` must be non-empty and not longer than `lhs`.
///
/// # Panics
///
/// Panics if `rhs` is empty or longer than `lhs`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_signed_signed};
/// // -1 + -1 == -2
/// let mut a = [Digit::MAX];
/// let high = add_signed_signed(&mut a, &[Digit::MAX]);
/// assert_eq!(a, [Digit::MAX - Digit::from(1u8)]);
/// assert_eq!(high, IDigit::from(-1i8));
/// ```
pub fn add_signed_signed(lhs: &mut [Digit], rhs: &[Digit]) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    add_unsigned_signed(lhs, rhs) + lhs_extension
}

/// Adds the unsigned digit `rhs` to the non-empty signed `lhs` in place, returning the signed
/// carry (-1, 0, or 1).
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_signed_digit};
/// // -1 + 5 == 4
/// let mut a = [Digit::MAX]; // -1
/// assert_eq!(add_signed_digit(&mut a, Digit::from(5u8)), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(4u8)]);
/// ```
pub fn add_signed_digit(lhs: &mut [Digit], rhs: Digit) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    IDigit::from(add_unsigned_digit(lhs, rhs)) + lhs_extension
}

/// Adds the signed digit `rhs` to the non-empty signed `lhs` in place, returning the signed
/// carry (0 or -1).
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_signed_idigit};
/// // -1 + -1 == -2
/// let mut a = [Digit::MAX];
/// let high = add_signed_idigit(&mut a, IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX - Digit::from(1u8)]);
/// assert_eq!(high, IDigit::from(-1i8));
/// ```
pub fn add_signed_idigit(lhs: &mut [Digit], rhs: IDigit) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    add_unsigned_idigit(lhs, rhs) + lhs_extension
}

/// Adds a signed carry (-1, 0, or 1) to the non-empty signed `lhs` in place, returning the
/// signed carry (0 or -1).
///
/// # Panics
///
/// Panics if `lhs` is empty, or if `carry` is not -1, 0, or 1.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_signed_icarry};
/// // -1 + 1 == 0.
/// let mut a = [Digit::MAX];
/// assert_eq!(add_signed_icarry(&mut a, IDigit::from(1i8)), IDigit::ZERO);
/// assert_eq!(a, [Digit::ZERO]);
///
/// // 0 + -1 == -1, borrowing out the top.
/// let mut a = [Digit::ZERO, Digit::ZERO];
/// assert_eq!(add_signed_icarry(&mut a, IDigit::from(-1i8)), IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX, Digit::MAX]);
/// ```
pub fn add_signed_icarry(lhs: &mut [Digit], carry: IDigit) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    add_unsigned_icarry(lhs, carry) + lhs_extension
}

/// Adds the signed digit `rhs` to the signed digit `lhs`, returning the low digit and the
/// signed carry (0 or -1) above it.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, add_idigit_idigit};
/// // 2 + 3 == 5, no carry.
/// assert_eq!(
///     add_idigit_idigit(IDigit::from(2i8), IDigit::from(3i8)),
///     (Digit::from(5u8), IDigit::ZERO)
/// );
///
/// // -1 + -1 == -2: the low digit wraps and the carry is -1.
/// assert_eq!(
///     add_idigit_idigit(IDigit::from(-1i8), IDigit::from(-1i8)),
///     (Digit::MAX - Digit::from(1u8), IDigit::from(-1i8))
/// );
///
/// // The two most-negative digits sum to -2^bits: low digit 0, carry -1.
/// assert_eq!(
///     add_idigit_idigit(IDigit::MIN, IDigit::MIN),
///     (Digit::ZERO, IDigit::from(-1i8))
/// );
/// ```
pub fn add_idigit_idigit(lhs: IDigit, rhs: IDigit) -> (Digit, IDigit) {
    let (low, carry) = lhs.cast_unsigned().overflowing_add(rhs.cast_unsigned());
    let icarry = sign_extension_idigit(lhs) + sign_extension_idigit(rhs) + IDigit::from(carry);
    (low, icarry)
}

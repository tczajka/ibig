//! Subtraction.

use crate::add::add_unsigned_icarry;
use crate::{Digit, IDigit, not, sign_extension, sign_extension_idigit};

/// Subtracts `rhs` from `lhs` in place, returning the borrow out of the most-significant digit.
/// The slices must have the same length.
///
/// # Panics
///
/// Panics if `lhs` and `rhs` have different lengths.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, sub_unsigned_unsigned_same_len};
/// let mut a = [Digit::ZERO, Digit::ZERO];
/// let borrow = sub_unsigned_unsigned_same_len(&mut a, &[Digit::from(1u8), Digit::ZERO]);
/// assert_eq!(a, [Digit::MAX, Digit::MAX]);
/// assert!(borrow);
/// ```
pub fn sub_unsigned_unsigned_same_len(lhs: &mut [Digit], rhs: &[Digit]) -> bool {
    assert_eq!(lhs.len(), rhs.len());
    let mut borrow = false;
    for (l, r) in lhs.iter_mut().zip(rhs) {
        let (diff, new_borrow) = l.borrowing_sub(*r, borrow);
        *l = diff;
        borrow = new_borrow;
    }
    borrow
}

/// Subtracts `rhs` from `lhs` in place, returning the borrow out of the most-significant digit.
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
/// # use ibig_core::{Digit, sub_unsigned_unsigned};
/// let mut a = [Digit::ZERO, Digit::from(3u8)];
/// let borrow = sub_unsigned_unsigned(&mut a, &[Digit::from(1u8)]);
/// assert_eq!(a, [Digit::MAX, Digit::from(2u8)]);
/// assert!(!borrow);
/// ```
pub fn sub_unsigned_unsigned(lhs: &mut [Digit], rhs: &[Digit]) -> bool {
    let (low, high) = lhs.split_at_mut(rhs.len());
    let borrow = sub_unsigned_unsigned_same_len(low, rhs);
    sub_unsigned_borrow(high, borrow)
}

/// Subtracts a single digit from the non-empty `lhs` in place, returning the borrow out of the
/// most-significant digit.
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, sub_unsigned_digit};
/// let mut a = [Digit::ZERO, Digit::from(8u8)];
/// let borrow = sub_unsigned_digit(&mut a, Digit::from(1u8));
/// assert_eq!(a, [Digit::MAX, Digit::from(7u8)]);
/// assert!(!borrow);
/// ```
pub fn sub_unsigned_digit(lhs: &mut [Digit], rhs: Digit) -> bool {
    let (low, high) = lhs.split_first_mut().expect("lhs is empty");
    let (diff, borrow) = low.overflowing_sub(rhs);
    *low = diff;
    sub_unsigned_borrow(high, borrow)
}

/// Subtracts a borrow (0 or 1) from `lhs` in place, returning the borrow out of the
/// most-significant digit.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, sub_unsigned_borrow};
/// let mut a = [Digit::ZERO, Digit::from(1u8)];
/// let borrow = sub_unsigned_borrow(&mut a, true);
/// assert_eq!(a, [Digit::MAX, Digit::ZERO]);
/// assert!(!borrow);
///
/// // Without an incoming borrow nothing changes.
/// let mut a = [Digit::ZERO];
/// assert!(!sub_unsigned_borrow(&mut a, false));
/// ```
pub fn sub_unsigned_borrow(lhs: &mut [Digit], borrow: bool) -> bool {
    borrow && sub_unsigned_1(lhs)
}

/// Subtracts 1 from `lhs` in place, returning the borrow out of the most-significant digit
/// (`true` exactly when `lhs` is all zeros).
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, sub_unsigned_1};
/// let mut a = [Digit::ZERO, Digit::from(1u8)];
/// let borrow = sub_unsigned_1(&mut a);
/// assert_eq!(a, [Digit::MAX, Digit::ZERO]);
/// assert!(!borrow);
/// ```
pub fn sub_unsigned_1(lhs: &mut [Digit]) -> bool {
    for d in lhs.iter_mut() {
        let (diff, overflow) = d.overflowing_sub(Digit::from(1u8));
        *d = diff;
        if !overflow {
            return false;
        }
    }
    true
}

/// Subtracts the signed `rhs` from the unsigned `lhs` in place, returning the signed carry
/// (-1, 0, or 1).
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
/// # use ibig_core::{Digit, IDigit, sub_unsigned_signed};
/// // 5 - -1 == 6, no carry.
/// let mut a = [Digit::from(5u8)];
/// assert_eq!(sub_unsigned_signed(&mut a, &[Digit::MAX]), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(6u8)]);
///
/// // 0 - 1 == -1: the result is negative, so the carry is -1.
/// let mut a = [Digit::ZERO];
/// assert_eq!(sub_unsigned_signed(&mut a, &[Digit::from(1u8)]), IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX]);
/// ```
pub fn sub_unsigned_signed(lhs: &mut [Digit], rhs: &[Digit]) -> IDigit {
    let rhs_extension = sign_extension(rhs);
    let (low, high) = lhs.split_at_mut(rhs.len());
    let low_borrow = sub_unsigned_unsigned_same_len(low, rhs);
    let low_carry = -IDigit::from(low_borrow) - rhs_extension; // -1..=1
    add_unsigned_icarry(high, low_carry)
}

/// Subtracts the signed digit `rhs` from the non-empty unsigned `lhs` in place, returning the
/// signed carry (-1, 0, or 1).
///
/// This is the single-digit form of [`sub_unsigned_signed`].
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, sub_unsigned_idigit};
/// // 5 - -3 == 8
/// let mut a = [Digit::from(5u8)];
/// assert_eq!(sub_unsigned_idigit(&mut a, IDigit::from(-3i8)), IDigit::ZERO);
/// assert_eq!(a, [Digit::from(8u8)]);
///
/// // 0 - 1 == -1, a borrow out of the most-significant digit.
/// let mut a = [Digit::ZERO];
/// assert_eq!(sub_unsigned_idigit(&mut a, IDigit::from(1i8)), IDigit::from(-1i8));
/// assert_eq!(a, [Digit::MAX]);
/// ```
pub fn sub_unsigned_idigit(lhs: &mut [Digit], rhs: IDigit) -> IDigit {
    let (low, high) = lhs.split_first_mut().expect("lhs is empty");
    let (diff, borrow) = low.overflowing_sub(rhs.cast_unsigned());
    *low = diff;
    let low_carry = -IDigit::from(borrow) - sign_extension_idigit(rhs); // -1..=1
    add_unsigned_icarry(high, low_carry)
}

/// Subtracts the signed digit `rhs` from the unsigned digit `lhs`, returning the low digit and
/// the signed carry (-1, 0, or 1) above it.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, sub_digit_idigit};
/// // 5 - 3 == 2, no carry.
/// assert_eq!(
///     sub_digit_idigit(Digit::from(5u8), IDigit::from(3i8)),
///     (Digit::from(2u8), IDigit::ZERO)
/// );
///
/// // 0 - 1 == -1: the low digit wraps and the carry is -1.
/// assert_eq!(
///     sub_digit_idigit(Digit::ZERO, IDigit::from(1i8)),
///     (Digit::MAX, IDigit::from(-1i8))
/// );
///
/// // (2^bits - 1) - -1 == 2^bits carries out into a +1.
/// assert_eq!(
///     sub_digit_idigit(Digit::MAX, IDigit::from(-1i8)),
///     (Digit::ZERO, IDigit::from(1i8))
/// );
/// ```
pub fn sub_digit_idigit(lhs: Digit, rhs: IDigit) -> (Digit, IDigit) {
    let (low, borrow) = lhs.overflowing_sub(rhs.cast_unsigned());
    let icarry = -IDigit::from(borrow) - sign_extension_idigit(rhs);
    (low, icarry)
}

/// Subtracts the signed `rhs` from the signed `lhs` in place, returning the signed carry
/// (0 or -1).
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
/// # use ibig_core::{Digit, IDigit, sub_signed_signed};
/// // 3 - 5 == -2
/// let mut a = [Digit::from(3u8)];
/// let high = sub_signed_signed(&mut a, &[Digit::from(5u8)]);
/// assert_eq!(a, [Digit::MAX - Digit::from(1u8)]);
/// assert_eq!(high, IDigit::from(-1i8));
/// ```
pub fn sub_signed_signed(lhs: &mut [Digit], rhs: &[Digit]) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    sub_unsigned_signed(lhs, rhs) + lhs_extension
}

/// Subtracts the signed digit `rhs` from the non-empty signed `lhs` in place, returning the
/// signed carry (0 or -1).
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, sub_signed_idigit};
/// // 3 - 5 == -2
/// let mut a = [Digit::from(3u8)];
/// let high = sub_signed_idigit(&mut a, IDigit::from(5i8));
/// assert_eq!(a, [Digit::MAX - Digit::from(1u8)]);
/// assert_eq!(high, IDigit::from(-1i8));
/// ```
pub fn sub_signed_idigit(lhs: &mut [Digit], rhs: IDigit) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    sub_unsigned_idigit(lhs, rhs) + lhs_extension
}

/// Subtracts the signed digit `rhs` from the signed digit `lhs`, returning the low digit and the
/// signed carry (0 or -1) above it.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, sub_idigit_idigit};
/// // 5 - 3 == 2, no carry.
/// assert_eq!(
///     sub_idigit_idigit(IDigit::from(5i8), IDigit::from(3i8)),
///     (Digit::from(2u8), IDigit::ZERO)
/// );
///
/// // 3 - 5 == -2: the low digit wraps and the carry is -1.
/// assert_eq!(
///     sub_idigit_idigit(IDigit::from(3i8), IDigit::from(5i8)),
///     (Digit::MAX - Digit::from(1u8), IDigit::from(-1i8))
/// );
///
/// // The most-negative minus the most-positive: MIN - MAX == -2^bits + 1.
/// assert_eq!(
///     sub_idigit_idigit(IDigit::MIN, IDigit::MAX),
///     (Digit::from(1u8), IDigit::from(-1i8))
/// );
/// ```
pub fn sub_idigit_idigit(lhs: IDigit, rhs: IDigit) -> (Digit, IDigit) {
    let (low, borrow) = lhs.cast_unsigned().overflowing_sub(rhs.cast_unsigned());
    let icarry = sign_extension_idigit(lhs) - sign_extension_idigit(rhs) - IDigit::from(borrow);
    (low, icarry)
}

/// Assigns `lhs = rhs - lhs` in place, returning the borrow out of the most-significant digit.
/// The slices must have the same length.
///
/// # Panics
///
/// Panics if `lhs` and `rhs` have different lengths.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, sub_rev_unsigned_unsigned_same_len};
/// // 5 - 3 stored back into the first slice.
/// let mut a = [Digit::from(3u8)];
/// let borrow = sub_rev_unsigned_unsigned_same_len(&mut a, &[Digit::from(5u8)]);
/// assert_eq!(a, [Digit::from(2u8)]);
/// assert!(!borrow);
/// ```
pub fn sub_rev_unsigned_unsigned_same_len(lhs: &mut [Digit], rhs: &[Digit]) -> bool {
    assert_eq!(lhs.len(), rhs.len());
    let mut borrow = false;
    for (l, r) in lhs.iter_mut().zip(rhs) {
        let (diff, new_borrow) = r.borrowing_sub(*l, borrow);
        *l = diff;
        borrow = new_borrow;
    }
    borrow
}

/// Subtracts the signed `lhs` from the signed `rhs`, assigning `lhs = rhs - lhs` in place and
/// returning the signed carry (0 or -1).
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
/// # use ibig_core::{Digit, IDigit, sub_rev_signed_signed};
/// // 5 - 3 == 2
/// let mut a = [Digit::from(3u8)];
/// let high = sub_rev_signed_signed(&mut a, &[Digit::from(5u8)]);
/// assert_eq!(a, [Digit::from(2u8)]);
/// assert_eq!(high, IDigit::ZERO);
/// ```
pub fn sub_rev_signed_signed(lhs: &mut [Digit], rhs: &[Digit]) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    let rhs_extension = sign_extension(rhs);
    let (low, high) = lhs.split_at_mut(rhs.len());
    let low_borrow = sub_rev_unsigned_unsigned_same_len(low, rhs);
    let low_carry = rhs_extension - IDigit::from(low_borrow); // -2..=0
    // negate the top part: -x = !x + 1
    not(high);
    let low_carry = low_carry + IDigit::from(1u8); // -1..=1
    let high_carry = add_unsigned_icarry(high, low_carry);
    !lhs_extension + high_carry
}

/// Subtracts the signed `lhs` from the signed digit `rhs`, assigning `lhs = rhs - lhs` in the
/// non-empty `lhs` in place and returning the signed carry (0 or -1).
///
/// # Panics
///
/// Panics if `lhs` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, IDigit, sub_rev_signed_idigit};
/// // 5 - 3 == 2
/// let mut a = [Digit::from(3u8)];
/// let high = sub_rev_signed_idigit(&mut a, IDigit::from(5i8));
/// assert_eq!(a, [Digit::from(2u8)]);
/// assert_eq!(high, IDigit::ZERO);
/// ```
pub fn sub_rev_signed_idigit(lhs: &mut [Digit], rhs: IDigit) -> IDigit {
    let lhs_extension = sign_extension(lhs);
    let rhs_extension = sign_extension_idigit(rhs);
    let (low, high) = lhs.split_first_mut().expect("lhs is empty");
    let (diff, low_borrow) = rhs.cast_unsigned().overflowing_sub(*low);
    *low = diff;
    let low_carry = rhs_extension - IDigit::from(low_borrow); // -2..=0
    // negate the top part: -x = !x + 1
    not(high);
    let low_carry = low_carry + IDigit::from(1u8); // -1..=1
    let high_carry = add_unsigned_icarry(high, low_carry);
    !lhs_extension + high_carry
}

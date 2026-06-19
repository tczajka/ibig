//! Midpoint.

use crate::ops::{BinaryOpRef, CommutativeBinaryOpRefBigBig};
use crate::repr::Digits;
use crate::{IBig, UBig};
use ibig_core::{Digit, IDigit};

impl UBig {
    /// Returns the midpoint of `self` and `rhs`.
    ///
    /// This is `(self + rhs) / 2`, rounded toward zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from(4u8).midpoint(&UBig::from(8u8)), UBig::from(6u8));
    /// // An odd sum is rounded down.
    /// assert_eq!(UBig::from(4u8).midpoint(&UBig::from(7u8)), UBig::from(5u8));
    /// ```
    pub fn midpoint(&self, rhs: &UBig) -> UBig {
        <MidpointUBigUBig as BinaryOpRef>::apply_ref_ref(self, rhs)
    }
}

impl IBig {
    /// Returns the midpoint of `self` and `rhs`.
    ///
    /// This is `(self + rhs) / 2`, rounded toward zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from(4).midpoint(&IBig::from(8)), IBig::from(6));
    /// // An odd sum is rounded toward zero.
    /// assert_eq!(IBig::from(4).midpoint(&IBig::from(7)), IBig::from(5));
    /// assert_eq!(IBig::from(-4).midpoint(&IBig::from(-7)), IBig::from(-5));
    /// ```
    pub fn midpoint(&self, rhs: &IBig) -> IBig {
        <MidpointIBigIBig as BinaryOpRef>::apply_ref_ref(self, rhs)
    }
}

/// The [`UBig::midpoint`] operation.
enum MidpointUBigUBig {}

impl CommutativeBinaryOpRefBigBig for MidpointUBigUBig {
    type Operand = UBig;
    type Output = UBig;

    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        UBig::from_digit(lhs.midpoint(rhs))
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        let carry = ibig_core::add_unsigned_digit(&mut digits, rhs);
        digits.push(carry.into());
        ibig_core::shr_small_unsigned(&mut digits, 1);
        UBig::from_digits(digits)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        // Clone the longer operand with room for a carry digit, add the shorter one, then halve.
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        let mut digits = Digits::with_capacity(longer.len() + 1);
        digits.extend_from_slice(longer);
        let carry = ibig_core::add_unsigned_unsigned(&mut digits, shorter);
        digits.push(carry.into());
        ibig_core::shr_small_unsigned(&mut digits, 1);
        UBig::from_digits(digits)
    }
}

/// The [`IBig::midpoint`] operation.
enum MidpointIBigIBig {}

impl CommutativeBinaryOpRefBigBig for MidpointIBigIBig {
    type Operand = IBig;
    type Output = IBig;

    fn apply_digit_digit(lhs: IDigit, rhs: IDigit) -> IBig {
        IBig::from_digit(lhs.midpoint(rhs))
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: IDigit) -> IBig {
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        let icarry = ibig_core::add_signed_idigit(&mut digits, rhs);
        digits.push(icarry.cast_unsigned());
        let is_negative = ibig_core::is_negative(&digits);
        ibig_core::add_unsigned_carry(&mut digits, is_negative);
        ibig_core::shr_small_signed(&mut digits, 1);
        IBig::from_digits(digits)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        let mut digits = Digits::with_capacity(longer.len() + 1);
        digits.extend_from_slice(longer);
        let icarry = ibig_core::add_signed_signed(&mut digits, shorter);
        digits.push(icarry.cast_unsigned());
        let is_negative = ibig_core::is_negative(&digits);
        ibig_core::add_unsigned_carry(&mut digits, is_negative);
        ibig_core::shr_small_signed(&mut digits, 1);
        IBig::from_digits(digits)
    }
}

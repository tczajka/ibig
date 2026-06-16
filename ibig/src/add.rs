//! Addition.

use crate::ops::{
    BigBig, BinaryOpRef, BinaryOpRefBigBig, CommutativeBinaryOpRefValBigBig, impl_binary_operator,
};
use crate::repr::Digits;
use crate::{IBig, UBig};
use core::ops::{Add, AddAssign};
use ibig_core::{Digit, IDigit, sign_extension, sign_extension_idigit};

impl UBig {
    /// Adds the signed `rhs` to `self`, returning `None` if the result would be negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(UBig::from(5u8).checked_add_signed(&IBig::from(3)), Some(UBig::from(8u8)));
    /// assert_eq!(UBig::from(5u8).checked_add_signed(&IBig::from(-3)), Some(UBig::from(2u8)));
    /// assert_eq!(UBig::from(5u8).checked_add_signed(&IBig::from(-8)), None);
    /// ```
    pub fn checked_add_signed(&self, rhs: &IBig) -> Option<UBig> {
        <CheckedAddUBigIBig as BinaryOpRef>::apply_ref_ref(self, rhs)
    }

    /// Adds the signed `rhs` to `self`.
    ///
    /// # Panics
    ///
    /// Panics if the result would be negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(UBig::from(5u8).strict_add_signed(&IBig::from(-3)), UBig::from(2u8));
    /// ```
    pub fn strict_add_signed(&self, rhs: &IBig) -> UBig {
        self.checked_add_signed(rhs)
            .unwrap_or_else(|| UBig::panic_negative())
    }

    /// Adds the signed `rhs` to `self`, saturating at zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(UBig::from(5u8).saturating_add_signed(&IBig::from(-3)), UBig::from(2u8));
    /// assert_eq!(UBig::from(5u8).saturating_add_signed(&IBig::from(-8)), UBig::ZERO);
    /// ```
    pub fn saturating_add_signed(&self, rhs: &IBig) -> UBig {
        self.checked_add_signed(rhs).unwrap_or(UBig::ZERO)
    }
}

impl IBig {
    /// Adds the unsigned `rhs` to `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(IBig::from(5).add_unsigned(&UBig::from(3u8)), IBig::from(8));
    /// assert_eq!(IBig::from(-5).add_unsigned(&UBig::from(3u8)), IBig::from(-2));
    /// ```
    pub fn add_unsigned(&self, rhs: &UBig) -> IBig {
        <AddIBigUBig as BinaryOpRef>::apply_ref_ref(self, rhs)
    }
}

/// Addition operation for [`UBig`].
enum AddUBigUBig {}

impl CommutativeBinaryOpRefValBigBig for AddUBigUBig {
    type Operand = UBig;
    type Output = UBig;

    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        let (sum, carry) = lhs.overflowing_add(rhs);
        UBig::from_two_digits(sum, carry.into())
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        // Clone with room for a possible carry digit.
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        Self::apply_val_digit(digits, rhs)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: Digit) -> UBig {
        let carry = ibig_core::add_unsigned_digit(&mut lhs, rhs);
        UBig::from_digits_carry(lhs, carry)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        // Clone the longer operand, with room for a possible carry digit, and add
        // the shorter one to it.
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        let mut digits = Digits::with_capacity(longer.len() + 1);
        digits.extend_from_slice(longer);
        let carry = ibig_core::add_unsigned_unsigned(&mut digits, shorter);
        UBig::from_digits_carry(digits, carry)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> UBig {
        let carry = if lhs.len() >= rhs.len() {
            ibig_core::add_unsigned_unsigned(&mut lhs, rhs)
        } else {
            // Add the overlapping low digits, then append the high digits of `rhs` and
            // propagate the carry through them. Reserve for the appended digits and a
            // possible carry digit.
            let lhs_len = lhs.len();
            lhs.reserve(rhs.len() - lhs_len + 1);
            let (rhs_low, rhs_high) = rhs.split_at(lhs_len);
            let low_carry = ibig_core::add_unsigned_unsigned_same_len(&mut lhs, rhs_low);
            lhs.extend_from_slice(rhs_high);
            ibig_core::add_unsigned_carry(&mut lhs[lhs_len..], low_carry)
        };
        UBig::from_digits_carry(lhs, carry)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> UBig {
        // Reuse storage from the longer operand.
        let (mut longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        let carry = ibig_core::add_unsigned_unsigned(&mut longer, &shorter);
        UBig::from_digits_carry(longer, carry)
    }
}

impl_binary_operator!(
    Add::add(UBig, UBig) -> UBig,
    AddAssign::add_assign,
    BigBig<AddUBigUBig>
);

/// The [`UBig::checked_add_signed`] operation.
enum CheckedAddUBigIBig {}

impl BinaryOpRefBigBig for CheckedAddUBigIBig {
    type Left = UBig;
    type Right = IBig;
    type Output = Option<UBig>;

    fn apply_digit_digit(lhs: Digit, rhs: IDigit) -> Option<UBig> {
        let (sum, overflow) = lhs.overflowing_add_signed(rhs);
        if !overflow {
            Some(UBig::from_digit(sum))
        } else if rhs.is_negative() {
            // The result is negative.
            None
        } else {
            // The sum overflowed a single digit into a most-significant `1`.
            Some(UBig::from_two_digits(sum, Digit::from(1u8)))
        }
    }

    fn apply_digit_ref(lhs: Digit, rhs: &[Digit]) -> Option<UBig> {
        // `rhs` (at least two digits) is longer than the single digit. If it is at least two
        // digits longer and negative, its magnitude exceeds `lhs`, so the result is negative.
        if rhs.len() >= 3 && ibig_core::is_negative(rhs) {
            return None;
        }
        // Clone the signed `rhs` (the longer operand) and add the unsigned digit `lhs`.
        let mut digits = Digits::with_capacity(rhs.len() + 1);
        digits.extend_from_slice(rhs);
        let icarry = ibig_core::add_signed_digit(&mut digits, lhs);
        UBig::try_from_digits_icarry(digits, icarry)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: IDigit) -> Option<UBig> {
        // Clone the unsigned `lhs` (the longer operand) and add the signed digit `rhs`.
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        let icarry = ibig_core::add_unsigned_idigit(&mut digits, rhs);
        UBig::try_from_digits_icarry(digits, icarry)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Option<UBig> {
        let mut digits;
        let icarry;
        if lhs.len() >= rhs.len() {
            // Clone the unsigned `lhs` (the longer operand) and add the signed `rhs`.
            digits = Digits::with_capacity(lhs.len() + 1);
            digits.extend_from_slice(lhs);
            icarry = ibig_core::add_unsigned_signed(&mut digits, rhs);
        } else {
            // `rhs` is longer. If it is at least two digits longer and negative, its magnitude
            // exceeds any `lhs` of `lhs.len()` digits, so the result is certainly negative.
            if rhs.len() >= lhs.len() + 2 && ibig_core::is_negative(rhs) {
                return None;
            }
            // Clone the signed `rhs` (the longer operand) and add the unsigned `lhs`.
            digits = Digits::with_capacity(rhs.len() + 1);
            digits.extend_from_slice(rhs);
            icarry = ibig_core::add_signed_unsigned(&mut digits, lhs);
        }
        UBig::try_from_digits_icarry(digits, icarry)
    }
}

/// The [`IBig::add_unsigned`] operation.
enum AddIBigUBig {}

impl BinaryOpRefBigBig for AddIBigUBig {
    type Left = IBig;
    type Right = UBig;
    type Output = IBig;

    fn apply_digit_digit(lhs: IDigit, rhs: Digit) -> IBig {
        let (sum, carry) = lhs.cast_unsigned().overflowing_add(rhs);
        IBig::from_two_digits(sum, IDigit::from(carry) + sign_extension_idigit(lhs))
    }

    fn apply_digit_ref(lhs: IDigit, rhs: &[Digit]) -> IBig {
        // Clone the unsigned `rhs` (the longer operand) and add the signed digit `lhs`.
        let mut digits = Digits::with_capacity(rhs.len() + 1);
        digits.extend_from_slice(rhs);
        let icarry = ibig_core::add_unsigned_idigit(&mut digits, lhs);
        IBig::from_digits_icarry(digits, icarry)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> IBig {
        // Clone the signed `lhs` (the longer operand) and add the unsigned digit `rhs`.
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        let icarry = ibig_core::add_signed_digit(&mut digits, rhs);
        IBig::from_digits_icarry(digits, icarry)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        let mut digits;
        let icarry;
        if lhs.len() >= rhs.len() {
            // Clone the signed `lhs` (the longer operand) and add the unsigned `rhs`.
            digits = Digits::with_capacity(lhs.len() + 1);
            digits.extend_from_slice(lhs);
            icarry = ibig_core::add_signed_unsigned(&mut digits, rhs);
        } else {
            // Clone the unsigned `rhs` (the longer operand) and add the signed `lhs`.
            digits = Digits::with_capacity(rhs.len() + 1);
            digits.extend_from_slice(rhs);
            icarry = ibig_core::add_unsigned_signed(&mut digits, lhs);
        }
        IBig::from_digits_icarry(digits, icarry)
    }
}

/// Addition operation for [`IBig`].
enum AddIBigIBig {}

impl CommutativeBinaryOpRefValBigBig for AddIBigIBig {
    type Operand = IBig;
    type Output = IBig;

    fn apply_digit_digit(lhs: IDigit, rhs: IDigit) -> IBig {
        let (sum, overflow) = lhs.overflowing_add(rhs);
        if overflow {
            // On overflow `lhs` and `rhs` share a sign, which is the sign of the two-digit result.
            IBig::from_two_digits(sum.cast_unsigned(), sign_extension_idigit(lhs))
        } else {
            IBig::from_digit(sum)
        }
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: IDigit) -> IBig {
        // Clone with room for a possible sign digit.
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        Self::apply_val_digit(digits, rhs)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: IDigit) -> IBig {
        let icarry = ibig_core::add_signed_idigit(&mut lhs, rhs);
        IBig::from_digits_icarry(lhs, icarry)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        // Clone the longer operand, with room for a possible sign digit, and add the
        // shorter one to it.
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        let mut digits = Digits::with_capacity(longer.len() + 1);
        digits.extend_from_slice(longer);
        Self::apply_val_ref(digits, shorter)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> IBig {
        let lhs_len = lhs.len();
        if lhs_len < rhs.len() {
            // Sign-extend `lhs` to the length of `rhs`. Reserve for the extension digits
            // and a possible sign digit.
            lhs.reserve(rhs.len() - lhs_len + 1);
            let fill = sign_extension(&lhs).cast_unsigned();
            lhs.resize(rhs.len(), fill);
        }
        let icarry = ibig_core::add_signed_signed(&mut lhs, rhs);
        IBig::from_digits_icarry(lhs, icarry)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> IBig {
        // Reuse storage from the longer operand.
        if lhs.len() >= rhs.len() {
            Self::apply_val_ref(lhs, &rhs)
        } else {
            Self::apply_val_ref(rhs, &lhs)
        }
    }
}

impl_binary_operator!(
    Add::add(IBig, IBig) -> IBig,
    AddAssign::add_assign,
    BigBig<AddIBigIBig>
);

//! Subtraction.

use crate::ops::{BinaryOpDigits, DigitsRhs, impl_binary_operator};
use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
    Digits,
};
use crate::{IBig, UBig};
use core::ops::{Sub, SubAssign};
use ibig_core::{Digit, SignedDigit, sign_extension, sign_extension_sdigit};

impl UBig {
    /// Subtracts `rhs` from `self`, returning `None` if the result would be negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from(5u8).checked_sub(&UBig::from(3u8)), Some(UBig::from(2u8)));
    /// assert_eq!(UBig::from(3u8).checked_sub(&UBig::from(5u8)), None);
    /// ```
    pub fn checked_sub(&self, rhs: &UBig) -> Option<UBig> {
        match (self.as_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => a.checked_sub(b).map(UBig::from_digit),
            // A multi-digit `rhs` is bigger than any single digit.
            (Small(_), Large(_)) => None,
            // A multi-digit `lhs` minus a single digit never underflows.
            (Large(lhs), Small(b)) => Some(SubOperation::apply_ref_digit(lhs, b)),
            (Large(lhs), Large(rhs)) => Self::checked_sub_large(lhs, rhs),
        }
    }

    /// Subtracts `rhs` from `self`, saturating at zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from(5u8).saturating_sub(&UBig::from(3u8)), UBig::from(2u8));
    /// assert_eq!(UBig::from(3u8).saturating_sub(&UBig::from(5u8)), UBig::ZERO);
    /// ```
    pub fn saturating_sub(&self, rhs: &UBig) -> UBig {
        self.checked_sub(rhs).unwrap_or(UBig::ZERO)
    }

    /// Checked subtraction of multi-digit values.
    fn checked_sub_large(lhs: &[Digit], rhs: &[Digit]) -> Option<UBig> {
        // A shorter `lhs` is necessarily smaller, and `ibig_core::sub_unsigned_unsigned` requires
        // `rhs` to not be longer than `lhs`.
        if lhs.len() < rhs.len() {
            return None;
        }
        let mut digits = Digits::from_slice(lhs);
        if ibig_core::sub_unsigned_unsigned(&mut digits, rhs) {
            return None;
        }
        Some(UBig::from_digits(digits))
    }
}

/// Subtraction operation.
struct SubOperation;

impl BinaryOpDigits<UBig> for SubOperation {
    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        let (diff, borrow) = lhs.overflowing_sub(rhs);
        if borrow {
            UBig::panic_negative();
        }
        UBig::from_digit(diff)
    }

    fn apply_digit_ref(_lhs: Digit, _rhs: &[Digit]) -> UBig {
        // A multi-digit `rhs` is bigger than any single digit.
        UBig::panic_negative()
    }

    fn apply_digit_val(_lhs: Digit, _rhs: Digits) -> UBig {
        // A multi-digit `rhs` is bigger than any single digit.
        UBig::panic_negative()
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        Self::apply_val_digit(Digits::from_slice(lhs), rhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        // Check the lengths before cloning a result that would only be discarded.
        if lhs.len() < rhs.len() {
            UBig::panic_negative();
        }
        Self::apply_val_ref(Digits::from_slice(lhs), rhs)
    }

    fn apply_ref_val(lhs: &[Digit], mut rhs: Digits) -> UBig {
        let rhs_len = rhs.len();
        if lhs.len() < rhs_len {
            UBig::panic_negative();
        }
        rhs.reserve(lhs.len() - rhs_len);
        let (lhs_low, lhs_high) = lhs.split_at(rhs_len);
        let borrow = ibig_core::sub_reverse_unsigned_unsigned_same_len(&mut rhs, lhs_low);
        rhs.extend_from_slice(lhs_high);
        if ibig_core::sub_unsigned_borrow(&mut rhs[rhs_len..], borrow) {
            UBig::panic_negative();
        }
        UBig::from_digits(rhs)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: Digit) -> UBig {
        // `lhs` has at least two digits, so subtracting a single digit cannot underflow.
        let borrow = ibig_core::sub_unsigned_digit(&mut lhs, rhs);
        assert!(!borrow);
        UBig::from_digits(lhs)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> UBig {
        if lhs.len() < rhs.len() || ibig_core::sub_unsigned_unsigned(&mut lhs, rhs) {
            UBig::panic_negative();
        }
        UBig::from_digits(lhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> UBig {
        // Reuse storage from `lhs`; the result is never longer than `lhs`.
        Self::apply_val_ref(lhs, &rhs)
    }
}

impl_binary_operator!(
    UBig,
    UBig,
    Sub::sub,
    SubAssign::sub_assign,
    DigitsRhs<SubOperation>
);

impl BinaryOpDigits<IBig> for SubOperation {
    fn apply_digit_digit(lhs: SignedDigit, rhs: SignedDigit) -> IBig {
        let (diff, overflow) = lhs.overflowing_sub(rhs);
        if overflow {
            // On overflow `lhs` and `rhs` have opposite signs, and the result's sign is `lhs`'s.
            IBig::from_two_digits(diff.cast_unsigned(), sign_extension_sdigit(lhs))
        } else {
            IBig::from_digit(diff)
        }
    }

    fn apply_digit_ref(lhs: SignedDigit, rhs: &[Digit]) -> IBig {
        // `rhs` is longer than the single digit `lhs`; sign-extend `lhs` to match in `apply_val_ref`.
        let mut digits = Digits::with_capacity(rhs.len() + 1);
        digits.push(lhs.cast_unsigned());
        Self::apply_val_ref(digits, rhs)
    }

    fn apply_digit_val(lhs: SignedDigit, mut rhs: Digits) -> IBig {
        // Reuse `rhs`'s storage: `rhs = lhs - rhs`.
        let scarry = ibig_core::sub_reverse_signed_sdigit(&mut rhs, lhs);
        IBig::from_digits_scarry(rhs, scarry)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: SignedDigit) -> IBig {
        // Clone `lhs` with room for a possible sign digit.
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        Self::apply_val_digit(digits, rhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        // Clone `lhs`, with room for sign-extension up to `rhs`'s length and a possible sign digit.
        let mut digits = Digits::with_capacity(lhs.len().max(rhs.len()) + 1);
        digits.extend_from_slice(lhs);
        Self::apply_val_ref(digits, rhs)
    }

    fn apply_ref_val(lhs: &[Digit], mut rhs: Digits) -> IBig {
        // Reuse `rhs`'s storage: `rhs = lhs - rhs`.
        let rhs_len = rhs.len();
        let scarry = if rhs_len >= lhs.len() {
            ibig_core::sub_reverse_signed_signed(&mut rhs, lhs)
        } else {
            let lhs_extension = sign_extension(lhs);
            let rhs_extension = sign_extension(&rhs);
            rhs.reserve(lhs.len() - rhs_len + 1);
            let (lhs_low, lhs_high) = lhs.split_at(rhs_len);
            let borrow = ibig_core::sub_reverse_unsigned_unsigned_same_len(&mut rhs, lhs_low);
            rhs.extend_from_slice(lhs_high);
            let low_carry = -SignedDigit::from(borrow) - rhs_extension; // -1..=1
            ibig_core::add_unsigned_scarry(&mut rhs[rhs_len..], low_carry) + lhs_extension
        };
        IBig::from_digits_scarry(rhs, scarry)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: SignedDigit) -> IBig {
        let scarry = ibig_core::sub_signed_sdigit(&mut lhs, rhs);
        IBig::from_digits_scarry(lhs, scarry)
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
        let scarry = ibig_core::sub_signed_signed(&mut lhs, rhs);
        IBig::from_digits_scarry(lhs, scarry)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> IBig {
        // Reuse `lhs`'s storage.
        Self::apply_val_ref(lhs, &rhs)
    }
}

impl_binary_operator!(
    IBig,
    IBig,
    Sub::sub,
    SubAssign::sub_assign,
    DigitsRhs<SubOperation>
);

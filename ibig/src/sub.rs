//! Subtraction.

use crate::ops::{
    BigBig, BinaryOpRef, BinaryOpRefBigBig, BinaryOpRefValBigBig, impl_binary_operator,
};
use crate::repr::Digits;
use crate::{IBig, UBig};
use core::ops::{Sub, SubAssign};
use ibig_core::{Digit, IDigit, sign_extension, sign_extension_idigit};

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
        <CheckedSubUBigUBig as BinaryOpRef>::apply_ref_ref(self, rhs)
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

    /// Subtracts the signed `rhs` from `self`, returning `None` if the result would be negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(UBig::from(5u8).checked_sub_signed(&IBig::from(3)), Some(UBig::from(2u8)));
    /// assert_eq!(UBig::from(5u8).checked_sub_signed(&IBig::from(-3)), Some(UBig::from(8u8)));
    /// assert_eq!(UBig::from(5u8).checked_sub_signed(&IBig::from(8)), None);
    /// ```
    pub fn checked_sub_signed(&self, rhs: &IBig) -> Option<UBig> {
        <CheckedSubUBigIBig as BinaryOpRef>::apply_ref_ref(self, rhs)
    }

    /// Subtracts the signed `rhs` from `self`.
    ///
    /// # Panics
    ///
    /// Panics if the result would be negative.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(UBig::from(5u8).strict_sub_signed(&IBig::from(-3)), UBig::from(8u8));
    /// ```
    pub fn strict_sub_signed(&self, rhs: &IBig) -> UBig {
        self.checked_sub_signed(rhs)
            .unwrap_or_else(|| UBig::panic_negative())
    }

    /// Subtracts the signed `rhs` from `self`, saturating at zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(UBig::from(5u8).saturating_sub_signed(&IBig::from(3)), UBig::from(2u8));
    /// assert_eq!(UBig::from(5u8).saturating_sub_signed(&IBig::from(8)), UBig::ZERO);
    /// ```
    pub fn saturating_sub_signed(&self, rhs: &IBig) -> UBig {
        self.checked_sub_signed(rhs).unwrap_or(UBig::ZERO)
    }
}

/// Subtraction operation for [`UBig`].
enum SubUBigUBig {}

impl BinaryOpRefValBigBig for SubUBigUBig {
    type Left = UBig;
    type Right = UBig;
    type Output = UBig;

    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        <CheckedSubUBigUBig as BinaryOpRefBigBig>::apply_digit_digit(lhs, rhs)
            .unwrap_or_else(|| UBig::panic_negative())
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
        <CheckedSubUBigUBig as BinaryOpRefBigBig>::apply_ref_ref(lhs, rhs)
            .unwrap_or_else(|| UBig::panic_negative())
    }

    fn apply_ref_val(lhs: &[Digit], mut rhs: Digits) -> UBig {
        let rhs_len = rhs.len();
        if lhs.len() < rhs_len {
            UBig::panic_negative();
        }
        rhs.reserve(lhs.len() - rhs_len);
        let (lhs_low, lhs_high) = lhs.split_at(rhs_len);
        let borrow = ibig_core::sub_rev_unsigned_unsigned_same_len(&mut rhs, lhs_low);
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
    Sub::sub(UBig, UBig) -> UBig,
    SubAssign::sub_assign,
    BigBig<SubUBigUBig>
);

/// The [`UBig::checked_sub`] operation.
enum CheckedSubUBigUBig {}

impl BinaryOpRefBigBig for CheckedSubUBigUBig {
    type Left = UBig;
    type Right = UBig;
    type Output = Option<UBig>;

    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> Option<UBig> {
        lhs.checked_sub(rhs).map(UBig::from_digit)
    }

    fn apply_digit_ref(_lhs: Digit, _rhs: &[Digit]) -> Option<UBig> {
        // A multi-digit `rhs` is bigger than any single digit.
        None
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> Option<UBig> {
        // A multi-digit `lhs` minus a single digit never underflows.
        Some(SubUBigUBig::apply_ref_digit(lhs, rhs))
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Option<UBig> {
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

/// The [`UBig::checked_sub_signed`] operation.
enum CheckedSubUBigIBig {}

impl BinaryOpRefBigBig for CheckedSubUBigIBig {
    type Left = UBig;
    type Right = IBig;
    type Output = Option<UBig>;

    fn apply_digit_digit(lhs: Digit, rhs: IDigit) -> Option<UBig> {
        let (low, icarry) = ibig_core::sub_digit_idigit(lhs, rhs);
        UBig::try_from_digit_icarry(low, icarry)
    }

    fn apply_digit_ref(lhs: Digit, rhs: &[Digit]) -> Option<UBig> {
        // `rhs` (at least two digits) is longer than the single digit. If it is at least two
        // digits longer and positive, it exceeds `lhs`, so the result is negative.
        if rhs.len() >= 3 && !ibig_core::is_negative(rhs) {
            return None;
        }
        // `lhs - rhs`: subtract the low digit, then bitwise-NOT `rhs`'s top digits and add the
        // carry out of the low subtraction. Subtracting a longer value can't overflow upward,
        // so the result fits in `rhs.len()` digits.
        let mut digits = Digits::with_capacity(rhs.len());
        let (&rhs_low, rhs_high) = rhs.split_first().unwrap();
        let (low, borrow) = lhs.overflowing_sub(rhs_low);
        digits.push(low);
        digits.extend_from_slice(rhs_high);
        let high = &mut digits[1..];
        ibig_core::not(high);
        // -rhs_high = !rhs_high + 1
        // low_carry = 1 - borrow = !borrow
        let low_carry = !borrow; // 0..=1
        let high_carry = ibig_core::add_unsigned_carry(high, low_carry);
        let icarry = !sign_extension(rhs) + IDigit::from(high_carry); // -1..=0
        UBig::try_from_digits_icarry(digits, icarry)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: IDigit) -> Option<UBig> {
        // Clone `lhs` with room for a possible carry.
        let mut digits = Digits::with_capacity(lhs.len() + 1);
        digits.extend_from_slice(lhs);
        let icarry = ibig_core::sub_unsigned_idigit(&mut digits, rhs); // 0..=1
        UBig::try_from_digits_icarry(digits, icarry)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Option<UBig> {
        if lhs.len() >= rhs.len() {
            // Clone the unsigned `lhs` and subtract the signed `rhs`; the result may grow a digit.
            let mut digits = Digits::with_capacity(lhs.len() + 1);
            digits.extend_from_slice(lhs);
            let icarry = ibig_core::sub_unsigned_signed(&mut digits, rhs);
            UBig::try_from_digits_icarry(digits, icarry)
        } else {
            // `rhs` is longer. If it is at least two digits longer and positive, it exceeds any
            // `lhs` of `lhs.len()` digits, so the result is negative.
            if rhs.len() >= lhs.len() + 2 && !ibig_core::is_negative(rhs) {
                return None;
            }
            // `lhs - rhs`: subtract the low digits, then bitwise-NOT `rhs`'s top digits and add
            // the carry out of the low subtraction. Subtracting a longer value can't overflow
            // upward, so the result fits in `rhs.len()` digits.
            let mut digits = Digits::from_slice(rhs);
            let (low, high) = digits.split_at_mut(lhs.len());
            let borrow = ibig_core::sub_rev_unsigned_unsigned_same_len(low, lhs);
            ibig_core::not(high);
            // -rhs_high = !rhs_high + 1
            // low_carry = 1 - borrow = !borrow
            let low_carry = !borrow; // 0..=1
            let high_carry = ibig_core::add_unsigned_carry(high, low_carry);
            let icarry = !sign_extension(rhs) + IDigit::from(high_carry); // -1..=0
            UBig::try_from_digits_icarry(digits, icarry)
        }
    }
}

/// Subtraction operation for [`IBig`].
enum SubIBigIBig {}

impl BinaryOpRefValBigBig for SubIBigIBig {
    type Left = IBig;
    type Right = IBig;
    type Output = IBig;

    fn apply_digit_digit(lhs: IDigit, rhs: IDigit) -> IBig {
        let (diff, overflow) = lhs.overflowing_sub(rhs);
        if overflow {
            // On overflow `lhs` and `rhs` have opposite signs, and the result's sign is `lhs`'s.
            IBig::from_two_digits(diff.cast_unsigned(), sign_extension_idigit(lhs))
        } else {
            IBig::from_digit(diff)
        }
    }

    fn apply_digit_ref(lhs: IDigit, rhs: &[Digit]) -> IBig {
        // `rhs` is longer than the single digit `lhs`; sign-extend `lhs` to match in `apply_val_ref`.
        let mut digits = Digits::with_capacity(rhs.len() + 1);
        digits.push(lhs.cast_unsigned());
        Self::apply_val_ref(digits, rhs)
    }

    fn apply_digit_val(lhs: IDigit, mut rhs: Digits) -> IBig {
        // Reuse `rhs`'s storage: `rhs = lhs - rhs`.
        let icarry = ibig_core::sub_rev_signed_idigit(&mut rhs, lhs);
        IBig::from_digits_icarry(rhs, icarry)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: IDigit) -> IBig {
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
        let icarry = if rhs_len >= lhs.len() {
            ibig_core::sub_rev_signed_signed(&mut rhs, lhs)
        } else {
            let lhs_extension = sign_extension(lhs);
            let rhs_extension = sign_extension(&rhs);
            rhs.reserve(lhs.len() - rhs_len + 1);
            let (lhs_low, lhs_high) = lhs.split_at(rhs_len);
            let borrow = ibig_core::sub_rev_unsigned_unsigned_same_len(&mut rhs, lhs_low);
            rhs.extend_from_slice(lhs_high);
            let low_carry = -IDigit::from(borrow) - rhs_extension; // -1..=1
            ibig_core::add_unsigned_icarry(&mut rhs[rhs_len..], low_carry) + lhs_extension
        };
        IBig::from_digits_icarry(rhs, icarry)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: IDigit) -> IBig {
        let icarry = ibig_core::sub_signed_idigit(&mut lhs, rhs);
        IBig::from_digits_icarry(lhs, icarry)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> IBig {
        let lhs_len = lhs.len();
        let icarry = if lhs_len >= rhs.len() {
            ibig_core::sub_signed_signed(&mut lhs, rhs)
        } else {
            // `rhs` is longer: subtract the low digits, then bitwise-NOT `rhs`'s top digits and
            // add the carry (folding in `lhs`'s sign). This is the dual of `apply_ref_val`'s
            // longer-`lhs` branch, avoiding a full sign-extension of `lhs`.
            let lhs_extension = sign_extension(&lhs);
            lhs.reserve(rhs.len() - lhs_len + 1);
            let (rhs_low, rhs_high) = rhs.split_at(lhs_len);
            let borrow = ibig_core::sub_unsigned_unsigned_same_len(&mut lhs, rhs_low);
            lhs.extend_from_slice(rhs_high);
            ibig_core::not(&mut lhs[lhs_len..]);
            let low_carry = IDigit::from(1u8) + lhs_extension - IDigit::from(borrow); // -1..=1
            ibig_core::add_unsigned_icarry(&mut lhs[lhs_len..], low_carry) + !sign_extension(rhs)
        };
        IBig::from_digits_icarry(lhs, icarry)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> IBig {
        // Reuse the longer operand's storage.
        if lhs.len() >= rhs.len() {
            Self::apply_val_ref(lhs, &rhs)
        } else {
            Self::apply_ref_val(&lhs, rhs)
        }
    }
}

impl_binary_operator!(
    Sub::sub(IBig, IBig) -> IBig,
    SubAssign::sub_assign,
    BigBig<SubIBigIBig>
);

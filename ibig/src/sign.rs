//! Sign operations on [`IBig`].

use crate::ops::{UnaryOpRef, UnaryOpRefBig, UnaryOpRefValBig, impl_unary_operator};
use crate::repr::Digits;
use crate::{IBig, UBig};
use core::ops::Neg;
use ibig_core::{Digit, IDigit};

impl IBig {
    /// Returns `true` if the number is negative (less than zero).
    pub fn is_negative(&self) -> bool {
        <IsNegativeIBig as UnaryOpRef>::apply_ref(self)
    }

    /// Returns `true` if the number is positive (greater than zero).
    pub fn is_positive(&self) -> bool {
        <IsPositiveIBig as UnaryOpRef>::apply_ref(self)
    }

    /// Returns a number representing the sign of `self`:
    /// * `-1` if negative
    /// * `0` if zero
    /// * `1` if positive
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from(-5i8).signum(), IBig::from(-1i8));
    /// assert_eq!(IBig::ZERO.signum(), IBig::ZERO);
    /// assert_eq!(IBig::from(5i8).signum(), IBig::from(1i8));
    /// ```
    pub fn signum(&self) -> IBig {
        <SignumIBig as UnaryOpRef>::apply_ref(self)
    }

    /// Returns the absolute value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from(-5i8).abs(), IBig::from(5i8));
    /// assert_eq!(IBig::from(5i8).abs(), IBig::from(5i8));
    /// ```
    pub fn abs(&self) -> IBig {
        <AbsIBig as UnaryOpRef>::apply_ref(self)
    }

    /// Returns the absolute value as a [`UBig`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::{IBig, UBig};
    /// assert_eq!(IBig::from(-5i8).abs_unsigned(), UBig::from(5u8));
    /// assert_eq!(IBig::from(5i8).abs_unsigned(), UBig::from(5u8));
    /// ```
    pub fn abs_unsigned(&self) -> UBig {
        <AbsUnsignedIBig as UnaryOpRef>::apply_ref(self)
    }
}

/// The [`IBig::is_negative`] operation.
enum IsNegativeIBig {}

impl UnaryOpRefBig for IsNegativeIBig {
    type Operand = IBig;
    type Output = bool;

    fn apply_digit(operand: IDigit) -> bool {
        operand.is_negative()
    }

    fn apply_ref(operand: &[Digit]) -> bool {
        ibig_core::is_negative(operand)
    }
}

/// The [`IBig::is_positive`] operation.
enum IsPositiveIBig {}

impl UnaryOpRefBig for IsPositiveIBig {
    type Operand = IBig;
    type Output = bool;

    fn apply_digit(operand: IDigit) -> bool {
        operand.is_positive()
    }

    fn apply_ref(operand: &[Digit]) -> bool {
        // A multi-digit value is never zero, so it is positive iff not negative.
        !ibig_core::is_negative(operand)
    }
}

/// The [`IBig::signum`] operation.
enum SignumIBig {}

impl UnaryOpRefBig for SignumIBig {
    type Operand = IBig;
    type Output = IBig;

    fn apply_digit(operand: IDigit) -> IBig {
        IBig::from_digit(operand.signum())
    }

    fn apply_ref(operand: &[Digit]) -> IBig {
        // A multi-digit value is never zero.
        if ibig_core::is_negative(operand) {
            IBig::from(-1i8)
        } else {
            IBig::from(1i8)
        }
    }
}

/// The [`IBig::abs`] operation.
enum AbsIBig {}

impl UnaryOpRefBig for AbsIBig {
    type Operand = IBig;
    type Output = IBig;

    fn apply_digit(operand: IDigit) -> IBig {
        // The magnitude is non-negative; only `IDigit::MIN` needs a second digit for the sign.
        IBig::from_two_digits(operand.unsigned_abs(), IDigit::ZERO)
    }

    fn apply_ref(operand: &[Digit]) -> IBig {
        let mut digits = Digits::with_capacity(operand.len() + 1);
        digits.extend_from_slice(operand);
        ibig_core::abs(&mut digits);
        // `abs` left an unsigned magnitude; reinterpret it as a non-negative two's complement
        // value, appending a zero sign digit (within the reserved capacity) when its top bit is
        // set.
        IBig::from_digits_idigit(digits, IDigit::ZERO)
    }
}

/// The [`IBig::abs_unsigned`] operation.
enum AbsUnsignedIBig {}

impl UnaryOpRefBig for AbsUnsignedIBig {
    type Operand = IBig;
    type Output = UBig;

    fn apply_digit(operand: IDigit) -> UBig {
        UBig::from_digit(operand.unsigned_abs())
    }

    fn apply_ref(operand: &[Digit]) -> UBig {
        let mut digits = Digits::from_slice(operand);
        ibig_core::abs(&mut digits);
        UBig::from_digits(digits)
    }
}

/// Negation for [`IBig`].
enum NegIBig {}

impl UnaryOpRefValBig for NegIBig {
    type Operand = IBig;
    type Output = IBig;

    fn apply_digit(operand: IDigit) -> IBig {
        let (neg, overflow) = operand.overflowing_neg();
        if overflow {
            // Only `IDigit::MIN` overflows; -MIN == 2^(bits-1) needs a second digit.
            IBig::from_two_digits(neg.cast_unsigned(), IDigit::ZERO)
        } else {
            IBig::from_digit(neg)
        }
    }

    fn apply_ref(operand: &[Digit]) -> IBig {
        // Clone with room for a possible sign digit.
        let mut digits = Digits::with_capacity(operand.len() + 1);
        digits.extend_from_slice(operand);
        <Self as UnaryOpRefValBig>::apply_val(digits)
    }

    fn apply_val(mut operand: Digits) -> IBig {
        let icarry = ibig_core::neg(&mut operand);
        IBig::from_digits_idigit(operand, icarry)
    }
}

impl_unary_operator!(Neg::neg(IBig) -> IBig, NegIBig);

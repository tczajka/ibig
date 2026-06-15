//! Sign operations on [`IBig`].

use crate::IBig;
use crate::ops::{UnaryOpRef, UnaryOpRefBig, UnaryOpRefValBig, impl_unary_operator};
use crate::repr::Digits;
use core::ops::Neg;
use ibig_core::{Digit, SignedDigit};

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
}

/// The [`IBig::is_negative`] operation.
enum IsNegativeIBig {}

impl UnaryOpRefBig for IsNegativeIBig {
    type Operand = IBig;
    type Output = bool;

    fn apply_digit(operand: SignedDigit) -> bool {
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

    fn apply_digit(operand: SignedDigit) -> bool {
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

    fn apply_digit(operand: SignedDigit) -> IBig {
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

/// Negation for [`IBig`].
enum NegIBig {}

impl UnaryOpRefValBig for NegIBig {
    type Operand = IBig;
    type Output = IBig;

    fn apply_digit(operand: SignedDigit) -> IBig {
        let (neg, overflow) = operand.overflowing_neg();
        if overflow {
            // Only `SignedDigit::MIN` overflows; -MIN == 2^(bits-1) needs a second digit.
            IBig::from_two_digits(neg.cast_unsigned(), SignedDigit::ZERO)
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
        let scarry = ibig_core::neg(&mut operand);
        IBig::from_digits_scarry(operand, scarry)
    }
}

impl_unary_operator!(Neg::neg(IBig) -> IBig, NegIBig);

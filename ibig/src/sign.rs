//! Sign operations on [`IBig`].

use crate::IBig;
use crate::ops::{UnaryOpDigits, impl_unary_operator};
use crate::repr::Digits;
use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
};
use core::ops::Neg;
use ibig_core::{Digit, SignedDigit};

impl IBig {
    /// Returns `true` if the number is negative (less than zero).
    pub fn is_negative(&self) -> bool {
        match self.as_digits() {
            Small(digit) => digit.is_negative(),
            Large(digits) => ibig_core::is_negative(digits),
        }
    }

    /// Returns `true` if the number is positive (greater than zero).
    pub fn is_positive(&self) -> bool {
        match self.as_digits() {
            Small(digit) => digit.is_positive(),
            // A multi-digit value is never zero, so it is positive iff not negative.
            Large(digits) => !ibig_core::is_negative(digits),
        }
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
        match self.as_digits() {
            Small(digit) => IBig::from_digit(digit.signum()),
            Large(digits) => IBig::signum_ref(digits),
        }
    }

    /// [`IBig::signum`] for a borrowed slice.
    fn signum_ref(digits: &[Digit]) -> IBig {
        // A multi-digit value is never zero.
        if ibig_core::is_negative(digits) {
            IBig::from(-1i8)
        } else {
            IBig::from(1i8)
        }
    }
}

/// Negation operation.
struct NegOperation;

impl UnaryOpDigits<IBig> for NegOperation {
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
        Self::apply_val(digits)
    }

    fn apply_val(mut operand: Digits) -> IBig {
        let scarry = ibig_core::neg(&mut operand);
        IBig::from_digits_scarry(operand, scarry)
    }
}

impl_unary_operator!(IBig, Neg::neg, NegOperation);

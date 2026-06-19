//! Equality comparison.

use crate::ops::{BigBig, BinaryOpRefVal, CommutativeBinaryOpRefValBigBig};
use crate::repr::Digits;
use crate::{IBig, UBig};
use ibig_core::{Digit, IDigit};

impl PartialEq for UBig {
    fn eq(&self, other: &UBig) -> bool {
        <BigBig<EqUBigUBig> as BinaryOpRefVal>::apply_ref_ref(self, other)
    }
}

impl Eq for UBig {}

impl PartialEq for IBig {
    fn eq(&self, other: &IBig) -> bool {
        <BigBig<EqIBigIBig> as BinaryOpRefVal>::apply_ref_ref(self, other)
    }
}

impl Eq for IBig {}

/// The [`UBig`] equality operation.
enum EqUBigUBig {}

impl CommutativeBinaryOpRefValBigBig for EqUBigUBig {
    type Operand = UBig;
    type Output = bool;

    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> bool {
        lhs == rhs
    }

    fn apply_ref_digit(_lhs: &[Digit], _rhs: Digit) -> bool {
        false
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> bool {
        lhs == rhs
    }

    fn apply_val_digit(_lhs: Digits, _rhs: Digit) -> bool {
        false
    }

    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> bool {
        lhs.as_slice() == rhs
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> bool {
        lhs == rhs
    }
}

/// The [`IBig`] equality operation.
enum EqIBigIBig {}

impl CommutativeBinaryOpRefValBigBig for EqIBigIBig {
    type Operand = IBig;
    type Output = bool;

    fn apply_digit_digit(lhs: IDigit, rhs: IDigit) -> bool {
        lhs == rhs
    }

    fn apply_ref_digit(_lhs: &[Digit], _rhs: IDigit) -> bool {
        false
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> bool {
        lhs == rhs
    }

    fn apply_val_digit(_lhs: Digits, _rhs: IDigit) -> bool {
        false
    }

    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> bool {
        lhs.as_slice() == rhs
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> bool {
        lhs == rhs
    }
}

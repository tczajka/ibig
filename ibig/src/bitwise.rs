//! Bitwise operators for [`UBig`] and [`IBig`].

use crate::ops::{
    CommutativeBinaryOpDigits, DigitsRhs, UnaryOpDigits, impl_binary_operator, impl_unary_operator,
};
use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
    Digits,
};
use crate::{IBig, UBig};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use ibig_core::{Digit, SignedDigit};

impl UBig {
    /// Returns `self & !rhs`: the value `self` with every bit that is set in `rhs` cleared.
    ///
    /// This is what `self & !rhs` would compute, but [`UBig`] has no bitwise-NOT operator:
    /// `!rhs` would have infinitely many leading one bits and is not representable as a
    /// [`UBig`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(
    ///     UBig::from(0b1110u8).bitandnot(&UBig::from(0b0101u8)),
    ///     UBig::from(0b1010u8)
    /// );
    /// ```
    pub fn bitandnot(&self, rhs: &UBig) -> UBig {
        match (self.as_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => UBig::from_digit(a & !b),
            (Small(a), Large(rhs)) => UBig::from_digit(a & !rhs[0]),
            (Large(lhs), Small(b)) => UBig::bitandnot_ref_digit(lhs, b),
            (Large(lhs), Large(rhs)) => UBig::bitandnot_ref_ref(lhs, rhs),
        }
    }

    /// [`UBig::bitandnot`] for a borrowed slice and a single digit.
    fn bitandnot_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        let mut digits = Digits::from_slice(lhs);
        digits[0] &= !rhs;
        UBig::from_digits(digits)
    }

    /// [`UBig::bitandnot`] for two borrowed slices.
    fn bitandnot_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        let mut digits = Digits::from_slice(lhs);
        let n = digits.len().min(rhs.len());
        ibig_core::bitandnot_same_len(&mut digits[..n], &rhs[..n]);
        UBig::from_digits(digits)
    }
}

/// Bitwise NOT operation.
struct NotOperation;

impl UnaryOpDigits<IBig> for NotOperation {
    fn apply_digit(operand: SignedDigit) -> IBig {
        IBig::from_digit(!operand)
    }

    fn apply_ref(operand: &[Digit]) -> IBig {
        Self::apply_val(Digits::from_slice(operand))
    }

    fn apply_val(mut operand: Digits) -> IBig {
        ibig_core::not(&mut operand);
        IBig::from_digits(operand)
    }
}

impl_unary_operator!(IBig, Not::not, NotOperation);

/// Bitwise AND operation.
struct BitAndOperation;

impl CommutativeBinaryOpDigits<UBig> for BitAndOperation {
    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        UBig::from_digit(lhs & rhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        Self::apply_digit_digit(lhs[0], rhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        let n = lhs.len().min(rhs.len());
        let mut digits = Digits::from_slice(&lhs[..n]);
        ibig_core::bitand_same_len(&mut digits, &rhs[..n]);
        UBig::from_digits(digits)
    }

    fn apply_val_digit(lhs: Digits, rhs: Digit) -> UBig {
        Self::apply_ref_digit(&lhs, rhs)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> UBig {
        let n = lhs.len().min(rhs.len());
        lhs.truncate(n);
        ibig_core::bitand_same_len(&mut lhs, &rhs[..n]);
        UBig::from_digits(lhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> UBig {
        // Reuse storage from shorter operand.
        if lhs.len() <= rhs.len() {
            Self::apply_val_ref(lhs, &rhs)
        } else {
            Self::apply_val_ref(rhs, &lhs)
        }
    }
}

impl_binary_operator!(
    UBig,
    UBig,
    BitAnd::bitand,
    BitAndAssign::bitand_assign,
    DigitsRhs<BitAndOperation>
);

impl CommutativeBinaryOpDigits<IBig> for BitAndOperation {
    fn apply_digit_digit(lhs: SignedDigit, rhs: SignedDigit) -> IBig {
        IBig::from_digit(lhs & rhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: SignedDigit) -> IBig {
        if rhs.is_negative() {
            // High digits are preserved.
            Self::apply_val_digit(Digits::from_slice(lhs), rhs)
        } else {
            // High digits are zeroed.
            Self::apply_digit_digit(lhs[0].cast_signed(), rhs)
        }
    }

    fn apply_val_digit(mut lhs: Digits, rhs: SignedDigit) -> IBig {
        if rhs.is_negative() {
            // High digits are preserved.
            lhs[0] &= rhs.cast_unsigned();
            IBig::from_digits(lhs)
        } else {
            // High digits are zeroed.
            Self::apply_digit_digit(lhs[0].cast_signed(), rhs)
        }
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        if ibig_core::is_negative(shorter) {
            // High digits are preserved: clone longer operand.
            Self::apply_val_ref(Digits::from_slice(longer), shorter)
        } else {
            // High digits are zeroed: clone shorter operand.
            Self::apply_val_ref(Digits::from_slice(shorter), longer)
        }
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> IBig {
        if lhs.len() >= rhs.len() {
            if !ibig_core::is_negative(rhs) {
                // Zero out high digits.
                lhs.truncate(rhs.len());
            }
            ibig_core::bitand_same_len(&mut lhs[..rhs.len()], rhs);
        } else {
            let (rhs_low, rhs_high) = rhs.split_at(lhs.len());
            let lhs_negative = ibig_core::is_negative(&lhs);
            ibig_core::bitand_same_len(&mut lhs, rhs_low);
            if lhs_negative {
                // Include high digits from `rhs`.
                lhs.extend_from_slice(rhs_high);
            }
        }
        IBig::from_digits(lhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> IBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        if ibig_core::is_negative(&shorter) {
            // High digits are preserved: reuse storage from longer operand.
            Self::apply_val_ref(longer, &shorter)
        } else {
            // High digits are zeroed: reuse storage from shorter operand.
            Self::apply_val_ref(shorter, &longer)
        }
    }
}

impl_binary_operator!(
    IBig,
    IBig,
    BitAnd::bitand,
    BitAndAssign::bitand_assign,
    DigitsRhs<BitAndOperation>
);

/// Bitwise OR operation.
struct BitOrOperation;

impl CommutativeBinaryOpDigits<UBig> for BitOrOperation {
    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        UBig::from_digit(lhs | rhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        Self::apply_val_digit(Digits::from_slice(lhs), rhs)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: Digit) -> UBig {
        // OR with a single digit only touches the low digit; the high digits are kept.
        lhs[0] |= rhs;
        UBig::from_digits(lhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        Self::apply_val_ref(Digits::from_slice(longer), shorter)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> UBig {
        // The high digits of the longer operand are kept (OR with the zero-extension).
        if lhs.len() >= rhs.len() {
            ibig_core::bitor_same_len(&mut lhs[..rhs.len()], rhs);
        } else {
            let (rhs_low, rhs_high) = rhs.split_at(lhs.len());
            ibig_core::bitor_same_len(&mut lhs, rhs_low);
            lhs.extend_from_slice(rhs_high);
        }
        UBig::from_digits(lhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> UBig {
        // Reuse storage from the longer operand.
        if lhs.len() >= rhs.len() {
            Self::apply_val_ref(lhs, &rhs)
        } else {
            Self::apply_val_ref(rhs, &lhs)
        }
    }
}

impl_binary_operator!(
    UBig,
    UBig,
    BitOr::bitor,
    BitOrAssign::bitor_assign,
    DigitsRhs<BitOrOperation>
);

impl CommutativeBinaryOpDigits<IBig> for BitOrOperation {
    fn apply_digit_digit(lhs: SignedDigit, rhs: SignedDigit) -> IBig {
        IBig::from_digit(lhs | rhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: SignedDigit) -> IBig {
        if rhs.is_negative() {
            // OR with a negative value sets every high bit, collapsing to a single digit.
            Self::apply_digit_digit(lhs[0].cast_signed(), rhs)
        } else {
            // High digits are preserved.
            Self::apply_val_digit(Digits::from_slice(lhs), rhs)
        }
    }

    fn apply_val_digit(mut lhs: Digits, rhs: SignedDigit) -> IBig {
        if rhs.is_negative() {
            // OR with a negative value sets every high bit, collapsing to a single digit.
            Self::apply_digit_digit(lhs[0].cast_signed(), rhs)
        } else {
            // High digits are preserved.
            lhs[0] |= rhs.cast_unsigned();
            IBig::from_digits(lhs)
        }
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        if ibig_core::is_negative(shorter) {
            // The high digits become redundant ones: the result has the shorter length.
            Self::apply_val_ref(Digits::from_slice(shorter), longer)
        } else {
            // High digits are preserved: clone the longer operand.
            Self::apply_val_ref(Digits::from_slice(longer), shorter)
        }
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> IBig {
        if lhs.len() >= rhs.len() {
            if ibig_core::is_negative(rhs) {
                // The shorter `rhs` is negative: its high digits are all ones, so they fill (and
                // make redundant) the high digits of `lhs`.
                lhs.truncate(rhs.len());
            }
            ibig_core::bitor_same_len(&mut lhs[..rhs.len()], rhs);
        } else {
            let (rhs_low, rhs_high) = rhs.split_at(lhs.len());
            let lhs_negative = ibig_core::is_negative(&lhs);
            ibig_core::bitor_same_len(&mut lhs, rhs_low);
            if !lhs_negative {
                // The shorter `lhs` is non-negative: keep the high digits from `rhs`.
                lhs.extend_from_slice(rhs_high);
            }
        }
        IBig::from_digits(lhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> IBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        if ibig_core::is_negative(&shorter) {
            // The result has the shorter length: reuse the shorter operand.
            Self::apply_val_ref(shorter, &longer)
        } else {
            // High digits are preserved: reuse the longer operand.
            Self::apply_val_ref(longer, &shorter)
        }
    }
}

impl_binary_operator!(
    IBig,
    IBig,
    BitOr::bitor,
    BitOrAssign::bitor_assign,
    DigitsRhs<BitOrOperation>
);

/// Bitwise XOR operation.
struct BitXorOperation;

impl CommutativeBinaryOpDigits<UBig> for BitXorOperation {
    fn apply_digit_digit(lhs: Digit, rhs: Digit) -> UBig {
        UBig::from_digit(lhs ^ rhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: Digit) -> UBig {
        Self::apply_val_digit(Digits::from_slice(lhs), rhs)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: Digit) -> UBig {
        // XOR with a single digit only touches the low digit; the high digits are kept.
        lhs[0] ^= rhs;
        UBig::from_digits(lhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> UBig {
        let (longer, shorter) = if lhs.len() >= rhs.len() {
            (lhs, rhs)
        } else {
            (rhs, lhs)
        };
        Self::apply_val_ref(Digits::from_slice(longer), shorter)
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> UBig {
        // The high digits of the longer operand are kept (XOR with the zero-extension).
        if lhs.len() >= rhs.len() {
            ibig_core::bitxor_same_len(&mut lhs[..rhs.len()], rhs);
        } else {
            let (rhs_low, rhs_high) = rhs.split_at(lhs.len());
            ibig_core::bitxor_same_len(&mut lhs, rhs_low);
            lhs.extend_from_slice(rhs_high);
        }
        UBig::from_digits(lhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> UBig {
        // Reuse storage from the longer operand.
        if lhs.len() >= rhs.len() {
            Self::apply_val_ref(lhs, &rhs)
        } else {
            Self::apply_val_ref(rhs, &lhs)
        }
    }
}

impl_binary_operator!(
    UBig,
    UBig,
    BitXor::bitxor,
    BitXorAssign::bitxor_assign,
    DigitsRhs<BitXorOperation>
);

impl CommutativeBinaryOpDigits<IBig> for BitXorOperation {
    fn apply_digit_digit(lhs: SignedDigit, rhs: SignedDigit) -> IBig {
        IBig::from_digit(lhs ^ rhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: SignedDigit) -> IBig {
        Self::apply_val_digit(Digits::from_slice(lhs), rhs)
    }

    fn apply_val_digit(mut lhs: Digits, rhs: SignedDigit) -> IBig {
        let (lhs_low, lhs_high) = lhs.split_first_mut().unwrap();
        *lhs_low ^= rhs.cast_unsigned();
        if rhs.is_negative() {
            // The sign-extension is all ones, so the high digits are flipped.
            ibig_core::not(lhs_high);
        }
        IBig::from_digits(lhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> IBig {
        // Clone the longer operand.
        if lhs.len() >= rhs.len() {
            Self::apply_val_ref(Digits::from_slice(lhs), rhs)
        } else {
            Self::apply_val_ref(Digits::from_slice(rhs), lhs)
        }
    }

    fn apply_val_ref(mut lhs: Digits, rhs: &[Digit]) -> IBig {
        if lhs.len() >= rhs.len() {
            let (lhs_low, lhs_high) = lhs.split_at_mut(rhs.len());
            ibig_core::bitxor_same_len(lhs_low, rhs);
            if ibig_core::is_negative(rhs) {
                // The shorter `rhs` is negative: its sign-extension flips the high digits.
                ibig_core::not(lhs_high);
            }
        } else {
            let (rhs_low, rhs_high) = rhs.split_at(lhs.len());
            let lhs_negative = ibig_core::is_negative(&lhs);
            ibig_core::bitxor_same_len(&mut lhs, rhs_low);
            let high_start = lhs.len();
            lhs.extend_from_slice(rhs_high);
            if lhs_negative {
                // The shorter `lhs` is negative: its sign-extension flips the high digits.
                ibig_core::not(&mut lhs[high_start..]);
            }
        }
        IBig::from_digits(lhs)
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
    IBig,
    IBig,
    BitXor::bitxor,
    BitXorAssign::bitxor_assign,
    DigitsRhs<BitXorOperation>
);

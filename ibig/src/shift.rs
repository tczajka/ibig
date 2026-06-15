//! Bit shift operators (`Shl`, `Shr`) for [`UBig`] and [`IBig`] by a `usize` amount.

use crate::ops::{BigCopy, BinaryOpRefValBigCopy, impl_binary_operator};
use crate::repr::{Digits, MAX_DIGITS, panic_number_too_large};
use crate::{IBig, UBig};
use core::iter::repeat_n;
use core::ops::{Shl, ShlAssign, Shr, ShrAssign};
use ibig_core::{BitIndex, DIGIT_BITS_USIZE, Digit, SignedDigit};

/// Left shift of a [`UBig`].
enum ShlUBig {}

impl BinaryOpRefValBigCopy for ShlUBig {
    type Left = UBig;
    type Right = usize;
    type Output = UBig;

    fn apply_digit(lhs: Digit, rhs: usize) -> UBig {
        // Shifting zero is zero (and avoids allocating `rhs / DIGIT_BITS` zero digits).
        if lhs == Digit::ZERO {
            return UBig::ZERO;
        }
        let index = BitIndex::from(rhs);
        let (low, high) = ibig_core::shl_small_digit(lhs, index.bit_index());
        // With no whole-digit offset, the pair is the entire result.
        if index.digit_index() == 0 {
            return UBig::from_two_digits(low, high);
        }
        if index.digit_index() >= MAX_DIGITS {
            panic_number_too_large();
        }
        // The shifted pair sits above `digit_index` prepended zero digits.
        let mut digits = Digits::with_capacity(index.digit_index() + 2);
        digits.resize(index.digit_index(), Digit::ZERO);
        digits.push(low);
        digits.push(high);
        UBig::from_digits(digits)
    }

    fn apply_ref(lhs: &[Digit], rhs: usize) -> UBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() > MAX_DIGITS - lhs.len() {
            panic_number_too_large();
        }
        let mut digits = Digits::with_capacity(index.digit_index() + lhs.len() + 1);
        digits.resize(index.digit_index(), Digit::ZERO);
        digits.extend_from_slice(lhs);
        let overflow =
            ibig_core::shl_small_unsigned(&mut digits[index.digit_index()..], index.bit_index());
        digits.push(overflow);
        UBig::from_digits(digits)
    }

    fn apply_val(mut lhs: Digits, rhs: usize) -> UBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() > MAX_DIGITS - lhs.len() {
            panic_number_too_large();
        }
        // The result grows by exactly the prepended zero digits plus the overflow digit.
        lhs.reserve_exact(index.digit_index() + 1);
        // Shift the bits in place, then prepend the whole zero digits.
        let overflow = ibig_core::shl_small_unsigned(&mut lhs, index.bit_index());
        lhs.insert_many(0, repeat_n(Digit::ZERO, index.digit_index()));
        lhs.push(overflow);
        UBig::from_digits(lhs)
    }
}

impl_binary_operator!(
    Shl::shl(UBig, usize) -> UBig,
    ShlAssign::shl_assign,
    BigCopy<ShlUBig>
);

/// Left shift of an [`IBig`].
enum ShlIBig {}

impl BinaryOpRefValBigCopy for ShlIBig {
    type Left = IBig;
    type Right = usize;
    type Output = IBig;

    fn apply_digit(lhs: SignedDigit, rhs: usize) -> IBig {
        // Shifting zero is zero (and avoids allocating `rhs / DIGIT_BITS` zero digits).
        if lhs == SignedDigit::ZERO {
            return IBig::ZERO;
        }
        let index = BitIndex::from(rhs);
        let (low, high) = ibig_core::shl_small_sdigit(lhs, index.bit_index());
        // With no whole-digit offset, the pair is the entire result.
        if index.digit_index() == 0 {
            return IBig::from_two_digits(low, high);
        }
        if index.digit_index() >= MAX_DIGITS {
            panic_number_too_large();
        }
        // The shifted pair sits above `digit_index` prepended zero digits.
        let mut digits = Digits::with_capacity(index.digit_index() + 2);
        digits.resize(index.digit_index(), Digit::ZERO);
        digits.push(low);
        digits.push(high.cast_unsigned());
        IBig::from_digits(digits)
    }

    fn apply_ref(lhs: &[Digit], rhs: usize) -> IBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() > MAX_DIGITS - lhs.len() {
            panic_number_too_large();
        }
        let mut digits = Digits::with_capacity(index.digit_index() + lhs.len() + 1);
        digits.resize(index.digit_index(), Digit::ZERO);
        digits.extend_from_slice(lhs);
        // The sign-extended overflow keeps the top digit two's-complement correct.
        let overflow =
            ibig_core::shl_small_signed(&mut digits[index.digit_index()..], index.bit_index());
        digits.push(overflow.cast_unsigned());
        IBig::from_digits(digits)
    }

    fn apply_val(mut lhs: Digits, rhs: usize) -> IBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() > MAX_DIGITS - lhs.len() {
            panic_number_too_large();
        }
        // The result grows by exactly the prepended zero digits plus the overflow digit.
        lhs.reserve_exact(index.digit_index() + 1);
        // Shift the bits in place (sign-extending the overflow), then prepend zeros.
        let overflow = ibig_core::shl_small_signed(&mut lhs, index.bit_index());
        lhs.insert_many(0, repeat_n(Digit::ZERO, index.digit_index()));
        lhs.push(overflow.cast_unsigned());
        IBig::from_digits(lhs)
    }
}

impl_binary_operator!(
    Shl::shl(IBig, usize) -> IBig,
    ShlAssign::shl_assign,
    BigCopy<ShlIBig>
);

/// Right shift of a [`UBig`].
enum ShrUBig {}

impl BinaryOpRefValBigCopy for ShrUBig {
    type Left = UBig;
    type Right = usize;
    type Output = UBig;

    fn apply_digit(lhs: Digit, rhs: usize) -> UBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() != 0 {
            return UBig::ZERO;
        }
        UBig::from_digit(lhs >> index.bit_index())
    }

    fn apply_ref(lhs: &[Digit], rhs: usize) -> UBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() >= lhs.len() {
            return UBig::ZERO;
        }
        // Copy only the surviving high digits.
        let mut digits = Digits::from_slice(&lhs[index.digit_index()..]);
        ibig_core::shr_small_unsigned(&mut digits, index.bit_index());
        UBig::from_digits(digits)
    }

    fn apply_val(mut lhs: Digits, rhs: usize) -> UBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() >= lhs.len() {
            return UBig::ZERO;
        }
        lhs.drain(..index.digit_index());
        ibig_core::shr_small_unsigned(&mut lhs, index.bit_index());
        UBig::from_digits(lhs)
    }
}

impl_binary_operator!(
    Shr::shr(UBig, usize) -> UBig,
    ShrAssign::shr_assign,
    BigCopy<ShrUBig>
);

/// Right shift of an [`IBig`].
enum ShrIBig {}

impl BinaryOpRefValBigCopy for ShrIBig {
    type Left = IBig;
    type Right = usize;
    type Output = IBig;

    fn apply_digit(lhs: SignedDigit, rhs: usize) -> IBig {
        // Beyond `DIGIT_BITS - 1`, the arithmetic shift saturates to the sign.
        let small: u32 = rhs.min(DIGIT_BITS_USIZE - 1).try_into().unwrap();
        IBig::from_digit(lhs >> small)
    }

    fn apply_ref(lhs: &[Digit], rhs: usize) -> IBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() >= lhs.len() {
            return IBig::shr_whole(lhs);
        }
        // Copy only the surviving high digits.
        let mut digits = Digits::from_slice(&lhs[index.digit_index()..]);
        ibig_core::shr_small_signed(&mut digits, index.bit_index());
        IBig::from_digits(digits)
    }

    fn apply_val(mut lhs: Digits, rhs: usize) -> IBig {
        let index = BitIndex::from(rhs);
        if index.digit_index() >= lhs.len() {
            return IBig::shr_whole(&lhs);
        }
        lhs.drain(..index.digit_index());
        ibig_core::shr_small_signed(&mut lhs, index.bit_index());
        IBig::from_digits(lhs)
    }
}

impl_binary_operator!(
    Shr::shr(IBig, usize) -> IBig,
    ShrAssign::shr_assign,
    BigCopy<ShrIBig>
);

impl IBig {
    /// The result of right-shifting the two's complement `digits` past all of its digits: the
    /// sign, `0` or `-1`.
    fn shr_whole(digits: &[Digit]) -> IBig {
        IBig::from_digit(ibig_core::sign_extension(digits))
    }
}

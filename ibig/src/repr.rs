//! Contains the definitions of [`UBig`] and [`IBig`] and maintains their invariants.

use core::hint::assert_unchecked;
use ibig_core::{
    DIGIT_BITS_USIZE, Digit, IDigit, min_len_signed, min_len_unsigned, sign_extension,
    sign_extension_idigit,
};
use smallvec::{SmallVec, smallvec};

/// Number of [`Digit`]s stored inline before the representation spills to the heap.
pub(crate) const INLINE_DIGITS: usize = 4;

/// Maximum number of [`Digit`]s in a value, chosen so that the total bit length
/// (`MAX_DIGITS * Digit::BITS`) still fits in a `usize`.
pub(crate) const MAX_DIGITS: usize = usize::MAX / DIGIT_BITS_USIZE;

/// Panics because a value would exceed [`MAX_DIGITS`]. The single source of the "number too
/// large" panic message.
#[cold]
pub(crate) fn panic_number_too_large() -> ! {
    panic!("number too large")
}

/// Storage for little-endian digits.
///
/// Values of at most [`INLINE_DIGITS`] digits are stored inline; larger values spill to a
/// heap allocation.
pub(crate) type Digits = SmallVec<[Digit; INLINE_DIGITS]>;

/// Unsigned big integer.
///
/// An arbitrarily large unsigned integer.
///
/// Operations whose result would be negative (e.g. subtracting a larger number from a smaller
/// one) panic.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UBig(
    /// The little-endian digits in canonical form:
    /// * the buffer is never empty
    /// * most-significant digit is nonzero except for the value zero
    /// * heap buffer is at least 25% used
    /// * a single digit is always stored inline
    Digits,
);

impl UBig {
    /// Construct from a single digit.
    pub(crate) fn from_digit(digit: Digit) -> UBig {
        // A single digit is always canonical.
        UBig(smallvec![digit])
    }

    /// Construct from a single digit, usable in `const` contexts.
    pub(crate) const fn const_from_digit(digit: Digit) -> UBig {
        const { assert!(INLINE_DIGITS >= 1) };
        let mut digits = [Digit::ZERO; INLINE_DIGITS];
        digits[0] = digit;
        // A single digit is always canonical.
        // SAFETY: `1 <= INLINE_DIGITS`.
        UBig(unsafe { Digits::from_const_with_len_unchecked(digits, 1) })
    }

    /// Construct from two digits.
    pub(crate) fn from_two_digits(low: Digit, high: Digit) -> UBig {
        if high == Digit::ZERO {
            UBig::from_digit(low)
        } else {
            // Two digits with a nonzero most-significant digit are canonical.
            UBig(smallvec![low, high])
        }
    }

    /// Construct from little-endian digits.
    ///
    /// # Panics
    ///
    /// Panics if, after normalization, the value has more than [`MAX_DIGITS`] digits.
    pub(crate) fn from_digits(mut digits: Digits) -> UBig {
        digits.truncate(min_len_unsigned(&digits));
        // `min_len_unsigned` returns 0 for zero, but `UBig` always keeps at least one digit.
        if digits.is_empty() {
            digits.push(Digit::ZERO);
        }
        UBig::shrink(digits)
    }

    /// Construct from at most `INLINE_DIGITS` little-endian digits.
    ///
    /// # Panics
    ///
    /// Panics if `digits.len() > INLINE_DIGITS`.
    pub(crate) const fn const_from_digits(digits: &[Digit]) -> UBig {
        assert!(digits.len() <= INLINE_DIGITS);

        let mut buffer = [Digit::ZERO; INLINE_DIGITS];
        let mut len = min_len_unsigned(digits);
        let (dest, _) = buffer.split_at_mut(len);
        let (src, _) = digits.split_at(len);
        dest.copy_from_slice(src);

        // `UBig` always keeps at least one digit.
        if len == 0 {
            len = 1;
        }
        // SAFETY: `len <= INLINE_DIGITS`.
        UBig(unsafe { Digits::from_const_with_len_unchecked(buffer, len) })
    }

    /// Construct from little-endian digits and another digit.
    ///
    /// # Panics
    ///
    /// Panics if the resulting value has more than [`MAX_DIGITS`] digits.
    pub(crate) fn from_digits_digit(mut digits: Digits, digit: Digit) -> UBig {
        if digit != Digit::ZERO {
            digits.push(digit);
            // `digit` is a nonzero most-significant digit, so `digits` is already at
            // its canonical length and non-empty; only the buffer capacity may need shrinking.
            UBig::shrink(digits)
        } else {
            UBig::from_digits(digits)
        }
    }

    /// Constructs a [`UBig`] from the single digit `low` topped by a signed `idigit`,
    /// returning `None` when `idigit` is negative (the value would be negative).
    pub(crate) fn try_from_digit_idigit(low: Digit, idigit: IDigit) -> Option<UBig> {
        let high = Digit::try_from(idigit).ok()?;
        Some(UBig::from_two_digits(low, high))
    }

    /// Constructs a [`UBig`] from `digits` topped by a signed `idigit`, returning `None`
    /// when `idigit` is negative (the value would be negative).
    pub(crate) fn try_from_digits_idigit(digits: Digits, idigit: IDigit) -> Option<UBig> {
        // A non-negative `idigit` is the high digit above `digits`; a negative one has no `Digit`
        // value, so the conversion fails and `?` returns `None`.
        let digit = Digit::try_from(idigit).ok()?;
        Some(UBig::from_digits_digit(digits, digit))
    }

    /// Wraps `digits` — already trimmed to its canonical length and non-empty — as a `UBig`,
    /// shrinking an over-allocated heap buffer and enforcing the [`MAX_DIGITS`] cap.
    ///
    /// # Panics
    ///
    /// Panics if `digits` has more than [`MAX_DIGITS`] digits.
    fn shrink(mut digits: Digits) -> UBig {
        if digits.spilled() {
            let len = digits.len();
            if len > MAX_DIGITS {
                panic_number_too_large();
            }
            if len <= digits.capacity() / 4 || len == 1 {
                digits.shrink_to_fit();
            }
        }
        UBig(digits)
    }

    /// Panics because a result would be negative. The single source of the "negative UBig"
    /// panic message.
    #[cold]
    pub(crate) fn panic_negative() -> ! {
        panic!("negative UBig")
    }
}

/// Signed big integer.
///
/// An arbitrarily large signed integer.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IBig(
    /// The little-endian digits of the two's complement representation in canonical form:
    /// * the buffer is never empty
    /// * most-significant digit is not a redundant sign-extension of the digit below it
    /// * heap buffer is at least 25% used
    /// * a single digit is always stored inline
    Digits,
);

impl IBig {
    /// Construct from a single signed digit.
    pub(crate) fn from_digit(digit: IDigit) -> IBig {
        // A single signed digit is always canonical.
        IBig(smallvec![digit.cast_unsigned()])
    }

    /// Construct from a single signed digit, usable in `const` contexts.
    pub(crate) const fn const_from_digit(digit: IDigit) -> IBig {
        let mut buffer = [Digit::ZERO; INLINE_DIGITS];
        buffer[0] = digit.cast_unsigned();
        // A single signed digit is always canonical.
        // SAFETY: `1 <= INLINE_DIGITS`.
        IBig(unsafe { Digits::from_const_with_len_unchecked(buffer, 1) })
    }

    /// Construct from the two little-endian digits of a two's complement representation,
    /// where `high` carries the sign.
    pub(crate) fn from_two_digits(low: Digit, high: IDigit) -> IBig {
        if high == sign_extension_idigit(low.cast_signed()) {
            IBig::from_digit(low.cast_signed())
        } else {
            IBig(smallvec![low, high.cast_unsigned()])
        }
    }

    /// Construct from the little-endian digits of a two's complement representation.
    ///
    /// # Panics
    ///
    /// Panics if `digits` is empty, or if, after normalization, the value has more than
    /// [`MAX_DIGITS`] digits.
    pub(crate) fn from_digits(mut digits: Digits) -> IBig {
        digits.truncate(min_len_signed(&digits));
        IBig::shrink(digits)
    }

    /// Construct from at most `INLINE_DIGITS` little-endian two's complement digits.
    ///
    /// # Panics
    ///
    /// Panics if `digits` is empty or longer than `INLINE_DIGITS`.
    pub(crate) const fn const_from_digits(digits: &[Digit]) -> IBig {
        assert!(!digits.is_empty() && digits.len() <= INLINE_DIGITS);
        let mut buffer = [Digit::ZERO; INLINE_DIGITS];
        // `min_len_signed` is always at least 1, so the buffer keeps a sign-carrying digit.
        let len = min_len_signed(digits);
        let (dest, _) = buffer.split_at_mut(len);
        let (src, _) = digits.split_at(len);
        dest.copy_from_slice(src);
        // SAFETY: `1 <= len <= INLINE_DIGITS`.
        IBig(unsafe { Digits::from_const_with_len_unchecked(buffer, len) })
    }

    /// Construct from little-endian two's complement digits plus an extra digit `idigit`.
    ///
    /// # Panics
    ///
    /// Panics if `digits` is empty, or if, after normalization, the value has more than
    /// [`MAX_DIGITS`] digits.
    pub(crate) fn from_digits_idigit(mut digits: Digits, idigit: IDigit) -> IBig {
        if idigit != sign_extension(&digits) {
            // `icarry` is a non-redundant most-significant digit, so `digits` is already at its
            // canonical length; only the buffer capacity may need shrinking.
            digits.push(idigit.cast_unsigned());
            IBig::shrink(digits)
        } else {
            IBig::from_digits(digits)
        }
    }

    /// Wraps `digits` — already trimmed to its canonical length — as an `IBig`, shrinking an
    /// over-allocated heap buffer and enforcing the [`MAX_DIGITS`] cap.
    ///
    /// # Panics
    ///
    /// Panics if `digits` has more than [`MAX_DIGITS`] digits.
    fn shrink(mut digits: Digits) -> IBig {
        if digits.spilled() {
            let len = digits.len();
            if len > MAX_DIGITS {
                panic_number_too_large();
            }
            if len <= digits.capacity() / 4 || len == 1 {
                digits.shrink_to_fit();
            }
        }
        IBig(digits)
    }
}

/// Result of `AsDigits::as_digits` and `AsDigits::into_digits`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum AsDigitsResult<S, L> {
    /// The value fits in one digit.
    Small(S),
    /// The value doesn't fit in one digit.
    Large(L),
}

/// Access to the digit representation.
pub(crate) trait AsDigits: Default {
    /// The single-digit type.
    type SingleDigit;

    /// The little-endian digits, by reference.
    fn as_digits(&self) -> AsDigitsResult<Self::SingleDigit, &[Digit]>;

    /// Consume into the little-endian digits.
    fn into_digits(self) -> AsDigitsResult<Self::SingleDigit, Digits>;
}

impl AsDigits for UBig {
    type SingleDigit = Digit;

    fn as_digits(&self) -> AsDigitsResult<Digit, &[Digit]> {
        if !self.0.spilled() && self.0.len() == 1 {
            AsDigitsResult::Small(self.0[0])
        } else {
            let res = self.0.as_slice();
            // SAFETY: We never have 0 digits and 1 digit is always inline.
            unsafe { assert_unchecked(res.len() > 1) };
            AsDigitsResult::Large(res)
        }
    }

    fn into_digits(self) -> AsDigitsResult<Digit, Digits> {
        if !self.0.spilled() && self.0.len() == 1 {
            AsDigitsResult::Small(self.0[0])
        } else {
            // SAFETY: We never have 0 digits and 1 digit is always inline.
            unsafe { assert_unchecked(self.0.len() > 1) };
            AsDigitsResult::Large(self.0)
        }
    }
}

impl AsDigits for IBig {
    type SingleDigit = IDigit;

    fn as_digits(&self) -> AsDigitsResult<IDigit, &[Digit]> {
        if !self.0.spilled() && self.0.len() == 1 {
            AsDigitsResult::Small(self.0[0].cast_signed())
        } else {
            let res = self.0.as_slice();
            // SAFETY: We never have 0 digits and 1 digit is always inline.
            unsafe { assert_unchecked(res.len() > 1) };
            AsDigitsResult::Large(res)
        }
    }

    fn into_digits(self) -> AsDigitsResult<IDigit, Digits> {
        if !self.0.spilled() && self.0.len() == 1 {
            AsDigitsResult::Small(self.0[0].cast_signed())
        } else {
            // SAFETY: We never have 0 digits and 1 digit is always inline.
            unsafe { assert_unchecked(self.0.len() > 1) };
            AsDigitsResult::Large(self.0)
        }
    }
}

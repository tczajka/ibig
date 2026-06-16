//! Conversions to and from [`UBig`] and [`IBig`].

use crate::ops::{UnaryOpRefVal, UnaryOpRefValBig};
use crate::repr::Digits;
use crate::{IBig, TryFromBigError, UBig};
use core::num::TryFromIntError;
use ibig_core::{Digit, SignedDigit};

impl UBig {
    /// Constructs from a `u8` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_u8(value: u8) -> UBig {
        UBig::const_from_digit(Digit::from_u8(value))
    }

    /// Constructs from a `u16` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_u16(value: u16) -> UBig {
        UBig::const_from_digit(Digit::from_u16(value))
    }

    /// Constructs from a `u32` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_u32(value: u32) -> UBig {
        match Digit::try_from_u32(value) {
            Some(digit) => UBig::const_from_digit(digit),
            None => UBig::const_from_le_bytes(&value.to_le_bytes()),
        }
    }

    /// Constructs from a `u64` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_u64(value: u64) -> UBig {
        match Digit::try_from_u64(value) {
            Some(digit) => UBig::const_from_digit(digit),
            None => UBig::const_from_le_bytes(&value.to_le_bytes()),
        }
    }
}

impl IBig {
    /// Constructs from an `i8` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_i8(value: i8) -> IBig {
        IBig::const_from_digit(SignedDigit::from_i8(value))
    }

    /// Constructs from an `i16` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_i16(value: i16) -> IBig {
        IBig::const_from_digit(SignedDigit::from_i16(value))
    }

    /// Constructs from an `i32` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_i32(value: i32) -> IBig {
        match SignedDigit::try_from_i32(value) {
            Some(digit) => IBig::const_from_digit(digit),
            None => IBig::const_from_le_bytes(&value.to_le_bytes()),
        }
    }

    /// Constructs from an `i64` in a `const` context.
    ///
    /// Outside of `const` contexts, use [`From`].
    pub const fn const_from_i64(value: i64) -> IBig {
        match SignedDigit::try_from_i64(value) {
            Some(digit) => IBig::const_from_digit(digit),
            None => IBig::const_from_le_bytes(&value.to_le_bytes()),
        }
    }
}

/// Implements `From<$t> for UBig` for an unsigned primitive: a value that fits in a single
/// digit takes the fast path, otherwise it goes through the little-endian bytes.
macro_rules! ubig_from_unsigned {
    ($t:ty) => {
        impl From<$t> for UBig {
            fn from(value: $t) -> UBig {
                match Digit::try_from(value) {
                    Ok(digit) => UBig::from_digit(digit),
                    Err(_) => UBig::from_le_bytes(&value.to_le_bytes()),
                }
            }
        }
    };
}

ubig_from_unsigned!(u8);
ubig_from_unsigned!(u16);
ubig_from_unsigned!(u32);
ubig_from_unsigned!(u64);
ubig_from_unsigned!(u128);
ubig_from_unsigned!(usize);

/// Implements `TryFrom<$signed> for UBig` by forwarding through the unsigned `$unsigned`: a
/// non-negative value converts, while a negative value yields the same `TryFromIntError`
/// that the unsigned conversion produces.
macro_rules! ubig_try_from_signed {
    ($signed:ty => $unsigned:ty) => {
        impl TryFrom<$signed> for UBig {
            type Error = TryFromIntError;

            fn try_from(value: $signed) -> Result<UBig, TryFromIntError> {
                <$unsigned>::try_from(value).map(UBig::from)
            }
        }
    };
}

ubig_try_from_signed!(i8 => u8);
ubig_try_from_signed!(i16 => u16);
ubig_try_from_signed!(i32 => u32);
ubig_try_from_signed!(i64 => u64);
ubig_try_from_signed!(i128 => u128);
ubig_try_from_signed!(isize => usize);

/// Converts a `bool`: `false` to zero and `true` to one.
impl From<bool> for UBig {
    fn from(value: bool) -> UBig {
        UBig::from_digit(Digit::from(value))
    }
}

/// Converts a `char` to its Unicode scalar value (code point).
impl From<char> for UBig {
    fn from(value: char) -> UBig {
        UBig::from(u32::from(value))
    }
}

/// Implements the [`UBig`]-to-`$t` conversion for an unsigned primitive through a `$marker`
/// type implementing [`UnaryOpRefValBig`].
macro_rules! try_from_ubig_unsigned {
    ($t:ty, $marker:ident) => {
        enum $marker {}

        impl UnaryOpRefValBig for $marker {
            type Operand = UBig;
            type Output = Result<$t, TryFromBigError>;

            fn apply_digit(digit: Digit) -> Result<$t, TryFromBigError> {
                <$t>::try_from(digit).map_err(|_| TryFromBigError)
            }

            fn apply_ref(digits: &[Digit]) -> Result<$t, TryFromBigError> {
                const N: usize = size_of::<$t>();
                const {
                    assert!(Digit::BYTES.is_power_of_two());
                    assert!(N.is_power_of_two());
                }

                // The minimum required number of bits is b:
                // b > (len - 1) * Digit::BITS
                // b <= len * Digit::BITS
                //
                // Since len >= 2 and Digit::BITS is a power of two:
                // next_power_of_two(b) = next_power_of_two(len * Digit::BITS)
                //
                // If the number fits, we must have:
                // b <= N * 8
                // next_power_of_two(b) <= N * 8
                // len * Digit::BITS <= N * 8
                // len * Digit::BYTES <= N

                // Compile-time fast path: two-digit values are too large for the target type.
                if 2 * Digit::BYTES > N {
                    return Err(TryFromBigError);
                }

                let num_bytes = digits.len() * Digit::BYTES;
                if num_bytes > N {
                    return Err(TryFromBigError);
                }
                let mut arr = [0u8; N];
                ibig_core::to_bytes_unsigned(digits, &mut arr);
                Ok(<$t>::from_le_bytes(arr))
            }

            fn apply_val(digits: Digits) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefValBig>::apply_ref(&digits)
            }
        }

        impl TryFrom<UBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: UBig) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_val(value)
            }
        }

        impl TryFrom<&UBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: &UBig) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_ref(value)
            }
        }
    };
}

try_from_ubig_unsigned!(u8, UBigToU8);
try_from_ubig_unsigned!(u16, UBigToU16);
try_from_ubig_unsigned!(u32, UBigToU32);
try_from_ubig_unsigned!(u64, UBigToU64);
try_from_ubig_unsigned!(u128, UBigToU128);
try_from_ubig_unsigned!(usize, UBigToUsize);

/// Implements the [`UBig`]-to-`$signed` conversion for a signed primitive through a `$marker`
/// type implementing [`UnaryOpRefValBig`].
macro_rules! try_from_ubig_signed {
    ($signed:ty, $marker:ident, $unsigned_marker:ident) => {
        enum $marker {}

        impl UnaryOpRefValBig for $marker {
            type Operand = UBig;
            type Output = Result<$signed, TryFromBigError>;

            fn apply_digit(digit: Digit) -> Result<$signed, TryFromBigError> {
                <$signed>::try_from(digit).map_err(|_| TryFromBigError)
            }

            fn apply_ref(digits: &[Digit]) -> Result<$signed, TryFromBigError> {
                let unsigned = <$unsigned_marker as UnaryOpRefValBig>::apply_ref(digits)?;
                <$signed>::try_from(unsigned).map_err(|_| TryFromBigError)
            }

            fn apply_val(digits: Digits) -> Result<$signed, TryFromBigError> {
                <$marker as UnaryOpRefValBig>::apply_ref(&digits)
            }
        }

        impl TryFrom<UBig> for $signed {
            type Error = TryFromBigError;

            fn try_from(value: UBig) -> Result<$signed, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_val(value)
            }
        }

        impl TryFrom<&UBig> for $signed {
            type Error = TryFromBigError;

            fn try_from(value: &UBig) -> Result<$signed, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_ref(value)
            }
        }
    };
}

try_from_ubig_signed!(i8, UBigToI8, UBigToU8);
try_from_ubig_signed!(i16, UBigToI16, UBigToU16);
try_from_ubig_signed!(i32, UBigToI32, UBigToU32);
try_from_ubig_signed!(i64, UBigToI64, UBigToU64);
try_from_ubig_signed!(i128, UBigToI128, UBigToU128);
try_from_ubig_signed!(isize, UBigToIsize, UBigToUsize);

/// The [`UBig`]-to-`bool` conversion: zero is `false`, one is `true`, anything else is out
/// of range.
enum UBigToBool {}

impl UnaryOpRefValBig for UBigToBool {
    type Operand = UBig;
    type Output = Result<bool, TryFromBigError>;

    fn apply_digit(operand: Digit) -> Result<bool, TryFromBigError> {
        operand.try_into().map_err(|_| TryFromBigError)
    }

    fn apply_ref(_operand: &[Digit]) -> Result<bool, TryFromBigError> {
        // A multi-digit value exceeds one, out of range for `bool`.
        Err(TryFromBigError)
    }

    fn apply_val(_operand: Digits) -> Result<bool, TryFromBigError> {
        // A multi-digit value exceeds one, out of range for `bool`.
        Err(TryFromBigError)
    }
}

impl TryFrom<UBig> for bool {
    type Error = TryFromBigError;

    fn try_from(value: UBig) -> Result<bool, TryFromBigError> {
        <UBigToBool as UnaryOpRefVal>::apply_val(value)
    }
}

impl TryFrom<&UBig> for bool {
    type Error = TryFromBigError;

    fn try_from(value: &UBig) -> Result<bool, TryFromBigError> {
        <UBigToBool as UnaryOpRefVal>::apply_ref(value)
    }
}

/// Implements `From<$t> for IBig` for a signed primitive: a value that fits in a single
/// digit takes the fast path, otherwise it goes through the little-endian bytes.
macro_rules! ibig_from_signed {
    ($t:ty) => {
        impl From<$t> for IBig {
            fn from(value: $t) -> IBig {
                match SignedDigit::try_from(value) {
                    Ok(digit) => IBig::from_digit(digit),
                    Err(_) => IBig::from_le_bytes(&value.to_le_bytes()),
                }
            }
        }
    };
}

ibig_from_signed!(i8);
ibig_from_signed!(i16);
ibig_from_signed!(i32);
ibig_from_signed!(i64);
ibig_from_signed!(i128);
ibig_from_signed!(isize);

/// Implements `From<$t> for IBig` for an unsigned primitive by converting through `UBig`.
macro_rules! ibig_from_unsigned {
    ($t:ty) => {
        impl From<$t> for IBig {
            fn from(value: $t) -> IBig {
                IBig::from(UBig::from(value))
            }
        }
    };
}

ibig_from_unsigned!(u8);
ibig_from_unsigned!(u16);
ibig_from_unsigned!(u32);
ibig_from_unsigned!(u64);
ibig_from_unsigned!(u128);
ibig_from_unsigned!(usize);

/// Converts a `bool`: `false` to zero and `true` to one.
impl From<bool> for IBig {
    fn from(value: bool) -> IBig {
        IBig::from_digit(SignedDigit::from(value))
    }
}

/// Implements the [`IBig`]-to-`$t` conversion for a signed primitive through a `$marker` type
/// implementing [`UnaryOpRefValBig`].
macro_rules! try_from_ibig_signed {
    ($t:ty, $marker:ident) => {
        enum $marker {}

        impl UnaryOpRefValBig for $marker {
            type Operand = IBig;
            type Output = Result<$t, TryFromBigError>;

            fn apply_digit(digit: SignedDigit) -> Result<$t, TryFromBigError> {
                <$t>::try_from(digit).map_err(|_| TryFromBigError)
            }

            fn apply_ref(digits: &[Digit]) -> Result<$t, TryFromBigError> {
                const N: usize = size_of::<$t>();
                const {
                    assert!(Digit::BYTES.is_power_of_two());
                    assert!(N.is_power_of_two());
                }

                // The minimum required number of bits is b:
                // b > (len - 1) * Digit::BITS
                // b <= len * Digit::BITS
                //
                // Since len >= 2 and Digit::BITS is a power of two:
                // next_power_of_two(b) = next_power_of_two(len * Digit::BITS)
                //
                // If the number fits, we must have:
                // b <= N * 8
                // next_power_of_two(b) <= N * 8
                // len * Digit::BITS <= N * 8
                // len * Digit::BYTES <= N

                // Compile-time fast path: two-digit values are too large for the target type.
                if 2 * Digit::BYTES > N {
                    return Err(TryFromBigError);
                }

                let num_bytes = digits.len() * Digit::BYTES;
                if num_bytes > N {
                    return Err(TryFromBigError);
                }
                let mut arr = [0u8; N];
                ibig_core::to_bytes_signed(digits, &mut arr);
                Ok(<$t>::from_le_bytes(arr))
            }

            fn apply_val(digits: Digits) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefValBig>::apply_ref(&digits)
            }
        }

        impl TryFrom<IBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: IBig) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_val(value)
            }
        }

        impl TryFrom<&IBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: &IBig) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_ref(value)
            }
        }
    };
}

try_from_ibig_signed!(i8, IBigToI8);
try_from_ibig_signed!(i16, IBigToI16);
try_from_ibig_signed!(i32, IBigToI32);
try_from_ibig_signed!(i64, IBigToI64);
try_from_ibig_signed!(i128, IBigToI128);
try_from_ibig_signed!(isize, IBigToIsize);

/// Implements the [`IBig`]-to-`$t` conversion for an unsigned primitive through a `$marker`
/// type implementing [`UnaryOpRefValBig`].
macro_rules! try_from_ibig_unsigned {
    ($t:ty, $marker:ident) => {
        enum $marker {}

        impl UnaryOpRefValBig for $marker {
            type Operand = IBig;
            type Output = Result<$t, TryFromBigError>;

            fn apply_digit(digit: SignedDigit) -> Result<$t, TryFromBigError> {
                <$t>::try_from(digit).map_err(|_| TryFromBigError)
            }

            fn apply_ref(digits: &[Digit]) -> Result<$t, TryFromBigError> {
                // A negative value is out of range for any unsigned type.
                if ibig_core::is_negative(digits) {
                    return Err(TryFromBigError);
                }
                // A non-negative value carries at most one most-significant sign-extension
                // zero digit; drop it.
                let (&top, rest) = digits.split_last().unwrap();
                let digits = if top == Digit::ZERO { rest } else { digits };

                const N: usize = size_of::<$t>();
                const {
                    assert!(Digit::BYTES.is_power_of_two());
                    assert!(N.is_power_of_two());
                }

                // The minimum required number of bits is b.
                // For len >= 2:
                // b > (len - 1) * Digit::BITS
                // b <= len * Digit::BITS
                //
                // For len = 1 the top digit's high bit is set (since we don't fit in a single
                // signed digit, otherwise we would have used the fast path).
                // b = len * Digit::BITS
                //
                // In either case:
                // next_power_of_two(b) = next_power_of_two(len * Digit::BITS)
                //
                // If the number fits, we must have:
                // b <= N * 8
                // next_power_of_two(b) <= N * 8
                // len * Digit::BITS <= N * 8
                // len * Digit::BYTES <= N
                let num_bytes = digits.len() * Digit::BYTES;
                if num_bytes > N {
                    return Err(TryFromBigError);
                }
                let mut arr = [0u8; N];
                ibig_core::to_bytes_unsigned(digits, &mut arr);
                Ok(<$t>::from_le_bytes(arr))
            }

            fn apply_val(digits: Digits) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefValBig>::apply_ref(&digits)
            }
        }

        impl TryFrom<IBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: IBig) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_val(value)
            }
        }

        impl TryFrom<&IBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: &IBig) -> Result<$t, TryFromBigError> {
                <$marker as UnaryOpRefVal>::apply_ref(value)
            }
        }
    };
}

try_from_ibig_unsigned!(u8, IBigToU8);
try_from_ibig_unsigned!(u16, IBigToU16);
try_from_ibig_unsigned!(u32, IBigToU32);
try_from_ibig_unsigned!(u64, IBigToU64);
try_from_ibig_unsigned!(u128, IBigToU128);
try_from_ibig_unsigned!(usize, IBigToUsize);

/// The [`IBig`]-to-`bool` conversion: zero is `false`, one is `true`, anything else is out
/// of range.
enum IBigToBool {}

impl UnaryOpRefValBig for IBigToBool {
    type Operand = IBig;
    type Output = Result<bool, TryFromBigError>;

    fn apply_digit(operand: SignedDigit) -> Result<bool, TryFromBigError> {
        operand.try_into().map_err(|_| TryFromBigError)
    }

    fn apply_ref(_operand: &[Digit]) -> Result<bool, TryFromBigError> {
        // A multi-digit value is not 0 or 1, out of range for `bool`.
        Err(TryFromBigError)
    }

    fn apply_val(_operand: Digits) -> Result<bool, TryFromBigError> {
        // A multi-digit value is not 0 or 1, out of range for `bool`.
        Err(TryFromBigError)
    }
}

impl TryFrom<IBig> for bool {
    type Error = TryFromBigError;

    fn try_from(value: IBig) -> Result<bool, TryFromBigError> {
        <IBigToBool as UnaryOpRefVal>::apply_val(value)
    }
}

impl TryFrom<&IBig> for bool {
    type Error = TryFromBigError;

    fn try_from(value: &IBig) -> Result<bool, TryFromBigError> {
        <IBigToBool as UnaryOpRefVal>::apply_ref(value)
    }
}

/// The fallible [`IBig`]-to-[`UBig`] conversion.
enum IBigToUBig {}

impl UnaryOpRefValBig for IBigToUBig {
    type Operand = IBig;
    type Output = Result<UBig, TryFromBigError>;

    fn apply_digit(digit: SignedDigit) -> Result<UBig, TryFromBigError> {
        if digit.is_negative() {
            Err(TryFromBigError)
        } else {
            Ok(UBig::from_digit(digit.cast_unsigned()))
        }
    }

    fn apply_ref(digits: &[Digit]) -> Result<UBig, TryFromBigError> {
        if ibig_core::is_negative(digits) {
            Err(TryFromBigError)
        } else {
            // A non-negative two's complement value's digits are its unsigned magnitude.
            Ok(UBig::from_digits(Digits::from_slice(digits)))
        }
    }

    fn apply_val(digits: Digits) -> Result<UBig, TryFromBigError> {
        if ibig_core::is_negative(&digits) {
            Err(TryFromBigError)
        } else {
            // A non-negative two's complement value's digits are its unsigned magnitude.
            Ok(UBig::from_digits(digits))
        }
    }
}

impl TryFrom<IBig> for UBig {
    type Error = TryFromBigError;

    fn try_from(value: IBig) -> Result<UBig, TryFromBigError> {
        <IBigToUBig as UnaryOpRefVal>::apply_val(value)
    }
}

impl TryFrom<&IBig> for UBig {
    type Error = TryFromBigError;

    fn try_from(value: &IBig) -> Result<UBig, TryFromBigError> {
        <IBigToUBig as UnaryOpRefVal>::apply_ref(value)
    }
}

/// The [`UBig`]-to-[`IBig`] conversion.
enum UBigToIBig {}

impl UnaryOpRefValBig for UBigToIBig {
    type Operand = UBig;
    type Output = IBig;

    fn apply_digit(digit: Digit) -> IBig {
        // A zero high digit keeps the value non-negative.
        IBig::from_two_digits(digit, SignedDigit::ZERO)
    }

    fn apply_ref(digits: &[Digit]) -> IBig {
        // If the top digit's sign bit is set, a zero digit is appended to keep the
        // two's complement reading positive; clone with room for it.
        let negative = ibig_core::is_negative(digits);
        let mut new_digits = Digits::with_capacity(digits.len() + usize::from(negative));
        new_digits.extend_from_slice(digits);
        if negative {
            new_digits.push(Digit::ZERO);
        }
        IBig::from_digits(new_digits)
    }

    fn apply_val(mut digits: Digits) -> IBig {
        if ibig_core::is_negative(&digits) {
            digits.push(Digit::ZERO);
        }
        IBig::from_digits(digits)
    }
}

impl From<UBig> for IBig {
    fn from(value: UBig) -> IBig {
        <UBigToIBig as UnaryOpRefVal>::apply_val(value)
    }
}

impl From<&UBig> for IBig {
    fn from(value: &UBig) -> IBig {
        <UBigToIBig as UnaryOpRefVal>::apply_ref(value)
    }
}

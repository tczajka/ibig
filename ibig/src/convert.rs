//! Conversions to and from [`UBig`] and [`IBig`].

use crate::ops::{UnaryOpRefVal, UnaryOpRefValBig};
use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
    Digits,
};
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

/// Forwards `TryFrom<$to> for $from` to the by-reference conversion.
macro_rules! try_from_big_value {
    ($to:ty, $from:ty) => {
        impl TryFrom<$from> for $to {
            type Error = TryFromBigError;

            fn try_from(value: $from) -> Result<Self, TryFromBigError> {
                Self::try_from(&value)
            }
        }
    };
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

/// Implements `TryFrom<UBig> for $t` for an unsigned primitive. A single-digit value is
/// converted directly; a larger value is read from its little-endian bytes.
macro_rules! try_from_ubig_unsigned {
    ($t:ty) => {
        impl TryFrom<&UBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: &UBig) -> Result<$t, TryFromBigError> {
                match value.as_digits() {
                    Small(digit) => <$t>::try_from(digit).map_err(|_| TryFromBigError),
                    Large(digits) => {
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
                }
            }
        }

        try_from_big_value!($t, UBig);
    };
}

try_from_ubig_unsigned!(u8);
try_from_ubig_unsigned!(u16);
try_from_ubig_unsigned!(u32);
try_from_ubig_unsigned!(u64);
try_from_ubig_unsigned!(u128);
try_from_ubig_unsigned!(usize);

/// Implements `TryFrom<UBig> for $signed` for a signed primitive: a single-digit value is
/// converted directly; otherwise the value is converted to the same-width unsigned type
/// `$unsigned` and then narrowed (which rejects values past the signed maximum).
macro_rules! try_from_ubig_signed {
    ($signed:ty => $unsigned:ty) => {
        impl TryFrom<&UBig> for $signed {
            type Error = TryFromBigError;

            fn try_from(value: &UBig) -> Result<$signed, TryFromBigError> {
                // Fast path.
                if let Small(digit) = value.as_digits() {
                    return <$signed>::try_from(digit).map_err(|_| TryFromBigError);
                }
                // Slow path.
                let unsigned = <$unsigned>::try_from(value)?;
                <$signed>::try_from(unsigned).map_err(|_| TryFromBigError)
            }
        }

        try_from_big_value!($signed, UBig);
    };
}

try_from_ubig_signed!(i8 => u8);
try_from_ubig_signed!(i16 => u16);
try_from_ubig_signed!(i32 => u32);
try_from_ubig_signed!(i64 => u64);
try_from_ubig_signed!(i128 => u128);
try_from_ubig_signed!(isize => usize);

/// Converts to `bool`: zero is `false`, one is `true`, anything else is out of range.
impl TryFrom<&UBig> for bool {
    type Error = TryFromBigError;

    fn try_from(value: &UBig) -> Result<bool, TryFromBigError> {
        match value.as_digits() {
            Small(digit) => digit.try_into().map_err(|_| TryFromBigError),
            Large(_) => Err(TryFromBigError),
        }
    }
}

try_from_big_value!(bool, UBig);

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

/// Implements `TryFrom<IBig> for $t` for a signed primitive. A single-digit value is
/// converted directly; a larger value is read from its sign-extended little-endian bytes.
macro_rules! signed_from_ibig {
    ($t:ty) => {
        impl TryFrom<&IBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: &IBig) -> Result<$t, TryFromBigError> {
                match value.as_digits() {
                    Small(digit) => <$t>::try_from(digit).map_err(|_| TryFromBigError),
                    Large(digits) => {
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

                        // Slow path.
                        let num_bytes = digits.len() * Digit::BYTES;
                        if num_bytes > N {
                            return Err(TryFromBigError);
                        }
                        let mut arr = [0u8; N];
                        ibig_core::to_bytes_signed(digits, &mut arr);
                        Ok(<$t>::from_le_bytes(arr))
                    }
                }
            }
        }

        try_from_big_value!($t, IBig);
    };
}

signed_from_ibig!(i8);
signed_from_ibig!(i16);
signed_from_ibig!(i32);
signed_from_ibig!(i64);
signed_from_ibig!(i128);
signed_from_ibig!(isize);

/// Implements `TryFrom<IBig> for $t` for an unsigned primitive. A single-digit value is
/// converted directly; a larger value must be non-negative and is read from the little-endian
/// bytes of its unsigned magnitude.
macro_rules! unsigned_from_ibig {
    ($t:ty) => {
        impl TryFrom<&IBig> for $t {
            type Error = TryFromBigError;

            fn try_from(value: &IBig) -> Result<$t, TryFromBigError> {
                match value.as_digits() {
                    Small(digit) => <$t>::try_from(digit).map_err(|_| TryFromBigError),
                    Large(digits) => {
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
                }
            }
        }

        try_from_big_value!($t, IBig);
    };
}

unsigned_from_ibig!(u8);
unsigned_from_ibig!(u16);
unsigned_from_ibig!(u32);
unsigned_from_ibig!(u64);
unsigned_from_ibig!(u128);
unsigned_from_ibig!(usize);

/// Converts to `bool`: zero is `false`, one is `true`, anything else is out of range.
impl TryFrom<&IBig> for bool {
    type Error = TryFromBigError;

    fn try_from(value: &IBig) -> Result<bool, TryFromBigError> {
        match value.as_digits() {
            Small(digit) => digit.try_into().map_err(|_| TryFromBigError),
            Large(_) => Err(TryFromBigError),
        }
    }
}

try_from_big_value!(bool, IBig);

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

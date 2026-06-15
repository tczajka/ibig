//! Byte-sequence conversions for [`UBig`] and [`IBig`].

use crate::ops::{UnaryOpRef, UnaryOpRefBig};
use crate::repr::{Digits, INLINE_DIGITS};
use crate::{IBig, UBig};
use alloc::{vec, vec::Vec};
use ibig_core::{Digit, SignedDigit};

impl UBig {
    /// Returns the little-endian (least-significant-first) byte representation, with no
    /// most-significant zero bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from(0x0105u16).to_le_bytes(), [5, 1]);
    /// assert_eq!(UBig::from(0u8).to_le_bytes(), []);
    /// ```
    pub fn to_le_bytes(&self) -> Vec<u8> {
        <ToLeBytesUBig as UnaryOpRef>::apply_ref(self)
    }

    /// Returns the big-endian (most-significant-first) byte representation, with no leading
    /// zero bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from(0x0105u16).to_be_bytes(), [1, 5]);
    /// assert_eq!(UBig::from(0u8).to_be_bytes(), []);
    /// ```
    pub fn to_be_bytes(&self) -> Vec<u8> {
        // Big-endian is the little-endian representation reversed.
        let mut bytes = self.to_le_bytes();
        bytes.reverse();
        bytes
    }

    /// Constructs a number from its little-endian (least-significant-first) byte
    /// representation. Any length is accepted, and most-significant zero bytes are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from_le_bytes(&[5, 1]), UBig::from(0x0105u16));
    /// ```
    pub fn from_le_bytes(bytes: &[u8]) -> UBig {
        let mut digits = Digits::new();
        digits.resize(bytes.len().div_ceil(Digit::BYTES), Digit::ZERO);
        ibig_core::from_bytes_unsigned(bytes, &mut digits);
        UBig::from_digits(digits)
    }

    /// Constructs from at most `INLINE_DIGITS * Digit::BYTES` little-endian bytes.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` is longer than `INLINE_DIGITS * Digit::BYTES`.
    pub(crate) const fn const_from_le_bytes(bytes: &[u8]) -> UBig {
        assert!(bytes.len() <= INLINE_DIGITS * Digit::BYTES);
        let mut digits = [Digit::ZERO; INLINE_DIGITS];
        let n = bytes.len().div_ceil(Digit::BYTES);
        let (used, _) = digits.split_at_mut(n);
        ibig_core::from_bytes_unsigned(bytes, used);
        UBig::const_from_digits(used)
    }

    /// Constructs a number from its big-endian (most-significant-first) byte representation.
    /// Any length is accepted, and leading zero bytes are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::UBig;
    /// assert_eq!(UBig::from_be_bytes(&[1, 5]), UBig::from(0x0105u16));
    /// ```
    pub fn from_be_bytes(bytes: &[u8]) -> UBig {
        let mut digits = Digits::new();
        digits.resize(bytes.len().div_ceil(Digit::BYTES), Digit::ZERO);
        ibig_core::from_be_bytes_unsigned(bytes, &mut digits);
        UBig::from_digits(digits)
    }
}

impl IBig {
    /// Returns the little-endian (least-significant-first) two's complement byte
    /// representation, with no redundant sign-extension bytes. The result
    /// always has at least one byte.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from(0i8).to_le_bytes(), [0]);
    /// assert_eq!(IBig::from(0x0102i16).to_le_bytes(), [2, 1]);
    /// assert_eq!(IBig::from(0xffffi32).to_le_bytes(), [0xff, 0xff, 0]);
    /// assert_eq!(IBig::from(-1i8).to_le_bytes(), [0xff]);
    /// ```
    pub fn to_le_bytes(&self) -> Vec<u8> {
        <ToLeBytesIBig as UnaryOpRef>::apply_ref(self)
    }

    /// Returns the big-endian (most-significant-first) two's complement byte representation,
    /// with no redundant leading sign-extension bytes. The result always has at least one
    /// byte.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from(0i8).to_be_bytes(), [0]);
    /// assert_eq!(IBig::from(0x0102i16).to_be_bytes(), [1, 2]);
    /// assert_eq!(IBig::from(0xffffi32).to_be_bytes(), [0, 0xff, 0xff]);
    /// assert_eq!(IBig::from(-1i8).to_be_bytes(), [0xff]);
    /// ```
    pub fn to_be_bytes(&self) -> Vec<u8> {
        let mut bytes = self.to_le_bytes();
        bytes.reverse();
        bytes
    }

    /// Constructs a number from its little-endian (least-significant-first) two's complement
    /// byte representation.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` is empty: a signed value needs at least one byte to carry its sign.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from_le_bytes(&[1, 2, 0]), IBig::from(0x0201i16));
    /// assert_eq!(IBig::from_le_bytes(&[0xff]), IBig::from(-1i8));
    /// ```
    pub fn from_le_bytes(bytes: &[u8]) -> IBig {
        let mut digits = Digits::new();
        digits.resize(bytes.len().div_ceil(Digit::BYTES), Digit::ZERO);
        ibig_core::from_bytes_signed(bytes, &mut digits);
        IBig::from_digits(digits)
    }

    /// Constructs from at most `INLINE_DIGITS * Digit::BYTES` little-endian two's complement
    /// bytes.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` is empty or longer than `INLINE_DIGITS * Digit::BYTES`.
    pub(crate) const fn const_from_le_bytes(bytes: &[u8]) -> IBig {
        assert!(bytes.len() <= INLINE_DIGITS * Digit::BYTES);
        let mut digits = [Digit::ZERO; INLINE_DIGITS];
        let n = bytes.len().div_ceil(Digit::BYTES);
        let (used, _) = digits.split_at_mut(n);
        ibig_core::from_bytes_signed(bytes, used);
        IBig::const_from_digits(used)
    }

    /// Constructs a number from its big-endian (most-significant-first) two's complement
    /// byte representation.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` is empty: a signed value needs at least one byte to carry its sign.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig::IBig;
    /// assert_eq!(IBig::from_be_bytes(&[0, 1, 2]), IBig::from(0x0102i16));
    /// assert_eq!(IBig::from_be_bytes(&[0xff]), IBig::from(-1i8));
    /// ```
    pub fn from_be_bytes(bytes: &[u8]) -> IBig {
        let mut digits = Digits::new();
        digits.resize(bytes.len().div_ceil(Digit::BYTES), Digit::ZERO);
        ibig_core::from_be_bytes_signed(bytes, &mut digits);
        IBig::from_digits(digits)
    }
}

/// The [`UBig::to_le_bytes`] operation.
enum ToLeBytesUBig {}

impl UnaryOpRefBig for ToLeBytesUBig {
    type Operand = UBig;
    type Output = Vec<u8>;

    fn apply_digit(operand: Digit) -> Vec<u8> {
        let mut bytes = operand.to_le_bytes().to_vec();
        bytes.truncate(ibig_core::min_len_bytes_unsigned(&bytes));
        bytes
    }

    fn apply_ref(operand: &[Digit]) -> Vec<u8> {
        let mut bytes = vec![0u8; operand.len() * Digit::BYTES];
        ibig_core::to_bytes_unsigned(operand, &mut bytes);
        bytes.truncate(ibig_core::min_len_bytes_unsigned(&bytes));
        bytes
    }
}

/// The [`IBig::to_le_bytes`] operation.
enum ToLeBytesIBig {}

impl UnaryOpRefBig for ToLeBytesIBig {
    type Operand = IBig;
    type Output = Vec<u8>;

    fn apply_digit(operand: SignedDigit) -> Vec<u8> {
        let mut bytes = operand.to_le_bytes().to_vec();
        bytes.truncate(ibig_core::min_len_bytes_signed(&bytes));
        bytes
    }

    fn apply_ref(operand: &[Digit]) -> Vec<u8> {
        let mut bytes = vec![0u8; operand.len() * Digit::BYTES];
        ibig_core::to_bytes_unsigned(operand, &mut bytes);
        bytes.truncate(ibig_core::min_len_bytes_signed(&bytes));
        bytes
    }
}

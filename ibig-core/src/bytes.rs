//! Byte serialization of digit slices.

use crate::Digit;
use crate::sign::{extend_signed_bytes, sign_extension_byte};

/// Writes the little-endian byte representation of `digits` into `bytes`, zero-extending to
/// fill `bytes`.
///
/// # Panics
///
/// Panics if `bytes.len() < digits.len() * Digit::BYTES`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, to_bytes_unsigned};
/// let mut bytes = vec![0xffu8; 20];
/// to_bytes_unsigned(&[Digit::from(0x0102u16)], &mut bytes);
/// assert_eq!(&bytes[..2], &[0x02, 0x01]);
/// assert!(bytes[2..].iter().all(|&b| b == 0));
/// ```
pub fn to_bytes_unsigned(digits: &[Digit], bytes: &mut [u8]) {
    let len = to_bytes_prefix(digits, bytes);
    bytes[len..].fill(0);
}

/// Writes the little-endian signed byte representation of `digits` into `bytes`,
/// sign-extending to fill `bytes`.
///
/// `bytes.len()` must be at least `digits.len() * Digit::BYTES`.
///
/// # Panics
///
/// Panics if `digits` is empty or `bytes.len() < digits.len() * Digit::BYTES`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, to_bytes_signed};
/// let mut bytes = vec![0u8; Digit::BYTES + 1];
/// to_bytes_signed(&[Digit::MAX], &mut bytes);
/// assert!(bytes.iter().all(|&b| b == 0xff));
/// ```
pub fn to_bytes_signed(digits: &[Digit], bytes: &mut [u8]) {
    let len = to_bytes_prefix(digits, bytes);
    extend_signed_bytes(bytes, len);
}

/// Writes `digits` into the low `digits.len() * Digit::BYTES` bytes of `bytes`,
/// returning that count. The remaining bytes are left unchanged.
///
/// # Panics
///
/// Panics if `bytes.len() < digits.len() * Digit::BYTES`.
fn to_bytes_prefix(digits: &[Digit], bytes: &mut [u8]) -> usize {
    let len = digits.len() * Digit::BYTES;
    let (chunks, _) = bytes[..len].as_chunks_mut::<{ Digit::BYTES }>();
    for (chunk, &digit) in chunks.iter_mut().zip(digits) {
        *chunk = digit.to_le_bytes();
    }
    len
}

/// Fills `digits` from the little-endian `bytes`.
///
/// `digits.len()` must equal `bytes.len().div_ceil(Digit::BYTES)`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, from_bytes_unsigned};
/// // Two bytes fit in a single digit at every digit width.
/// let mut digits = [Digit::ZERO];
/// from_bytes_unsigned(&[0x02, 0x01], &mut digits);
/// assert_eq!(digits, [Digit::from(0x0102u16)]);
/// ```
pub const fn from_bytes_unsigned(bytes: &[u8], digits: &mut [Digit]) {
    assert!(digits.len() == bytes.len().div_ceil(Digit::BYTES));
    let (chunks, rem) = bytes.as_chunks::<{ Digit::BYTES }>();
    let mut i = 0;
    while i < chunks.len() {
        digits[i] = Digit::from_le_bytes(chunks[i]);
        i += 1;
    }
    if !rem.is_empty() {
        let mut arr = [0u8; Digit::BYTES];
        let (dest, _) = arr.split_at_mut(rem.len());
        dest.copy_from_slice(rem);
        digits[i] = Digit::from_le_bytes(arr);
        i += 1;
    }
    assert!(i == digits.len());
}

/// Fills `digits` from the big-endian `bytes`.
///
/// `digits.len()` must equal `bytes.len().div_ceil(Digit::BYTES)`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, from_be_bytes_unsigned};
/// // Two bytes fit in a single digit at every digit width.
/// let mut digits = [Digit::ZERO; 1];
/// from_be_bytes_unsigned(&[0x01, 0x02], &mut digits);
/// assert_eq!(digits, [Digit::from(0x0102u16)]);
/// ```
pub fn from_be_bytes_unsigned(bytes: &[u8], digits: &mut [Digit]) {
    assert_eq!(digits.len(), bytes.len().div_ceil(Digit::BYTES));
    let mut digit_iter = digits.iter_mut();
    let (rem, chunks) = bytes.as_rchunks::<{ Digit::BYTES }>();
    for &chunk in chunks.iter().rev() {
        *digit_iter.next().unwrap() = Digit::from_be_bytes(chunk);
    }
    if !rem.is_empty() {
        let mut arr = [0u8; Digit::BYTES];
        arr[Digit::BYTES - rem.len()..].copy_from_slice(rem);
        *digit_iter.next().unwrap() = Digit::from_be_bytes(arr);
    }
    assert!(digit_iter.next().is_none());
}

/// Fills `digits` from the little-endian signed `bytes`.
///
/// `digits.len()` must equal `bytes.len().div_ceil(Digit::BYTES)`.
///
/// # Panics
///
/// Panics if `bytes` is empty: a signed value needs at least one byte to carry its sign.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, from_bytes_signed};
/// let mut digits = [Digit::ZERO; 1];
/// from_bytes_signed(&[1, 2], &mut digits);
/// assert_eq!(digits, [Digit::from(0x0201u16)]);
/// from_bytes_signed(&[0xff], &mut digits);
/// assert_eq!(digits, [Digit::MAX]);
/// ```
pub const fn from_bytes_signed(bytes: &[u8], digits: &mut [Digit]) {
    assert!(!bytes.is_empty());
    assert!(digits.len() == bytes.len().div_ceil(Digit::BYTES));

    let (chunks, rem) = bytes.as_chunks::<{ Digit::BYTES }>();
    let mut i = 0;
    while i < chunks.len() {
        digits[i] = Digit::from_le_bytes(chunks[i]);
        i += 1;
    }
    if let Some(&last) = rem.last() {
        let fill = sign_extension_byte(last.cast_signed()).cast_unsigned();
        let mut arr = [fill; Digit::BYTES];
        let (dest, _) = arr.split_at_mut(rem.len());
        dest.copy_from_slice(rem);
        digits[i] = Digit::from_le_bytes(arr);
        i += 1;
    }
    assert!(i == digits.len());
}

/// Fills `digits` from the big-endian signed `bytes`, sign-extending the
/// most-significant digit.
///
/// `digits.len()` must equal `bytes.len().div_ceil(Digit::BYTES)`.
///
/// # Panics
///
/// Panics if `bytes` is empty: a signed value needs at least one byte to carry its sign.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, from_be_bytes_signed};
/// let mut digits = [Digit::ZERO; 1];
/// from_be_bytes_signed(&[1, 2], &mut digits);
/// assert_eq!(digits, [Digit::from(0x0102u16)]);
/// // [0xff] is -1; it sign-extends to fill the digit with ones.
/// from_be_bytes_signed(&[0xff], &mut digits);
/// assert_eq!(digits, [Digit::MAX]);
/// ```
pub fn from_be_bytes_signed(bytes: &[u8], digits: &mut [Digit]) {
    assert!(!bytes.is_empty());
    assert_eq!(digits.len(), bytes.len().div_ceil(Digit::BYTES));
    let mut digit_iter = digits.iter_mut();
    let (rem, chunks) = bytes.as_rchunks::<{ Digit::BYTES }>();
    for &chunk in chunks.iter().rev() {
        *digit_iter.next().unwrap() = Digit::from_be_bytes(chunk);
    }
    if let Some(&first) = rem.first() {
        let fill = sign_extension_byte(first.cast_signed()).cast_unsigned();
        let mut arr = [fill; Digit::BYTES];
        arr[Digit::BYTES - rem.len()..].copy_from_slice(rem);
        *digit_iter.next().unwrap() = Digit::from_be_bytes(arr);
    }
    assert!(digit_iter.next().is_none());
}

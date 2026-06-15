//! Bit operations.

use crate::{Digit, is_negative, min_len_unsigned};

/// The width of a [`Digit`] in bits, as a `usize`.
pub const DIGIT_BITS_USIZE: usize = Digit::BITS as usize;

/// A bit position within a digit slice.
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, DIGIT_BITS_USIZE};
/// let index = BitIndex::from(DIGIT_BITS_USIZE + 3);
/// assert_eq!(index.digit_index(), 1);
/// assert_eq!(index.bit_index(), 3);
/// assert_eq!(usize::try_from(index).unwrap(), DIGIT_BITS_USIZE + 3);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BitIndex {
    digit_index: usize,
    bit_index: u32,
}

impl BitIndex {
    /// Creates a `BitIndex` from a digit index and a bit index within that digit.
    ///
    /// # Panics
    ///
    /// Panics if `bit_index >= Digit::BITS`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ibig_core::BitIndex;
    /// let index = BitIndex::new(1, 3);
    /// assert_eq!(index.digit_index(), 1);
    /// assert_eq!(index.bit_index(), 3);
    /// ```
    pub fn new(digit_index: usize, bit_index: u32) -> BitIndex {
        assert!(bit_index < Digit::BITS, "bit index out of range");
        BitIndex {
            digit_index,
            bit_index,
        }
    }

    /// The index of the [`Digit`] that holds the bit.
    pub fn digit_index(self) -> usize {
        self.digit_index
    }

    /// The index of the bit within its [`Digit`], in the range `0..Digit::BITS`.
    pub fn bit_index(self) -> u32 {
        self.bit_index
    }
}

impl From<usize> for BitIndex {
    fn from(index: usize) -> BitIndex {
        BitIndex {
            digit_index: index / DIGIT_BITS_USIZE,
            bit_index: (index % DIGIT_BITS_USIZE).try_into().unwrap(),
        }
    }
}

/// The error returned when converting a [`BitIndex`] to a `usize` that would
/// overflow `usize`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitIndexOutOfRange;

impl core::fmt::Display for BitIndexOutOfRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("bit index does not fit in a usize")
    }
}

impl core::error::Error for BitIndexOutOfRange {}

impl TryFrom<BitIndex> for usize {
    type Error = BitIndexOutOfRange;

    fn try_from(index: BitIndex) -> Result<usize, BitIndexOutOfRange> {
        const {
            assert!(Digit::BITS.is_power_of_two());
        }
        let high = index
            .digit_index
            .checked_mul(DIGIT_BITS_USIZE)
            .ok_or(BitIndexOutOfRange)?;
        Ok(high | usize::try_from(index.bit_index).unwrap())
    }
}

/// Returns the bit at `index`. An `index` whose digit is at or above the value's length reads as
/// `false`, since the value is zero-extended.
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, Digit, bit_unsigned};
/// assert!(bit_unsigned(&[Digit::from(0b101u8)], BitIndex::from(0)));
/// assert!(!bit_unsigned(&[Digit::from(0b101u8)], BitIndex::from(1)));
/// assert!(bit_unsigned(&[Digit::from(0b101u8)], BitIndex::from(2)));
/// // Above the value's bits.
/// assert!(!bit_unsigned(&[Digit::from(5u8)], BitIndex::from(100)));
/// // The low bit of the second digit.
/// assert!(bit_unsigned(&[Digit::ZERO, Digit::from(1u8)], BitIndex::new(1, 0)));
/// ```
pub fn bit_unsigned(digits: &[Digit], index: BitIndex) -> bool {
    index.digit_index() < digits.len() && digit_bit(digits[index.digit_index()], index.bit_index())
}

/// Returns the bit at `index` of the non-empty `digits`, interpreted as a signed value.
/// An `index` whose digit is at or above `digits.len()` reads as the sign bit,
/// since the value is sign-extended.
///
/// # Panics
///
/// Panics if `digits` is empty.
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, Digit, bit_signed};
/// // -1 is all ones, including the sign-extended positions.
/// assert!(bit_signed(&[Digit::MAX], BitIndex::from(0)));
/// assert!(bit_signed(&[Digit::MAX], BitIndex::from(100)));
/// // 0b101 is non-negative, so high positions read as zero.
/// assert!(bit_signed(&[Digit::from(0b101u8)], BitIndex::from(0)));
/// assert!(!bit_signed(&[Digit::from(0b101u8)], BitIndex::from(100)));
/// ```
pub fn bit_signed(digits: &[Digit], index: BitIndex) -> bool {
    if index.digit_index() < digits.len() {
        digit_bit(digits[index.digit_index()], index.bit_index())
    } else {
        is_negative(digits)
    }
}

/// Sets the bit at `index` to `value`.
///
/// # Panics
///
/// Panics if `index.digit_index() >= digits.len()`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, Digit, set_bit};
/// let mut digits = [Digit::from(0b100u8)];
/// set_bit(&mut digits, BitIndex::from(0), true);
/// assert_eq!(digits, [Digit::from(0b101u8)]);
/// set_bit(&mut digits, BitIndex::from(2), false);
/// assert_eq!(digits, [Digit::from(0b001u8)]);
/// ```
pub fn set_bit(digits: &mut [Digit], index: BitIndex, value: bool) {
    let mask = Digit::from_u8(1) << index.bit_index();
    if value {
        digits[index.digit_index()] |= mask;
    } else {
        digits[index.digit_index()] &= !mask;
    }
}

/// Returns the index of the highest set bit of `digits`, or `None` if the value is zero.
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, Digit, highest_one};
/// assert_eq!(highest_one(&[]), None);
/// assert_eq!(highest_one(&[Digit::ZERO, Digit::ZERO]), None);
/// assert_eq!(highest_one(&[Digit::from(0b101u8)]), Some(BitIndex::new(0, 2)));
/// assert_eq!(highest_one(&[Digit::from(5u8), Digit::ZERO]), Some(BitIndex::new(0, 2)));
/// assert_eq!(highest_one(&[Digit::ZERO, Digit::from(1u8)]), Some(BitIndex::new(1, 0)));
/// ```
pub fn highest_one(digits: &[Digit]) -> Option<BitIndex> {
    let len = min_len_unsigned(digits);
    if len == 0 {
        None
    } else {
        let bit_index = Digit::BITS - 1 - digits[len - 1].leading_zeros();
        Some(BitIndex::new(len - 1, bit_index))
    }
}

/// Returns the index of the lowest set bit of `digits`, or `None` if the value is zero.
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, Digit, lowest_one};
/// assert_eq!(lowest_one(&[Digit::ZERO]), None);
/// assert_eq!(lowest_one(&[Digit::from(0b1100u8)]), Some(BitIndex::new(0, 2)));
/// assert_eq!(lowest_one(&[Digit::ZERO, Digit::from(1u8)]), Some(BitIndex::new(1, 0)));
/// ```
pub fn lowest_one(digits: &[Digit]) -> Option<BitIndex> {
    digits
        .iter()
        .position(|&digit| digit != Digit::ZERO)
        .map(|digit_index| BitIndex::new(digit_index, digits[digit_index].trailing_zeros()))
}

/// Returns the index of the lowest unset bit of `digits`, or `None` if every bit is set (the
/// slice is all ones, or empty).
///
/// # Examples
///
/// ```
/// # use ibig_core::{BitIndex, Digit, lowest_zero};
/// assert_eq!(lowest_zero(&[Digit::MAX]), None);
/// assert_eq!(lowest_zero(&[Digit::from(0b1011u8)]), Some(BitIndex::new(0, 2)));
/// assert_eq!(lowest_zero(&[Digit::MAX, Digit::from(0b10u8)]), Some(BitIndex::new(1, 0)));
/// ```
pub fn lowest_zero(digits: &[Digit]) -> Option<BitIndex> {
    digits
        .iter()
        .position(|&digit| digit != Digit::MAX)
        .map(|digit_index| BitIndex::new(digit_index, digits[digit_index].trailing_ones()))
}

/// Returns the number of one bits (the population count) in `digits`.
///
/// # Overflow
///
/// For extremely long slices, the result may overflow `usize`.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, count_ones};
/// assert_eq!(count_ones(&[]), 0);
/// assert_eq!(count_ones(&[Digit::from(0b101u8)]), 2);
/// assert_eq!(count_ones(&[Digit::MAX]), Digit::BITS as usize);
/// assert_eq!(count_ones(&[Digit::from(0b11u8), Digit::from(0b1u8)]), 3);
/// ```
pub fn count_ones(digits: &[Digit]) -> usize {
    digits
        .iter()
        .map(|&digit| usize::try_from(digit.count_ones()).unwrap())
        .sum()
}

/// Returns `true` if `digits` is a power of two, i.e. exactly one bit is set. Returns `false`
/// for zero.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, is_power_of_two};
/// assert!(is_power_of_two(&[Digit::from(8u8)]));
/// assert!(!is_power_of_two(&[Digit::from(6u8)]));
/// assert!(!is_power_of_two(&[Digit::ZERO]));
/// assert!(is_power_of_two(&[Digit::ZERO, Digit::from(4u8)]));
/// assert!(!is_power_of_two(&[Digit::from(1u8), Digit::from(1u8)]));
/// ```
pub fn is_power_of_two(digits: &[Digit]) -> bool {
    let len = min_len_unsigned(digits);
    let Some((top, low)) = digits[..len].split_last() else {
        return false;
    };
    top.is_power_of_two() && min_len_unsigned(low) == 0
}

/// Replaces the value with the smallest power of two greater than or equal to it.
///
/// Returns `true` on overflow — when that power of two does not fit in
/// `digits.len() * Digit::BITS` bits — in which case `digits` is set to zero.
///
/// # Examples
///
/// ```
/// # use ibig_core::{Digit, next_power_of_two};
/// let mut digits = [Digit::from(5u8), Digit::from(5u8)];
/// assert!(!next_power_of_two(&mut digits));
/// assert_eq!(digits, [Digit::ZERO, Digit::from(8u8)]);
/// let mut digits = [];
/// assert!(next_power_of_two(&mut digits));
/// let mut digits = [Digit::ZERO];
/// assert!(!next_power_of_two(&mut digits));
/// assert_eq!(digits, [Digit::from(1u8)]);
/// let mut digits = [Digit::MAX];
/// assert!(next_power_of_two(&mut digits));
/// assert_eq!(digits, [Digit::ZERO]);
/// ```
pub fn next_power_of_two(digits: &mut [Digit]) -> bool {
    let len = min_len_unsigned(digits);
    let top_overflow = 'top_overflow: {
        let Some((top, low)) = digits[..len].split_last_mut() else {
            // len == 0, so overflow
            break 'top_overflow true;
        };
        let min_len_low = min_len_unsigned(low);
        if min_len_low != 0 {
            // Clear `low` and add 1 to top.
            low[..min_len_low].fill(Digit::ZERO);
            let (new_top, top_overflow) = top.overflowing_add(Digit::from_u8(1));
            *top = new_top;
            if top_overflow {
                break 'top_overflow true;
            }
        }
        // Increase `top` to the next power of two.
        match top.checked_next_power_of_two() {
            Some(p) => {
                *top = p;
                false
            }
            None => {
                *top = Digit::ZERO;
                true
            }
        }
    };
    if top_overflow {
        if len >= digits.len() {
            return true;
        }
        digits[len] = Digit::from_u8(1);
    }
    false
}

/// Returns the bit at `bit_index` (which must be less than `Digit::BITS`) of a single digit.
fn digit_bit(digit: Digit, bit_index: u32) -> bool {
    (digit >> bit_index) & Digit::from_u8(1) != Digit::ZERO
}

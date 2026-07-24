//! Integration tests for bit operations.

use ibig_core::{
    BitIndex, BitIndexOutOfRange, Digit, bit_signed, bit_unsigned, count_ones, highest_one,
    is_power_of_two, lowest_one, lowest_zero, next_power_of_two,
};
use proptest::prelude::*;

const fn digit(n: u16) -> Digit {
    Digit::from_u16(n)
}

const BITS: usize = Digit::BITS as usize;

fn idx(position: usize) -> BitIndex {
    BitIndex::from(position)
}

proptest! {
    // Converting any `usize` into  `BitIndex` and back recovers the same index.
    #[test]
    fn test_bit_index_roundtrip(position: usize) {
        let index = BitIndex::from(position);
        prop_assert!(index.bit_index() < Digit::BITS);
        prop_assert_eq!(usize::try_from(index).unwrap(), position);
    }
}

#[test]
fn test_bit_index_overflow() {
    // `digit_index * DIGIT_BITS_USIZE` overflows `usize`, so recombination fails.
    let index = BitIndex::new(usize::MAX, 1);
    assert_eq!(usize::try_from(index), Err(BitIndexOutOfRange));
}

#[test]
#[should_panic]
fn test_bit_index_new_bad_bit() {
    // A bit index at or above the digit width is rejected.
    let _ = BitIndex::new(0, Digit::BITS);
}

#[test]
fn test_bit_unsigned() {
    // 0b101 = 5.
    let d = [digit(0b101)];
    assert!(bit_unsigned(&d, idx(0)));
    assert!(!bit_unsigned(&d, idx(1)));
    assert!(bit_unsigned(&d, idx(2)));
    assert!(!bit_unsigned(&d, idx(3)));
    // The value is zero-extended above its bits.
    assert!(!bit_unsigned(&d, idx(BITS - 1)));
    assert!(!bit_unsigned(&d, idx(BITS)));
    assert!(!bit_unsigned(&d, idx(1000)));
    // An empty slice is the value zero: every bit is clear.
    assert!(!bit_unsigned(&[], idx(0)));
    assert!(!bit_unsigned(&[], idx(1000)));
    // Bits across a digit boundary: low digit all ones, high digit 0b10.
    let d2 = [Digit::MAX, digit(0b10)];
    assert!(bit_unsigned(&d2, idx(0)));
    assert!(bit_unsigned(&d2, idx(BITS - 1)));
    assert!(!bit_unsigned(&d2, idx(BITS))); // bit 0 of 0b10
    assert!(bit_unsigned(&d2, idx(BITS + 1))); // bit 1 of 0b10
    assert!(!bit_unsigned(&d2, idx(BITS + 2)));
}

#[test]
fn test_bit_signed() {
    // -1 is all ones, including every sign-extended position.
    assert!(bit_signed(&[Digit::MAX], idx(0)));
    assert!(bit_signed(&[Digit::MAX], idx(BITS - 1)));
    assert!(bit_signed(&[Digit::MAX], idx(BITS)));
    assert!(bit_signed(&[Digit::MAX], idx(1000)));
    // 0b101 is non-negative: the sign bit and everything above it are zero.
    let p = [digit(0b101)];
    assert!(bit_signed(&p, idx(0)));
    assert!(!bit_signed(&p, idx(1)));
    assert!(bit_signed(&p, idx(2)));
    assert!(!bit_signed(&p, idx(BITS - 1)));
    assert!(!bit_signed(&p, idx(BITS)));
    assert!(!bit_signed(&p, idx(1000)));
    // A negative two-digit value [0, MAX]: low digit zero, high digit all ones, sign negative.
    let neg = [digit(0), Digit::MAX];
    assert!(!bit_signed(&neg, idx(0)));
    assert!(!bit_signed(&neg, idx(BITS - 1)));
    assert!(bit_signed(&neg, idx(BITS)));
    assert!(bit_signed(&neg, idx(2 * BITS - 1)));
    assert!(bit_signed(&neg, idx(2 * BITS))); // sign-extended
    assert!(bit_signed(&neg, idx(1000)));
}

#[test]
#[should_panic]
fn test_bit_signed_empty() {
    let _ = bit_signed(&[], idx(0));
}

#[test]
fn test_highest_one() {
    let cases: &[(&[Digit], Option<BitIndex>)] = &[
        // Zero in any form has no set bit.
        (&[], None),
        (&[digit(0)], None),
        (&[digit(0), digit(0)], None),
        // Small values.
        (&[digit(1)], Some(BitIndex::new(0, 0))),
        (&[digit(0b101)], Some(BitIndex::new(0, 2))),
        (&[Digit::MAX], Some(BitIndex::new(0, Digit::BITS - 1))),
        // Most-significant zero digits don't count.
        (&[digit(5), digit(0)], Some(BitIndex::new(0, 2))),
        // A set bit in a higher digit, above a zero low digit.
        (&[digit(0), digit(1)], Some(BitIndex::new(1, 0))),
        (
            &[digit(0), Digit::MAX],
            Some(BitIndex::new(1, Digit::BITS - 1)),
        ),
        (
            &[Digit::MAX, Digit::MAX],
            Some(BitIndex::new(1, Digit::BITS - 1)),
        ),
    ];
    for &(digits, expected) in cases {
        assert_eq!(highest_one(digits), expected);
    }
}

#[test]
fn test_lowest_one() {
    let cases: &[(&[Digit], Option<BitIndex>)] = &[
        // Within the low digit.
        (&[digit(0b1100)], Some(BitIndex::new(0, 2))),
        (&[digit(1)], Some(BitIndex::new(0, 0))),
        (&[Digit::MAX], Some(BitIndex::new(0, 0))),
        // A zero low digit skips to the next.
        (&[digit(0), digit(0b100)], Some(BitIndex::new(1, 2))),
        (&[digit(0), digit(0), digit(1)], Some(BitIndex::new(2, 0))),
        // All zeros (the value zero): no set bit.
        (&[], None),
        (&[digit(0), digit(0)], None),
    ];
    for &(digits, expected) in cases {
        assert_eq!(lowest_one(digits), expected);
    }
}

#[test]
fn test_lowest_zero() {
    let cases: &[(&[Digit], Option<BitIndex>)] = &[
        // Within the low digit.
        (&[digit(0b1011)], Some(BitIndex::new(0, 2))),
        (&[digit(0)], Some(BitIndex::new(0, 0))),
        // An all-ones low digit skips to the next.
        (&[Digit::MAX, digit(0b1011)], Some(BitIndex::new(1, 2))),
        (
            &[Digit::MAX, Digit::MAX, digit(0)],
            Some(BitIndex::new(2, 0)),
        ),
        // All ones (no zero bit): None.
        (&[], None),
        (&[Digit::MAX, Digit::MAX], None),
    ];
    for &(digits, expected) in cases {
        assert_eq!(lowest_zero(digits), expected);
    }
}

#[test]
fn test_count_ones() {
    let cases: &[(&[Digit], usize)] = &[
        (&[], 0),
        (&[digit(0)], 0),
        (&[digit(0b101)], 2),
        (&[Digit::MAX], BITS),
        // Counts across every digit, including most-significant zeros.
        (&[digit(0b11), digit(0b1)], 3),
        (&[Digit::MAX, digit(0)], BITS),
        (&[Digit::MAX, Digit::MAX], 2 * BITS),
    ];
    for &(digits, expected) in cases {
        assert_eq!(count_ones(digits), expected);
    }
}

#[test]
fn test_is_power_of_two() {
    let cases: &[(&[Digit], bool)] = &[
        // Zero and all-zero slices are not powers of two.
        (&[], false),
        (&[digit(0)], false),
        (&[digit(0), digit(0)], false),
        // Single-digit powers of two and non-powers.
        (&[digit(1)], true),
        (&[digit(8)], true),
        (&[digit(6)], false),
        (&[Digit::MAX], false),
        // The high bit of a digit is a power of two.
        (&[Digit::MAX / digit(2) + digit(1)], true),
        // A single set bit in a higher digit (lower digits zero).
        (&[digit(0), digit(0), digit(4)], true),
        // More than one set bit, across digits or within one.
        (&[digit(1), digit(1)], false),
        (&[digit(0), digit(3)], false),
        (&[digit(1), digit(0), digit(4)], false),
    ];
    for &(digits, expected) in cases {
        assert_eq!(is_power_of_two(digits), expected);
    }
}

#[test]
fn test_next_power_of_two() {
    let high_bit = Digit::from_u8(1) << (Digit::BITS - 1);

    // Empty slice rounds up to one and overflows.
    assert!(next_power_of_two(&mut []));

    // Zero rounds up to one.
    let mut d = [digit(0), digit(0)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(1), digit(0)]);

    // Already a power of two: unchanged.
    let mut d = [digit(1)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(1)]);
    let mut d = [digit(8)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(8)]);

    // Rounds up to the next power of two.
    let mut d = [digit(3)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(4)]);
    let mut d = [digit(5)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(8)]);

    // Multi-digit: [1, 1] rounds up to [0, 2].
    let mut d = [digit(1), digit(1)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), digit(2)]);

    // Multi-digit power of two: unchanged.
    let mut d = [digit(0), digit(0), digit(0), digit(8)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), digit(0), digit(0), digit(8)]);

    let mut d = [digit(0), high_bit];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), high_bit]);

    // Multi-digit non-power of two.
    let mut d = [digit(0), digit(1), digit(0), digit(8)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), digit(0), digit(0), digit(16)]);

    let mut d = [digit(0), digit(1), high_bit, digit(0)];
    assert!(!next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), digit(0), digit(0), digit(1)]);

    // Overflow: all-ones has no fitting next power of two, so the value becomes zero.
    let mut d = [Digit::MAX];
    assert!(next_power_of_two(&mut d));
    assert_eq!(d, [digit(0)]);
    let mut d = [Digit::MAX, Digit::MAX];
    assert!(next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), digit(0)]);

    // Overflow boundary: the top bit is set but the value is not a power of two.
    let mut d = [digit(1), high_bit];
    assert!(next_power_of_two(&mut d));
    assert_eq!(d, [digit(0), digit(0)]);
}

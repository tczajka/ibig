//! Integration tests for minimal canonical length helpers.

use ibig_core::{
    Digit, min_len_bytes_signed, min_len_bytes_unsigned, min_len_signed, min_len_unsigned,
};

fn digit(n: u8) -> Digit {
    Digit::from(n)
}

#[test]
fn test_min_len_unsigned() {
    let cases: &[(&[Digit], usize)] = &[
        // Zero needs no digits: empty and all-zero slices collapse to 0.
        (&[], 0),
        (&[digit(0)], 0),
        (&[digit(0), digit(0), digit(0)], 0),
        // A nonzero value keeps up to its most-significant nonzero digit.
        (&[digit(5)], 1),
        (&[digit(5), digit(0)], 1),
        (&[digit(1), digit(2), digit(0)], 2),
        // A zero low digit below a nonzero digit is kept.
        (&[digit(0), digit(1)], 2),
        // Nothing to strip.
        (&[digit(1), digit(2), digit(3)], 3),
    ];
    for &(digits, expected) in cases {
        assert_eq!(min_len_unsigned(digits), expected);
    }
}

#[test]
fn test_min_len_signed() {
    let cases: &[(&[Digit], usize)] = &[
        // Zero needs one digit.
        (&[digit(0)], 1),
        (&[digit(0), digit(0)], 1),
        // A redundant zero sign-extension above a non-negative digit is stripped.
        (&[digit(5), digit(0)], 1),
        (&[digit(1), digit(2), digit(0), digit(0)], 2),
        // -1 is all-ones; the redundant all-ones sign-extension digits are stripped.
        (&[Digit::MAX], 1),
        (&[Digit::MAX, Digit::MAX, Digit::MAX], 1),
        // A positive value whose top significant digit has its high bit set needs a leading
        // zero digit, which must be kept.
        (&[Digit::MAX, digit(0)], 2),
        // A negative value whose top significant digit has its high bit clear needs a leading
        // all-ones digit, which must be kept.
        (&[digit(0), Digit::MAX], 2),
        (&[digit(5), Digit::MAX], 2),
    ];
    for &(digits, expected) in cases {
        assert_eq!(min_len_signed(digits), expected);
    }
}

#[test]
#[should_panic]
fn test_min_len_signed_empty() {
    let _ = min_len_signed(&[]);
}

#[test]
fn test_min_len_bytes_unsigned() {
    let cases: &[(&[u8], usize)] = &[
        (&[], 0),
        (&[0], 0),
        (&[0, 0, 0], 0),
        (&[5], 1),
        (&[5, 0], 1),
        (&[1, 2, 0], 2),
        (&[0, 1], 2),
    ];
    for &(bytes, expected) in cases {
        assert_eq!(min_len_bytes_unsigned(bytes), expected);
    }
}

#[test]
fn test_min_len_bytes_signed() {
    let cases: &[(&[u8], usize)] = &[
        // Zero and sign-only sequences collapse to a single byte.
        (&[0], 1),
        (&[0, 0, 0], 1),
        (&[0xff, 0xff, 0xff], 1),
        // Redundant sign bytes above a same-signed byte are stripped.
        (&[5, 0], 1),
        (&[0xfe, 0xff], 1),
        // A leading byte needed to carry the sign is kept: 0xc8 alone is negative, so +200
        // needs the leading zero; 0x80 alone is positive-looking below 0x80, so -32768 needs it.
        (&[0xc8, 0x00], 2), // +200
        (&[0x00, 0x80], 2), // -32768
        (&[5, 0, 0, 0], 1),
    ];
    for &(bytes, expected) in cases {
        assert_eq!(min_len_bytes_signed(bytes), expected);
    }
}

#[test]
#[should_panic]
fn test_min_len_bytes_signed_empty() {
    let _ = min_len_bytes_signed(&[]);
}

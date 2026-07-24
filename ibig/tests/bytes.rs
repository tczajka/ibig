//! Integration tests for `UBig` and `IBig` <-> byte-sequence conversions.

use ibig::proptest::{ibig_up_to_bits, ubig_up_to_bits};
use ibig::{IBig, UBig};
use proptest::prelude::*;

#[test]
fn ubig_zero() {
    assert_eq!(UBig::from_le_bytes(&[]), UBig::ZERO);
    assert_eq!(UBig::from_le_bytes(&[0, 0, 0]), UBig::ZERO);
    assert_eq!(UBig::from_be_bytes(&[]), UBig::ZERO);
    assert_eq!(UBig::from_be_bytes(&[0, 0, 0]), UBig::ZERO);
    assert_eq!(UBig::ZERO.to_le_bytes(), []);
    assert_eq!(UBig::ZERO.to_be_bytes(), []);
}

#[test]
fn ubig_le_normalizes_trailing_zeros() {
    assert_eq!(UBig::from_le_bytes(&[5, 0, 0]).to_le_bytes(), [5]);
    assert_eq!(UBig::from_le_bytes(&[1, 2, 0, 0, 0]).to_le_bytes(), [1, 2]);
}

#[test]
fn ubig_be_normalizes_leading_zeros() {
    assert_eq!(UBig::from_be_bytes(&[0, 0, 5]).to_be_bytes(), [5]);
    assert_eq!(UBig::from_be_bytes(&[0, 0, 0, 1, 2]).to_be_bytes(), [1, 2]);
}

proptest! {
    // Round-trip `value -> bytes -> value` for random values up to 1000 bits.
    #[test]
    fn ubig_le_round_trip(x in ubig_up_to_bits(1000)) {
        prop_assert_eq!(UBig::from_le_bytes(&x.to_le_bytes()), x);
    }

    #[test]
    fn ubig_be_round_trip(x in ubig_up_to_bits(1000)) {
        prop_assert_eq!(UBig::from_be_bytes(&x.to_be_bytes()), x);
    }

    #[test]
    fn ibig_le_round_trip(x in ibig_up_to_bits(1000)) {
        prop_assert_eq!(IBig::from_le_bytes(&x.to_le_bytes()), x);
    }

    #[test]
    fn ibig_be_round_trip(x in ibig_up_to_bits(1000)) {
        prop_assert_eq!(IBig::from_be_bytes(&x.to_be_bytes()), x);
    }

    // Reading little-endian bytes equals reading the reversed bytes big-endian.
    #[test]
    fn ubig_from_le_be_agree(bytes in proptest::collection::vec(any::<u8>(), 0..=100)) {
        let mut reversed = bytes.clone();
        reversed.reverse();
        prop_assert_eq!(UBig::from_le_bytes(&bytes), UBig::from_be_bytes(&reversed));
    }

    #[test]
    fn ibig_from_le_be_agree(bytes in proptest::collection::vec(any::<u8>(), 1..=100)) {
        let mut reversed = bytes.clone();
        reversed.reverse();
        prop_assert_eq!(IBig::from_le_bytes(&bytes), IBig::from_be_bytes(&reversed));
    }

    // `to_be_bytes` is `to_le_bytes` reversed.
    #[test]
    fn ubig_to_le_be_agree(x in ubig_up_to_bits(1000)) {
        let mut reversed = x.to_le_bytes();
        reversed.reverse();
        prop_assert_eq!(x.to_be_bytes(), reversed);
    }

    #[test]
    fn ibig_to_le_be_agree(x in ibig_up_to_bits(1000)) {
        let mut reversed = x.to_le_bytes();
        reversed.reverse();
        prop_assert_eq!(x.to_be_bytes(), reversed);
    }
}

#[test]
fn ibig_sign_extension_is_ignored() {
    // +200 with extra zero sign padding.
    assert_eq!(IBig::from_le_bytes(&[200, 0, 0]), IBig::from(200i16));
    // -100 with extra 0xff sign padding.
    assert_eq!(IBig::from_le_bytes(&[0x9c, 0xff, 0xff]), IBig::from(-100i8));
    // The big-endian side ignores leading sign bytes too.
    assert_eq!(IBig::from_be_bytes(&[0xff, 0xff, 0x9c]), IBig::from(-100i8));
}

#[test]
#[should_panic]
fn ibig_from_le_empty_panics() {
    let _ = IBig::from_le_bytes(&[]);
}

#[test]
#[should_panic]
fn ibig_from_be_empty_panics() {
    let _ = IBig::from_be_bytes(&[]);
}

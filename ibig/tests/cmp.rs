//! Integration tests for equality comparison.

use ibig::{IBig, UBig};

#[test]
fn ubig() {
    // Single digit vs single digit.
    assert_eq!(UBig::from(5u8), UBig::from(5u8));
    assert_ne!(UBig::from(5u8), UBig::from(6u8));
    assert_eq!(UBig::ZERO, UBig::ZERO);

    // Multi-digit vs multi-digit (same length).
    let a = (UBig::from(1u8) << 100) + UBig::from(7u8);
    let b = (UBig::from(1u8) << 100) + UBig::from(7u8);
    let c = (UBig::from(1u8) << 100) + UBig::from(8u8);
    assert_eq!(a, b);
    assert_ne!(a, c);

    // A single digit is never equal to a multi-digit value (both dispatch orders).
    assert_ne!(UBig::from(7u8), UBig::from(1u8) << 100);
    assert_ne!(UBig::from(1u8) << 100, UBig::from(7u8));

    // Multi-digit values of different lengths.
    assert_ne!(UBig::from(1u8) << 100, UBig::from(1u8) << 200);
}

#[test]
fn ibig() {
    // Single digit, including sign.
    assert_eq!(IBig::from(5), IBig::from(5));
    assert_ne!(IBig::from(5), IBig::from(-5));
    assert_eq!(IBig::from(-5), IBig::from(-5));
    assert_eq!(IBig::ZERO, IBig::ZERO);

    // Multi-digit vs multi-digit.
    let a = (IBig::from(1) << 100) + IBig::from(3);
    let b = (IBig::from(1) << 100) + IBig::from(3);
    let c = (IBig::from(1) << 100) + IBig::from(4);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(&a, &(IBig::from(-1) << 100));

    // A single digit is never equal to a multi-digit value, for either sign.
    assert_ne!(IBig::from(3), IBig::from(1) << 100);
    assert_ne!(IBig::from(-1) << 100, IBig::from(-1));
}

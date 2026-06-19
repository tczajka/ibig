//! Integration tests for `IBig` sign operations.

use ibig::proptest::ibig_up_to_bits;
use ibig::{IBig, UBig};
use proptest::prelude::*;

#[test]
fn is_negative() {
    assert!(!IBig::ZERO.is_negative());
    assert!(!IBig::from(5i8).is_negative());
    assert!(IBig::from(-5i8).is_negative());
    assert!(!IBig::from(u64::MAX).is_negative());
    assert!(IBig::from(-1i128 << 100).is_negative());
}

#[test]
fn is_positive() {
    assert!(!IBig::ZERO.is_positive());
    assert!(IBig::from(5i8).is_positive());
    assert!(!IBig::from(-5i8).is_positive());
    assert!(IBig::from(u64::MAX).is_positive());
    assert!(!IBig::from(-1i128 << 100).is_positive());
}

#[test]
fn signum() {
    assert_eq!(IBig::ZERO.signum(), IBig::ZERO);
    assert_eq!(IBig::from(5i8).signum(), IBig::from(1i8));
    assert_eq!(IBig::from(-5i8).signum(), IBig::from(-1i8));
    assert_eq!(IBig::from(u64::MAX).signum(), IBig::from(1i8));
    assert_eq!(IBig::from(1i128 << 100).signum(), IBig::from(1i8));
    assert_eq!(IBig::from(-1i128 << 100).signum(), IBig::from(-1i8));
}

#[test]
fn neg() {
    assert_eq!(-IBig::from(5), IBig::from(-5));
    assert_eq!(-IBig::from(-5), IBig::from(5));
    assert_eq!(-IBig::ZERO, IBig::ZERO);
    // Negation of a borrowed value.
    assert_eq!(-&IBig::from(7), IBig::from(-7));

    // Negating the most-negative single-digit value needs an extra digit.
    assert_eq!(-IBig::from(i64::MIN), IBig::from(1) << 63);
    assert_eq!(-IBig::from(i16::MIN), IBig::from(1) << 15);

    // Multi-digit values.
    assert_eq!(-(IBig::from(1) << 200), IBig::from(-1) << 200);
    assert_eq!(-(IBig::from(-1) << 200), IBig::from(1) << 200);
}

#[test]
fn abs() {
    assert_eq!(IBig::ZERO.abs(), IBig::ZERO);
    assert_eq!(IBig::from(5i8).abs(), IBig::from(5i8));
    assert_eq!(IBig::from(-5i8).abs(), IBig::from(5i8));

    // The receiver is borrowed, not consumed.
    let a = IBig::from(-7i8);
    assert_eq!(a.abs(), IBig::from(7i8));
    assert_eq!(a, IBig::from(-7i8));

    // The most-negative single-digit value gains a digit: |i64::MIN| == 2^63.
    assert_eq!(IBig::from(i64::MIN).abs(), IBig::from(1) << 63);
    assert_eq!(IBig::from(i16::MIN).abs(), IBig::from(1) << 15);

    // Multi-digit values.
    assert_eq!((IBig::from(-1) << 200).abs(), IBig::from(1) << 200);
    assert_eq!((IBig::from(1) << 200).abs(), IBig::from(1) << 200);
}

#[test]
fn abs_unsigned() {
    assert_eq!(IBig::ZERO.abs_unsigned(), UBig::ZERO);
    assert_eq!(IBig::from(5i8).abs_unsigned(), UBig::from(5u8));
    assert_eq!(IBig::from(-5i8).abs_unsigned(), UBig::from(5u8));

    // |i64::MIN| == 2^63.
    assert_eq!(IBig::from(i64::MIN).abs_unsigned(), UBig::from(1u8) << 63);
    assert_eq!(IBig::from(i16::MIN).abs_unsigned(), UBig::from(1u8) << 15);

    // Multi-digit values.
    assert_eq!(
        (IBig::from(-1) << 200).abs_unsigned(),
        UBig::from(1u8) << 200
    );
    assert_eq!(
        (IBig::from(1) << 200).abs_unsigned(),
        UBig::from(1u8) << 200
    );
}

proptest! {
    // Double negation is the identity, and negation equals subtracting from zero.
    #[test]
    fn neg_props(a in ibig_up_to_bits(300)) {
        prop_assert_eq!(&-(-&a), &a);
        prop_assert_eq!(-&a, IBig::ZERO - &a);
    }

    // |a| is non-negative, agrees with |-a|, and matches `abs_unsigned`.
    #[test]
    fn abs_props(a in ibig_up_to_bits(300)) {
        let abs = a.abs();
        prop_assert!(!abs.is_negative());
        prop_assert_eq!(&abs, &(-&a).abs());
        prop_assert_eq!(&abs, &IBig::from(a.abs_unsigned()));
    }
}

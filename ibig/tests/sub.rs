//! Integration tests for subtraction.

use ibig::proptest::{ibig_up_to_bits, ubig_up_to_bits};
use ibig::{IBig, UBig};
use proptest::prelude::*;

proptest! {
    // `UBig` subtraction matches `u128` subtraction, across every operand form.
    #[test]
    fn ubig_vs_u128(a: u128, b: u128) {
        let (a, b) = (a.max(b), a.min(b));
        let x = UBig::from(a);
        let y = UBig::from(b);
        let diff = UBig::from(a - b);
        prop_assert_eq!(&x - &y, diff);
    }

    // Subtraction undoes addition, zero is the right identity, and a value minus itself is zero.
    #[test]
    fn ubig_algebra(
        a in ubig_up_to_bits(300),
        b in ubig_up_to_bits(300),
    ) {
        prop_assert_eq!(&((&a + &b) - &b), &a);
        prop_assert_eq!(&(&a - UBig::ZERO), &a);
        prop_assert_eq!(&a - &a, UBig::ZERO);
    }

    // `UBig::checked_sub` matches `u128::checked_sub`.
    #[test]
    fn ubig_checked_sub_vs_u128(a: u128, b: u128) {
        let x = UBig::from(a);
        let y = UBig::from(b);
        prop_assert_eq!(x.checked_sub(&y), a.checked_sub(b).map(UBig::from));
    }

    // `checked_sub` agrees with `-` when the result exists.
    #[test]
    fn ubig_checked_sub_algebra(
        a in ubig_up_to_bits(300),
        b in ubig_up_to_bits(300),
    ) {
        prop_assert_eq!((&a + &b).checked_sub(&b), Some(a));
    }

    // `UBig::saturating_sub` matches `u128::saturating_sub`.
    #[test]
    fn ubig_saturating_sub_vs_u128(a: u128, b: u128) {
        let x = UBig::from(a);
        let y = UBig::from(b);
        prop_assert_eq!(x.saturating_sub(&y), UBig::from(a.saturating_sub(b)));
    }

    // `IBig` subtraction matches `i128` subtraction, across every operand form.
    #[test]
    fn ibig_vs_i128(a: i128, b: i128) {
        let x = IBig::from(a);
        let y = IBig::from(b);
        let (low, overflow) = a.overflowing_sub(b);
        let mut diff = IBig::from(low);
        if overflow {
            // The wrapped difference is 2^128 away from the true one, opposite to its own sign.
            diff += IBig::from(-low.signum()) << 128;
        }
        prop_assert_eq!(&x - &y, diff);
    }

    #[test]
    fn ibig_algebra(
        a in ibig_up_to_bits(300),
        b in ibig_up_to_bits(300),
    ) {
        prop_assert_eq!(&((&a + &b) - &b), &a);
        prop_assert_eq!(&(&a - IBig::ZERO), &a);
        prop_assert_eq!(&a - &a, IBig::ZERO);
        prop_assert_eq!(&a - &b, &a + (-&b));
    }

    // `UBig::checked_sub_signed` equals `a - b` when non-negative, else `None`.
    #[test]
    fn ubig_checked_sub_signed_vs_ibig(a in ubig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        let diff = IBig::from(&a) - &b;
        prop_assert_eq!(a.checked_sub_signed(&b), UBig::try_from(&diff).ok());
    }

    // `UBig::saturating_sub_signed` equals `a - b` clamped at zero.
    #[test]
    fn ubig_saturating_sub_signed_vs_ibig(a in ubig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        let diff = IBig::from(&a) - &b;
        prop_assert_eq!(a.saturating_sub_signed(&b), UBig::try_from(&diff).unwrap_or(UBig::ZERO));
    }

    // `UBig::strict_sub_signed` agrees with `checked_sub_signed` whenever the result exists.
    #[test]
    fn ubig_strict_sub_signed_matches_checked(a in ubig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        if let Some(expected) = a.checked_sub_signed(&b) {
            prop_assert_eq!(a.strict_sub_signed(&b), expected);
        }
    }
}

#[test]
fn ubig_sub_basic() {
    assert_eq!(UBig::from(5u8) - UBig::from(3u8), UBig::from(2u8));
    assert_eq!(UBig::ZERO - UBig::ZERO, UBig::ZERO);
    // A borrow shrinks the value by a digit.
    assert_eq!(
        UBig::from(u128::from(u64::MAX) + 1) - UBig::from(1u8),
        UBig::from(u64::MAX)
    );
    // A borrow propagates through many all-zeros digits.
    let big = UBig::from(1u8) << 256;
    let almost = UBig::from_le_bytes(&[0xff; 32]);
    assert_eq!(big - UBig::from(1u8), almost);

    // `&lhs - owned_rhs`
    let a = (UBig::from(1u8) << 200) + UBig::from(123u8);
    let same_len = (UBig::from(1u8) << 200) - UBig::from(1u8);
    assert_eq!(&a - same_len.clone(), &a - &same_len);
    let shorter = UBig::from(5u8) << 100;
    assert_eq!(&a - shorter.clone(), &a - &shorter);
}

#[test]
fn ibig_sub_basic() {
    assert_eq!(IBig::from(5) - IBig::from(3), IBig::from(2));
    assert_eq!(IBig::from(3) - IBig::from(5), IBig::from(-2));
    assert_eq!(IBig::from(-5) - IBig::from(-3), IBig::from(-2));
    assert_eq!(IBig::from(5) - IBig::from(-3), IBig::from(8));
    assert_eq!(IBig::ZERO - IBig::ZERO, IBig::ZERO);

    // The result grows by a sign digit: 2^255 - (-2^255) == 2^256.
    assert_eq!(
        (IBig::from(1) << 255) - (IBig::from(-1) << 255),
        IBig::from(1) << 256
    );

    // Crossing zero with multi-digit operands: 2^200 - 2^201 == -2^200.
    assert_eq!(
        (IBig::from(1) << 200) - (IBig::from(1) << 201),
        IBig::from(-1) << 200
    );

    // A single digit minus a long value (`lhs` shorter than `rhs`); verify via addition.
    let r = IBig::from(7) - (IBig::from(1) << 192);
    assert_eq!(r + (IBig::from(1) << 192), IBig::from(7));

    // `owned - &ref` with a multi-digit *negative* `lhs` shorter than `rhs`
    // (exercises `SubIBigIBig::apply_val_ref`, which other forms bypass).
    let a = -((IBig::from(1) << 100) + IBig::from(1));
    let b = IBig::from(1) << 300;
    assert_eq!(
        a - &b,
        -((IBig::from(1) << 300) + (IBig::from(1) << 100) + IBig::from(1))
    );
}

#[test]
fn ubig_checked_sub_basic() {
    assert_eq!(
        UBig::from(5u8).checked_sub(&UBig::from(3u8)),
        Some(UBig::from(2u8))
    );
    assert_eq!(UBig::from(2u8).checked_sub(&UBig::from(3u8)), None);
    // A single digit is smaller than any multi-digit value.
    assert_eq!(UBig::from(3u8).checked_sub(&(UBig::from(1u8) << 100)), None);
    // A multi-digit value minus a single digit never underflows.
    let big = UBig::from(1u8) << 100;
    assert_eq!(
        (&big + UBig::from(1u8)).checked_sub(&UBig::from(1u8)),
        Some(big.clone())
    );
    // Multi-digit underflow: a shorter `lhs`, and a smaller `lhs` of the same length.
    assert_eq!(big.checked_sub(&(UBig::from(1u8) << 200)), None);
    assert_eq!(big.checked_sub(&(&big + UBig::from(1u8))), None);
}

#[test]
fn ubig_saturating_sub_basic() {
    assert_eq!(
        UBig::from(5u8).saturating_sub(&UBig::from(3u8)),
        UBig::from(2u8)
    );
    // Underflow saturates at zero.
    assert_eq!(UBig::from(3u8).saturating_sub(&UBig::from(5u8)), UBig::ZERO);
    assert_eq!(
        UBig::from(3u8).saturating_sub(&(UBig::from(1u8) << 100)),
        UBig::ZERO
    );
    // A large non-underflowing difference is preserved.
    let big = UBig::from(1u8) << 100;
    assert_eq!(
        (&big + UBig::from(7u8)).saturating_sub(&UBig::from(7u8)),
        big
    );
}

#[test]
fn ubig_checked_sub_signed_basic() {
    // Single-digit cases.
    assert_eq!(
        UBig::from(5u8).checked_sub_signed(&IBig::from(3)),
        Some(UBig::from(2u8))
    );
    // Subtracting a negative adds.
    assert_eq!(
        UBig::from(5u8).checked_sub_signed(&IBig::from(-3)),
        Some(UBig::from(8u8))
    );
    assert_eq!(
        UBig::from(5u8).checked_sub_signed(&IBig::from(5)),
        Some(UBig::ZERO)
    );
    // A negative result.
    assert_eq!(UBig::from(5u8).checked_sub_signed(&IBig::from(8)), None);
    // `self` shorter than a positive `rhs`.
    assert_eq!(
        UBig::from(3u8).checked_sub_signed(&(IBig::from(1) << 200)),
        None
    );

    // A borrow shrinks the value by a digit: 2^256 - 1.
    let big = UBig::from(1u8) << 256;
    assert_eq!(
        big.checked_sub_signed(&IBig::from(1)),
        Some(&big - UBig::from(1u8))
    );
    // A large negative `rhs` grows the value: 2^256 - -(2^100) == 2^256 + 2^100.
    assert_eq!(
        big.checked_sub_signed(&(IBig::from(-1) << 100)),
        Some(&big + (UBig::from(1u8) << 100))
    );
}

#[test]
fn ubig_saturating_sub_signed_basic() {
    assert_eq!(
        UBig::from(5u8).saturating_sub_signed(&IBig::from(3)),
        UBig::from(2u8)
    );
    // Subtracting a negative adds.
    assert_eq!(
        UBig::from(5u8).saturating_sub_signed(&IBig::from(-3)),
        UBig::from(8u8)
    );
    // The result saturates at zero rather than going negative.
    assert_eq!(
        UBig::from(5u8).saturating_sub_signed(&IBig::from(8)),
        UBig::ZERO
    );
    assert_eq!(
        UBig::from(3u8).saturating_sub_signed(&(IBig::from(1) << 200)),
        UBig::ZERO
    );
}

#[test]
fn ubig_strict_sub_signed_basic() {
    assert_eq!(
        UBig::from(5u8).strict_sub_signed(&IBig::from(3)),
        UBig::from(2u8)
    );
    assert_eq!(
        UBig::from(5u8).strict_sub_signed(&IBig::from(-3)),
        UBig::from(8u8)
    );
}

#[test]
#[should_panic(expected = "negative UBig")]
fn ubig_strict_sub_signed_negative() {
    UBig::from(5u8).strict_sub_signed(&IBig::from(8));
}

#[test]
#[should_panic]
fn ubig_sub_underflow_small_small() {
    let _ = UBig::from(2u8) - UBig::from(3u8);
}

#[test]
#[should_panic]
fn ubig_sub_underflow_small_large() {
    let _ = UBig::from(3u8) - (UBig::from(1u8) << 100);
}

#[test]
#[should_panic]
fn ubig_sub_underflow_large_large() {
    let _ = (UBig::from(1u8) << 100) - (UBig::from(1u8) << 200);
}

#[test]
#[should_panic]
fn ubig_sub_underflow_large_large_same_len() {
    let _ = (UBig::from(1u8) << 100) - ((UBig::from(1u8) << 100) + UBig::from(1u8));
}

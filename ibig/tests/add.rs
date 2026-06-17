//! Integration tests for the `UBig` and `IBig` addition operators.

use ibig::proptest::{ibig_up_to_bits, ubig_up_to_bits};
use ibig::{IBig, UBig};
use proptest::prelude::*;

proptest! {
    // `UBig` addition matches `u128` addition, across every operand form.
    #[test]
    fn ubig_vs_u128(a: u128, b: u128) {
        let x = UBig::from(a);
        let y = UBig::from(b);
        let (low, carry) = a.overflowing_add(b);
        let mut sum = UBig::from(low);
        if carry {
            sum |= UBig::from(1u8) << 128;
        }
        prop_assert_eq!(x + y, sum);
    }

    // Addition is commutative and associative, and zero is the identity.
    #[test]
    fn ubig_algebra(
        a in ubig_up_to_bits(300),
        b in ubig_up_to_bits(300),
        c in ubig_up_to_bits(300),
    ) {
        prop_assert_eq!(&a + &b, &b + &a);
        prop_assert_eq!((&a + &b) + &c, &a + (&b + &c));
        prop_assert_eq!(&(&a + UBig::ZERO), &a);
    }

    // `IBig` addition matches `i128` addition, across every operand form.
    #[test]
    fn ibig_vs_i128(a: i128, b: i128) {
        let x = IBig::from(a);
        let y = IBig::from(b);
        let (low, overflow) = a.overflowing_add(b);
        let mut sum = IBig::from(low);
        if overflow {
            // The wrapped sum is off by 2^128 in the direction of the operands' shared sign.
            sum += IBig::from(a.signum()) << 128;
        }
        prop_assert_eq!(x + y, sum);
    }

    // `IBig` addition is commutative and associative, and zero is the identity.
    #[test]
    fn ibig_algebra(
        a in ibig_up_to_bits(300),
        b in ibig_up_to_bits(300),
        c in ibig_up_to_bits(300),
    ) {
        prop_assert_eq!(&a + &b, &b + &a);
        prop_assert_eq!((&a + &b) + &c, &a + (&b + &c));
        prop_assert_eq!(&(&a + IBig::ZERO), &a);
    }

    // `UBig::checked_add_signed` equals `a + b` when non-negative, else `None`.
    #[test]
    fn ubig_checked_add_signed_vs_ibig(a in ubig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        let sum = IBig::from(&a) + &b;
        prop_assert_eq!(a.checked_add_signed(&b), UBig::try_from(&sum).ok());
    }

    // `UBig::saturating_add_signed` equals `a + b` clamped at zero.
    #[test]
    fn ubig_saturating_add_signed_vs_ibig(a in ubig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        let sum = IBig::from(&a) + &b;
        prop_assert_eq!(a.saturating_add_signed(&b), UBig::try_from(&sum).unwrap_or(UBig::ZERO));
    }

    // `UBig::strict_add_signed` agrees with `checked_add_signed` whenever the result exists.
    #[test]
    fn ubig_strict_add_signed_matches_checked(a in ubig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        if let Some(expected) = a.checked_add_signed(&b) {
            prop_assert_eq!(a.strict_add_signed(&b), expected);
        }
    }

    // `IBig::add_unsigned(a, b)` (a signed, b unsigned) equals `IBig` addition `a + b`.
    #[test]
    fn ibig_add_unsigned_vs_ibig(a in ibig_up_to_bits(300), b in ubig_up_to_bits(300)) {
        prop_assert_eq!(a.add_unsigned(&b), &a + IBig::from(&b));
    }
}

#[test]
fn add_basic() {
    assert_eq!(UBig::from(2u8) + UBig::from(3u8), UBig::from(5u8));
    assert_eq!(UBig::ZERO + UBig::ZERO, UBig::ZERO);
    // A carry grows the value by a digit.
    assert_eq!(
        UBig::from(u64::MAX) + UBig::from(1u8),
        UBig::from(u128::from(u64::MAX) + 1)
    );
    // A carry propagates through many all-ones digits.
    let almost = UBig::from_le_bytes(&[0xff; 32]);
    let one_more = UBig::from(1u8) << 256;
    assert_eq!(almost + UBig::from(1u8), one_more);
}

#[test]
fn ibig_add_basic() {
    assert_eq!(IBig::from(2) + IBig::from(3), IBig::from(5));
    assert_eq!(IBig::from(-2) + IBig::from(-3), IBig::from(-5));
    assert_eq!(IBig::from(5) + IBig::from(-3), IBig::from(2));
    assert_eq!(IBig::ZERO + IBig::ZERO, IBig::ZERO);

    // A positive sum that grows by a sign digit.
    assert_eq!(
        (IBig::from(1) << 255) + (IBig::from(1) << 255),
        IBig::from(1) << 256
    );

    // Opposite large values cancel to zero.
    assert_eq!((IBig::from(1) << 256) + (IBig::from(-1) << 256), IBig::ZERO);

    // A negative single digit sign-extends across a long positive operand:
    // 2^192 + -1 == 2^192 - 1.
    assert_eq!(
        (IBig::from(1) << 192) + IBig::from(-1),
        IBig::from(UBig::from_le_bytes(&[0xff; 24]))
    );

    // Multi-digit *negative* `lhs` shorter than `rhs`, via `owned + &ref`
    // (`apply_val_ref`) and `owned + owned` (`apply_val_val`) — paths other forms bypass.
    let small = -((IBig::from(1) << 100) + IBig::from(1));
    let big = IBig::from(1) << 300;
    let expected = (IBig::from(1) << 300) - (IBig::from(1) << 100) - IBig::from(1);
    assert_eq!(small.clone() + &big, expected.clone());
    assert_eq!(small + big, expected);
}

#[test]
fn ibig_add_digit_boundary() {
    // Adding two values at a signed-integer boundary. On the matching word size each pair is a
    // single-digit-plus-single-digit addition that overflows into a second digit; on the others
    // it still has to come out right. `i128::from` keeps the expected sum exact.
    fn check(a: i128, b: i128) {
        assert_eq!(IBig::from(a) + IBig::from(b), IBig::from(a + b));
    }

    // The boundaries of each digit width.
    check(i128::from(i16::MAX), i128::from(i16::MAX));
    check(i128::from(i16::MIN), i128::from(i16::MIN));
    check(i128::from(i32::MAX), i128::from(i32::MAX));
    check(i128::from(i32::MIN), i128::from(i32::MIN));
    check(i128::from(i64::MAX), i128::from(i64::MAX));
    check(i128::from(i64::MIN), i128::from(i64::MIN));

    // The smallest additions that overflow a single digit at each width.
    check(i128::from(i16::MAX), 1);
    check(i128::from(i16::MIN), -1);
    check(i128::from(i64::MAX), 1);
    check(i128::from(i64::MIN), -1);
}

#[test]
fn ubig_checked_add_signed_basic() {
    // Single-digit cases.
    assert_eq!(
        UBig::from(5u8).checked_add_signed(&IBig::from(3)),
        Some(UBig::from(8u8))
    );
    assert_eq!(
        UBig::from(5u8).checked_add_signed(&IBig::from(-3)),
        Some(UBig::from(2u8))
    );
    assert_eq!(
        UBig::from(5u8).checked_add_signed(&IBig::from(-5)),
        Some(UBig::ZERO)
    );
    // A negative result.
    assert_eq!(UBig::from(5u8).checked_add_signed(&IBig::from(-8)), None);
    // `self` shorter than a negative `rhs`.
    assert_eq!(
        UBig::from(3u8).checked_add_signed(&(IBig::from(-1) << 200)),
        None
    );

    // A carry grows the value by a digit: (2^256 - 1) + 1 == 2^256.
    let big = UBig::from(1u8) << 256;
    assert_eq!(
        (&big - UBig::from(1u8)).checked_add_signed(&IBig::from(1)),
        Some(big.clone())
    );
    // A large negative `rhs`: 2^256 + -(2^100) == 2^256 - 2^100.
    assert_eq!(
        big.checked_add_signed(&(IBig::from(-1) << 100)),
        Some(&big - (UBig::from(1u8) << 100))
    );
}

#[test]
fn ubig_saturating_add_signed_basic() {
    assert_eq!(
        UBig::from(5u8).saturating_add_signed(&IBig::from(3)),
        UBig::from(8u8)
    );
    assert_eq!(
        UBig::from(5u8).saturating_add_signed(&IBig::from(-3)),
        UBig::from(2u8)
    );
    // The result saturates at zero rather than going negative.
    assert_eq!(
        UBig::from(5u8).saturating_add_signed(&IBig::from(-5)),
        UBig::ZERO
    );
    assert_eq!(
        UBig::from(5u8).saturating_add_signed(&IBig::from(-8)),
        UBig::ZERO
    );
    assert_eq!(
        UBig::from(3u8).saturating_add_signed(&(IBig::from(-1) << 200)),
        UBig::ZERO
    );
}

#[test]
fn ubig_strict_add_signed_basic() {
    assert_eq!(
        UBig::from(5u8).strict_add_signed(&IBig::from(3)),
        UBig::from(8u8)
    );
    assert_eq!(
        UBig::from(5u8).strict_add_signed(&IBig::from(-3)),
        UBig::from(2u8)
    );
    assert_eq!(
        UBig::from(5u8).strict_add_signed(&IBig::from(-5)),
        UBig::ZERO
    );
}

#[test]
#[should_panic(expected = "negative UBig")]
fn ubig_strict_add_signed_negative() {
    UBig::from(5u8).strict_add_signed(&IBig::from(-8));
}

#[test]
fn ibig_add_unsigned_basic() {
    // Single-digit cases.
    assert_eq!(IBig::from(5).add_unsigned(&UBig::from(3u8)), IBig::from(8));
    assert_eq!(
        IBig::from(-5).add_unsigned(&UBig::from(3u8)),
        IBig::from(-2)
    );
    assert_eq!(IBig::from(-3).add_unsigned(&UBig::from(5u8)), IBig::from(2));
    assert_eq!(IBig::from(-5).add_unsigned(&UBig::ZERO), IBig::from(-5));
    // A single-digit sum that overflows the 16-bit digit width into a second digit.
    assert_eq!(
        IBig::from(30000).add_unsigned(&UBig::from(40000u32)),
        IBig::from(70000)
    );

    // A single-digit signed value plus a large unsigned value.
    let big = UBig::from(1u8) << 256;
    assert_eq!(
        IBig::from(-1).add_unsigned(&big),
        IBig::from(&big) - IBig::from(1)
    );

    // A large signed value plus a single unsigned digit.
    assert_eq!(
        (IBig::from(-1) << 256).add_unsigned(&UBig::from(1u8)),
        (IBig::from(-1) << 256) + IBig::from(1)
    );

    // Large + large, with the signed operand longer.
    assert_eq!(
        (IBig::from(-1) << 256).add_unsigned(&(UBig::from(1u8) << 100)),
        (IBig::from(-1) << 256) + (IBig::from(1) << 100)
    );
    // Large + large, with the unsigned operand longer.
    assert_eq!(
        (IBig::from(-1) << 100).add_unsigned(&(UBig::from(1u8) << 256)),
        (IBig::from(1) << 256) - (IBig::from(1) << 100)
    );
}

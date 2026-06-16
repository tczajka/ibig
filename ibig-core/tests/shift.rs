//! Integration tests for small bit shifts (by less than a digit's width).

use ibig_core::{
    Digit, IDigit, is_negative, shl_small_digit, shl_small_idigit, shl_small_signed,
    shl_small_unsigned, shr_small_signed, shr_small_unsigned,
};
use proptest::collection::vec;
use proptest::prelude::*;

fn digit(n: u8) -> Digit {
    Digit::from(n)
}

fn idigit(n: i8) -> IDigit {
    IDigit::from(n)
}

#[test]
fn shl_small_unsigned_basic() {
    let mut a = [digit(0b101), Digit::ZERO];
    assert_eq!(shl_small_unsigned(&mut a, 2), Digit::ZERO);
    assert_eq!(a, [digit(0b10100), Digit::ZERO]);

    // Carry across digits and an overflow out the top.
    let mut a = [Digit::MAX, Digit::MAX];
    assert_eq!(shl_small_unsigned(&mut a, 1), digit(1));
    assert_eq!(a, [!digit(1), Digit::MAX]);

    // A zero shift is the identity with no overflow.
    let mut a = [Digit::MAX, digit(7)];
    assert_eq!(shl_small_unsigned(&mut a, 0), Digit::ZERO);
    assert_eq!(a, [Digit::MAX, digit(7)]);

    // Empty slice is allowed.
    let mut empty: [Digit; 0] = [];
    assert_eq!(shl_small_unsigned(&mut empty, 3), Digit::ZERO);
}

#[test]
fn shr_small_unsigned_basic() {
    let mut a = [digit(0b10100), Digit::ZERO];
    shr_small_unsigned(&mut a, 2);
    assert_eq!(a, [digit(0b101), Digit::ZERO]);

    // The high bits of the low digit are pulled down from the next digit.
    let mut a = [Digit::ZERO, digit(1)];
    shr_small_unsigned(&mut a, 1);
    assert_eq!(a, [digit(1) << (Digit::BITS - 1), Digit::ZERO]);

    // A zero shift is the identity.
    let mut a = [Digit::MAX, digit(7)];
    shr_small_unsigned(&mut a, 0);
    assert_eq!(a, [Digit::MAX, digit(7)]);

    // Empty slice is allowed.
    let mut empty: [Digit; 0] = [];
    shr_small_unsigned(&mut empty, 3);
}

#[test]
fn shl_small_digit_basic() {
    assert_eq!(
        shl_small_digit(digit(0b101), 2),
        (digit(0b10100), Digit::ZERO)
    );

    // The top bit shifts into the high digit.
    assert_eq!(shl_small_digit(Digit::MAX, 1), (!digit(1), digit(1)));

    // A zero shift is the identity with a zero high digit.
    assert_eq!(shl_small_digit(digit(7), 0), (digit(7), Digit::ZERO));
}

#[test]
fn shl_small_signed_basic() {
    // -1 << 1 == -2, with a sign-extended overflow.
    let mut a = [Digit::MAX];
    assert_eq!(shl_small_signed(&mut a, 1), idigit(-1));
    assert_eq!(a, [!digit(1)]);

    // A non-negative value sign-extends to zero overflow.
    let mut a = [digit(0b101)];
    assert_eq!(shl_small_signed(&mut a, 2), IDigit::ZERO);
    assert_eq!(a, [digit(0b10100)]);

    // A zero shift yields the sign extension as the overflow digit.
    let mut a = [Digit::MAX]; // -1
    assert_eq!(shl_small_signed(&mut a, 0), idigit(-1));
    assert_eq!(a, [Digit::MAX]);
    let mut a = [digit(7)]; // +7
    assert_eq!(shl_small_signed(&mut a, 0), IDigit::ZERO);
    assert_eq!(a, [digit(7)]);
}

#[test]
fn shl_small_idigit_basic() {
    // -1 << 1 == -2, spanning two digits.
    assert_eq!(shl_small_idigit(idigit(-1), 1), (!digit(1), idigit(-1)));

    // A non-negative value has a sign-extended (zero) high digit.
    assert_eq!(
        shl_small_idigit(idigit(0b101), 2),
        (digit(0b10100), IDigit::ZERO)
    );

    // A zero shift yields the sign extension as the high digit.
    assert_eq!(shl_small_idigit(idigit(-1), 0), (Digit::MAX, idigit(-1)));
    assert_eq!(shl_small_idigit(idigit(7), 0), (digit(7), IDigit::ZERO));
}

#[test]
fn shr_small_signed_basic() {
    // -4 >> 1 == -2 (arithmetic shift).
    let mut a = [!digit(3)];
    shr_small_signed(&mut a, 1);
    assert_eq!(a, [!digit(1)]);

    // The sign extends into the top digit across a multi-digit value.
    let mut a = [Digit::ZERO, Digit::MAX]; // negative
    shr_small_signed(&mut a, 1);
    assert_eq!(a, [digit(1) << (Digit::BITS - 1), Digit::MAX]);
}

proptest! {
    // Shifting left by `shift`, appending the overflow digit, then shifting right by `shift`
    // restores the whole original value (the appended digit ends up zero).
    #[test]
    fn shl_then_shr(digits in vec(any::<Digit>(), 0..20), shift in 0u32..Digit::BITS) {
        let mut a = digits.clone();
        let overflow = shl_small_unsigned(&mut a, shift);
        a.push(overflow);
        shr_small_unsigned(&mut a, shift);

        let mut expected = digits.clone();
        expected.push(Digit::ZERO);
        prop_assert_eq!(a, expected);
    }

    // The same round-trip for the signed (arithmetic) shifts: the appended digit ends up as
    // the value's sign extension.
    #[test]
    fn shl_then_shr_signed(digits in vec(any::<Digit>(), 1..20), shift in 0u32..Digit::BITS) {
        let mut a = digits.clone();
        let overflow = shl_small_signed(&mut a, shift);
        a.push(overflow.cast_unsigned());
        shr_small_signed(&mut a, shift);

        let mut expected = digits.clone();
        expected.push(if is_negative(&digits) { Digit::MAX } else { Digit::ZERO });
        prop_assert_eq!(a, expected);
    }

    // The single-digit shifts match the slice shifts on a one-digit slice.
    #[test]
    fn shl_small_digit_matches_slice(d: Digit, shift in 0u32..Digit::BITS) {
        let mut a = [d];
        let overflow = shl_small_unsigned(&mut a, shift);
        prop_assert_eq!(shl_small_digit(d, shift), (a[0], overflow));
    }

    #[test]
    fn shl_small_idigit_matches_slice(d: IDigit, shift in 0u32..Digit::BITS) {
        let mut a = [d.cast_unsigned()];
        let overflow = shl_small_signed(&mut a, shift);
        prop_assert_eq!(shl_small_idigit(d, shift), (a[0], overflow));
    }
}

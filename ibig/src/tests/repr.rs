//! Tests of the `UBig` and `IBig` internal representations.

use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
    Digits,
};
use crate::{IBig, UBig};
use ibig_core::{Digit, IDigit};
use smallvec::{SmallVec, smallvec};

const fn digit(n: u16) -> Digit {
    Digit::from_u16(n)
}

const fn idigit(n: i16) -> IDigit {
    IDigit::from_i16(n)
}

#[test]
fn ubig_from_digit() {
    // Every value, including zero, is a single inline digit.
    assert_eq!(
        UBig::from_digit(Digit::ZERO).into_digits(),
        Small(Digit::ZERO)
    );
    assert_eq!(UBig::from_digit(digit(42)).into_digits(), Small(digit(42)));
}

#[test]
fn ibig_from_digit() {
    // A single signed digit is stored inline, as its two's complement bit pattern.
    assert_eq!(IBig::from_digit(idigit(0)).into_digits(), Small(idigit(0)));
    assert_eq!(
        IBig::from_digit(idigit(42)).into_digits(),
        Small(idigit(42))
    );
    // -1 is all-ones in two's complement.
    assert_eq!(
        IBig::from_digit(idigit(-1)).into_digits(),
        Small(idigit(-1))
    );
}

#[test]
fn ubig_from_two_digits() {
    // A zero high digit is redundant and dropped.
    assert_eq!(
        UBig::from_two_digits(digit(5), Digit::ZERO).into_digits(),
        Small(digit(5))
    );
    assert_eq!(
        UBig::from_two_digits(Digit::ZERO, Digit::ZERO).into_digits(),
        Small(Digit::ZERO)
    );
    // A nonzero high digit is kept, even above a zero low digit.
    assert_eq!(
        UBig::from_two_digits(digit(5), digit(1)).into_digits(),
        Large(smallvec![digit(5), digit(1)])
    );
    assert_eq!(
        UBig::from_two_digits(Digit::ZERO, Digit::MAX).into_digits(),
        Large(smallvec![Digit::ZERO, Digit::MAX])
    );
}

#[test]
fn ibig_from_two_digits() {
    // A high digit that is a redundant sign extension of the low digit is dropped.
    assert_eq!(
        IBig::from_two_digits(digit(5), idigit(0)).into_digits(),
        Small(idigit(5))
    );
    assert_eq!(
        IBig::from_two_digits(Digit::MAX, idigit(-1)).into_digits(),
        Small(idigit(-1))
    );
    // A high digit needed to carry the sign is kept.
    assert_eq!(
        IBig::from_two_digits(Digit::MAX, idigit(0)).into_digits(),
        Large(smallvec![Digit::MAX, digit(0)])
    );
    assert_eq!(
        IBig::from_two_digits(digit(0), idigit(-1)).into_digits(),
        Large(smallvec![digit(0), Digit::MAX])
    );
    // A high digit with significant bits of its own is kept.
    assert_eq!(
        IBig::from_two_digits(digit(5), idigit(2)).into_digits(),
        Large(smallvec![digit(5), digit(2)])
    );
}

#[test]
fn ubig_const_from_digits() {
    // Usable in a `const` context.
    const FIVE: UBig = UBig::const_from_digits(&[digit(5)]);
    assert_eq!(FIVE.into_digits(), Small(digit(5)));

    // An empty slice normalizes to the single digit `[0]`.
    assert_eq!(
        UBig::const_from_digits(&[]).into_digits(),
        Small(Digit::ZERO)
    );
    // An all-zero slice also normalizes to `[0]`.
    assert_eq!(
        UBig::const_from_digits(&[Digit::ZERO, Digit::ZERO]).into_digits(),
        Small(Digit::ZERO)
    );
    // Most-significant zero digits are stripped.
    assert_eq!(
        UBig::const_from_digits(&[digit(7), Digit::ZERO]).into_digits(),
        Small(digit(7))
    );
    // A full `INLINE_DIGITS`-length slice is kept verbatim and stays inline.
    const FULL: UBig = UBig::const_from_digits(&[digit(1), digit(2), digit(3), digit(4)]);
    assert_eq!(
        FULL.into_digits(),
        Large(smallvec![digit(1), digit(2), digit(3), digit(4)])
    );
    let Large(buf) = FULL.into_digits() else {
        panic!("a multi-digit value is not `Large`");
    };
    assert!(!buf.spilled());
}

#[test]
#[should_panic]
fn ubig_const_from_digits_panics_when_too_long() {
    // More than `INLINE_DIGITS` (= 4) digits cannot be stored inline.
    UBig::const_from_digits(&[digit(1), digit(2), digit(3), digit(4), digit(5)]);
}

#[test]
fn ibig_const_from_digits() {
    // Usable in a `const` context.
    const FIVE: IBig = IBig::const_from_digits(&[digit(5)]);
    assert_eq!(FIVE.into_digits(), Small(idigit(5)));

    // A single two's complement digit is kept as-is, positive or negative.
    assert_eq!(
        IBig::const_from_digits(&[digit(42)]).into_digits(),
        Small(idigit(42))
    );
    assert_eq!(
        IBig::const_from_digits(&[Digit::MAX]).into_digits(),
        Small(idigit(-1))
    );
    // A redundant zero sign-extension above a non-negative digit is stripped.
    assert_eq!(
        IBig::const_from_digits(&[digit(5), digit(0)]).into_digits(),
        Small(idigit(5))
    );
    // An all-ones sign-extension above a negative digit is stripped.
    assert_eq!(
        IBig::const_from_digits(&[Digit::MAX, Digit::MAX]).into_digits(),
        Small(idigit(-1))
    );
    // A leading zero digit needed to keep `2^W - 1` positive is preserved.
    assert_eq!(
        IBig::const_from_digits(&[Digit::MAX, digit(0)]).into_digits(),
        Large(smallvec![Digit::MAX, digit(0)])
    );
}

#[test]
#[should_panic]
fn ibig_const_from_digits_panics_on_empty() {
    IBig::const_from_digits(&[]);
}

#[test]
#[should_panic]
fn ibig_const_from_digits_panics_when_too_long() {
    // More than `INLINE_DIGITS` (= 4) digits cannot be stored inline.
    IBig::const_from_digits(&[digit(1), digit(2), digit(3), digit(4), digit(5)]);
}

#[test]
fn ubig_from_digits_normalizes() {
    // Empty and all-zero buffers normalize to the single digit `[0]`.
    assert_eq!(
        UBig::from_digits(smallvec![]).into_digits(),
        Small(Digit::ZERO)
    );
    assert_eq!(
        UBig::from_digits(smallvec![Digit::ZERO, Digit::ZERO]).into_digits(),
        Small(Digit::ZERO)
    );
    // Most-significant zero digits are stripped.
    assert_eq!(
        UBig::from_digits(smallvec![digit(7), Digit::ZERO]).into_digits(),
        Small(digit(7))
    );
    assert_eq!(
        UBig::from_digits(smallvec![digit(1), digit(2), Digit::ZERO]).into_digits(),
        Large(smallvec![digit(1), digit(2)])
    );
}

#[test]
fn ubig_from_digits_inlines_small() {
    // A spilled buffer that normalizes to few digits is moved back inline, including
    // a value that collapses to a single digit.
    let mut digits: Digits = SmallVec::with_capacity(100);
    digits.push(digit(5));
    digits.push(Digit::ZERO);
    assert!(digits.spilled());
    assert_eq!(UBig::from_digits(digits).into_digits(), Small(digit(5)));
}

#[test]
fn ubig_from_digits_shrinks_capacity() {
    // A heap buffer far larger than its contents is compacted but stays on the heap.
    let mut digits: Digits = SmallVec::with_capacity(100);
    for i in 1..=5 {
        digits.push(digit(i));
    }
    let Large(buf) = UBig::from_digits(digits).into_digits() else {
        panic!("a multi-digit value is not `Large`");
    };
    assert_eq!(buf.len(), 5);
    assert!(buf.spilled());
    assert!(buf.capacity() < 100);
}

#[test]
fn ubig_from_digits_no_need_to_shrink() {
    // A large heap buffer that is already tightly sized is kept as-is, not shrunk.
    let mut digits: Digits = SmallVec::with_capacity(10);
    for i in 1..=10 {
        digits.push(digit(i));
    }
    assert!(digits.spilled());
    let Large(buf) = UBig::from_digits(digits).into_digits() else {
        panic!("a multi-digit value is not `Large`");
    };
    assert_eq!(buf.len(), 10);
    assert!(buf.spilled());
}

#[test]
fn ibig_from_digits_normalizes() {
    assert_eq!(
        IBig::from_digits(smallvec![digit(0), digit(0)]).into_digits(),
        Small(idigit(0))
    );
    // A redundant zero sign-extension above a non-negative digit is stripped.
    assert_eq!(
        IBig::from_digits(smallvec![digit(5), digit(0), digit(0)]).into_digits(),
        Small(idigit(5))
    );
    // For a negative value the sign-extension digits are all-ones, and are stripped.
    assert_eq!(
        IBig::from_digits(smallvec![Digit::MAX, Digit::MAX, Digit::MAX]).into_digits(),
        Small(idigit(-1))
    );
}

#[test]
#[should_panic]
fn ibig_from_digits_panics_on_empty() {
    IBig::from_digits(smallvec![]);
}

#[test]
fn ibig_from_digits_keeps_needed_sign_digit() {
    // The unsigned value 2^W - 1 (all-ones in one digit) is negative as a single two's
    // complement digit, so representing it as a positive number needs a leading zero digit
    // that must not be stripped.
    assert_eq!(
        IBig::from_digits(smallvec![Digit::MAX, digit(0)]).into_digits(),
        Large(smallvec![Digit::MAX, digit(0)])
    );
    // Likewise a leading all-ones digit is needed below a non-negative digit to stay
    // negative.
    assert_eq!(
        IBig::from_digits(smallvec![digit(0), Digit::MAX]).into_digits(),
        Large(smallvec![digit(0), Digit::MAX])
    );
}

#[test]
fn ibig_from_digits_shrinks_capacity() {
    // A heap buffer far larger than its contents is compacted but stays on the heap.
    let mut digits: Digits = SmallVec::with_capacity(100);
    for i in 1..=5 {
        digits.push(digit(i));
    }
    let Large(buf) = IBig::from_digits(digits).into_digits() else {
        panic!("a multi-digit value is not `Large`");
    };
    assert_eq!(buf.len(), 5);
    assert!(buf.spilled());
    assert!(buf.capacity() < 100);
}

#[test]
fn ibig_from_digits_no_need_to_shrink() {
    // A large heap buffer that is already tightly sized is kept as-is, not shrunk.
    let mut digits: Digits = SmallVec::with_capacity(10);
    for i in 1..=10 {
        digits.push(digit(i));
    }
    assert!(digits.spilled());
    let Large(buf) = IBig::from_digits(digits).into_digits() else {
        panic!("a multi-digit value is not `Large`");
    };
    assert_eq!(buf.len(), 10);
    assert!(buf.spilled());
}

#[test]
fn ibig_from_digits_inlines_small() {
    // A spilled buffer that normalizes to few digits is moved back inline, including a
    // value that collapses to a single digit.
    let mut digits: Digits = SmallVec::with_capacity(100);
    digits.push(digit(5));
    digits.push(digit(0));
    assert!(digits.spilled());
    assert_eq!(IBig::from_digits(digits).into_digits(), Small(idigit(5)));
}

#[test]
fn ubig_as_digits() {
    // A single digit (including zero) is reported as `Small`.
    assert_eq!(
        UBig::from_digit(Digit::ZERO).as_digits(),
        Small(Digit::ZERO)
    );
    assert_eq!(UBig::from_digit(digit(9)).as_digits(), Small(digit(9)));
    // A multi-digit value is reported as `Large` over the full digit slice.
    assert_eq!(
        UBig::from_digits(smallvec![digit(1), digit(2)]).as_digits(),
        Large([digit(1), digit(2)].as_slice())
    );
}

#[test]
fn ibig_as_digits() {
    // A single signed digit is reported as `Small`.
    assert_eq!(IBig::from_digit(idigit(0)).as_digits(), Small(idigit(0)));
    assert_eq!(IBig::from_digit(idigit(-1)).as_digits(), Small(idigit(-1)));
    // A value needing two digits does not fit in a single signed digit, so it is `Large`.
    assert_eq!(
        IBig::from_digits(smallvec![Digit::MAX, digit(0)]).as_digits(),
        Large([Digit::MAX, digit(0)].as_slice())
    );
}

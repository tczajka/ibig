//! Integration tests for `midpoint`.

use ibig::proptest::{ibig_up_to_bits, ubig_up_to_bits};
use ibig::{IBig, UBig};
use proptest::prelude::*;

fn ubig(n: u64) -> UBig {
    UBig::from(n)
}

fn ibig(n: i64) -> IBig {
    IBig::from(n)
}

#[test]
fn ubig_midpoint() {
    // Single-digit (native) path.
    assert_eq!(ubig(4).midpoint(&ubig(8)), ubig(6));
    assert_eq!(ubig(0).midpoint(&ubig(0)), ubig(0));
    assert_eq!(ubig(5).midpoint(&ubig(5)), ubig(5));
    // An odd sum rounds down.
    assert_eq!(ubig(4).midpoint(&ubig(7)), ubig(5));
    // The native path avoids overflow near the top of a digit.
    assert_eq!(
        UBig::from(u64::MAX).midpoint(&UBig::from(u64::MAX)),
        UBig::from(u64::MAX)
    );
    assert_eq!(
        UBig::from(u64::MAX).midpoint(&ubig(0)),
        UBig::from(u64::MAX / 2)
    );

    // Multi-digit and mixed paths.
    let big = UBig::from(1u8) << 200;
    assert_eq!(big.midpoint(&big), big.clone());
    assert_eq!(big.midpoint(&ubig(0)), UBig::from(1u8) << 199);
    // Odd multi-digit sum rounds down.
    assert_eq!((&big + ubig(1)).midpoint(&big), big.clone());
}

#[test]
fn ibig_midpoint() {
    // Single-digit (native) path.
    assert_eq!(ibig(4).midpoint(&ibig(8)), ibig(6));
    assert_eq!(ibig(-4).midpoint(&ibig(-8)), ibig(-6));
    assert_eq!(ibig(-5).midpoint(&ibig(5)), ibig(0));
    // Odd sums round toward zero on both sides.
    assert_eq!(ibig(4).midpoint(&ibig(7)), ibig(5));
    assert_eq!(ibig(-4).midpoint(&ibig(-7)), ibig(-5));
    // A sum of -1 rounds toward zero to 0, not down to -1.
    assert_eq!(ibig(0).midpoint(&ibig(-1)), ibig(0));
    assert_eq!(ibig(2).midpoint(&ibig(-3)), ibig(0));

    // Multi-digit and mixed paths.
    let big = IBig::from(1) << 200;
    assert_eq!((-&big).midpoint(&big), ibig(0));
    assert_eq!(big.midpoint(&ibig(0)), IBig::from(1) << 199);
    // Negative odd multi-digit sum rounds toward zero.
    assert_eq!((-&big - ibig(1)).midpoint(&(-&big)), -&big);
}

proptest! {
    // Midpoint is symmetric and equals `(a + b) / 2` truncated toward zero, verified by doubling.
    #[test]
    fn ubig_midpoint_props(a in ubig_up_to_bits(300), b in ubig_up_to_bits(300)) {
        let m = a.midpoint(&b);
        prop_assert_eq!(&m, &b.midpoint(&a));
        // `2 * m` equals `a + b` or, for an odd sum, one less.
        let remainder = (&a + &b) - (&m + &m);
        prop_assert!(remainder == UBig::ZERO || remainder == UBig::from(1u8));
    }

    #[test]
    fn ibig_midpoint_props(a in ibig_up_to_bits(300), b in ibig_up_to_bits(300)) {
        let m = a.midpoint(&b);
        prop_assert_eq!(&m, &b.midpoint(&a));
        let sum = &a + &b;
        // `2 * m` equals the sum, or differs by 1 toward zero (remainder shares the sum's sign).
        let remainder = &sum - (&m + &m);
        prop_assert!(
            remainder == IBig::ZERO
                || remainder == IBig::from(1)
                || remainder == IBig::from(-1)
        );
        if remainder != IBig::ZERO {
            prop_assert_eq!(remainder.is_negative(), sum.is_negative());
        }
    }
}

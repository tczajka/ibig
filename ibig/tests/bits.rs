//! Integration tests for `UBig` and `IBig` bit operations.

use ibig::proptest::ubig_up_to_bits;
use ibig::{IBig, UBig};
use proptest::prelude::*;

/// `2^k` as a `UBig`.
fn ubig_pow2(k: usize) -> UBig {
    let mut x = UBig::ZERO;
    x.set_bit(k, true);
    x
}

/// `2^k` as an `IBig`.
fn ibig_pow2(k: usize) -> IBig {
    let mut x = IBig::ZERO;
    x.set_bit(k, true);
    x
}

#[test]
fn ubig_bit() {
    // Single digit (fast path).
    let a = UBig::from(0b10010u8);
    assert!(!a.bit(0));
    assert!(a.bit(1));
    assert!(a.bit(4));
    assert!(!a.bit(5));
    assert!(!a.bit(1000));

    // Multi-digit (slow path): 2^100.
    let p = UBig::from(1u128 << 100);
    assert!(!p.bit(0));
    assert!(!p.bit(99));
    assert!(p.bit(100));
    assert!(!p.bit(101));
    assert!(!p.bit(1000));

    // Low bits set across digit boundaries.
    let m = UBig::from(u128::MAX);
    assert!(m.bit(0));
    assert!(m.bit(127));
    assert!(!m.bit(128));
}

#[test]
fn ibig_bit() {
    // Single-digit non-negative (fast path).
    let a = IBig::from(0b10010i8);
    assert!(!a.bit(0));
    assert!(a.bit(1));
    assert!(a.bit(4));
    assert!(!a.bit(5));
    assert!(!a.bit(1000));

    // Single-digit negative (fast path): -1 is all ones.
    let neg1 = IBig::from(-1i8);
    assert!(neg1.bit(0));
    assert!(neg1.bit(1000));

    // -2 = ...11111110.
    let neg2 = IBig::from(-2i8);
    assert!(!neg2.bit(0));
    assert!(neg2.bit(1));
    assert!(neg2.bit(1000));

    // Multi-digit non-negative (slow path): `u64::MAX` gains a sign-zero high digit.
    let big = IBig::from(u64::MAX);
    assert!(big.bit(0));
    assert!(big.bit(63));
    assert!(!big.bit(64));
    assert!(!big.bit(1000));

    // Multi-digit negative (slow path): -(2^100) is zero below bit 100, all ones at and above.
    let neg = IBig::from(-1i128 << 100);
    assert!(!neg.bit(0));
    assert!(!neg.bit(99));
    assert!(neg.bit(100));
    assert!(neg.bit(200));
    assert!(neg.bit(1000));
}

#[test]
fn ubig_set_bit() {
    // Set and clear within a digit.
    let mut a = UBig::from(0b100u8);
    a.set_bit(0, true);
    assert_eq!(a, UBig::from(0b101u8));
    a.set_bit(2, false);
    assert_eq!(a, UBig::from(0b001u8));

    // Setting a high bit grows the value: 0 -> 2^100.
    let mut b = UBig::ZERO;
    b.set_bit(100, true);
    assert_eq!(b, UBig::from(1u128 << 100));
    assert!(b.bit(100));

    // Clearing the only set bit yields zero.
    let mut c = UBig::from(1u8);
    c.set_bit(0, false);
    assert_eq!(c, UBig::ZERO);

    // Clearing a bit far above the value is a no-op.
    let mut d = UBig::from(5u8);
    d.set_bit(1000, false);
    assert_eq!(d, UBig::from(5u8));

    // Re-setting an already-set bit is idempotent; clearing across digits works.
    let mut e = UBig::from(u128::MAX);
    e.set_bit(0, true);
    assert_eq!(e, UBig::from(u128::MAX));
    e.set_bit(0, false);
    assert_eq!(e, UBig::from(u128::MAX - 1));
}

#[test]
fn ibig_set_bit() {
    // Set and clear within a digit.
    let mut a = IBig::from(0b100i8);
    a.set_bit(0, true);
    assert_eq!(a, IBig::from(0b101i8));
    a.set_bit(2, false);
    assert_eq!(a, IBig::from(0b001i8));

    // Setting a digit's most-significant bit of a positive value must not flip the sign.
    // Bit 63 is the top bit of a digit at every word size (64-1 = 2*32-1 = 4*16-1).
    let mut p = IBig::ZERO;
    p.set_bit(63, true);
    assert!(p.is_positive());
    assert!(p.bit(63));
    assert!(!p.bit(64));

    // Setting a bit far above a positive value: 5 -> 5 + 2^100, still positive.
    let mut q = IBig::from(5i8);
    q.set_bit(100, true);
    assert!(q.is_positive());
    assert!(q.bit(0));
    assert!(q.bit(2));
    assert!(q.bit(100));
    assert!(!q.bit(101));

    // -1 is all ones; clearing bit 0 gives -2.
    let mut n = IBig::from(-1i8);
    n.set_bit(0, false);
    assert_eq!(n, IBig::from(-2i8));

    // Clearing a digit's top bit of a negative value keeps it negative.
    let mut m = IBig::from(-1i8);
    m.set_bit(63, false);
    assert!(m.is_negative());
    assert!(!m.bit(63));
    assert!(m.bit(64)); // still sign-extended ones above

    // Setting a high bit of a negative value (already 1) is a no-op.
    let mut r = IBig::from(-5i8);
    r.set_bit(1000, true);
    assert_eq!(r, IBig::from(-5i8));

    // Clearing a high bit of a non-negative value (already 0) is a no-op.
    let mut z = IBig::from(7i8);
    z.set_bit(1000, false);
    assert_eq!(z, IBig::from(7i8));

    // A bit below the top digit of a multi-digit value: modified in place, no growth.
    // `u64::MAX` is multi-digit at every word size (it gains a sign-zero high digit).
    let mut big = IBig::from(u64::MAX);
    big.set_bit(0, false);
    assert_eq!(big, IBig::from(u64::MAX - 1));
    big.set_bit(0, true);
    assert_eq!(big, IBig::from(u64::MAX));
}

#[test]
fn ubig_bit_width() {
    assert_eq!(UBig::ZERO.bit_width(), 0);
    assert_eq!(UBig::from(1u8).bit_width(), 1);
    assert_eq!(UBig::from(0b101u8).bit_width(), 3);
    assert_eq!(UBig::from(u128::MAX).bit_width(), 128);
    assert_eq!(UBig::from(1u128 << 100).bit_width(), 101);
}

#[test]
fn ubig_ilog2() {
    assert_eq!(UBig::from(1u8).ilog2(), 0);
    assert_eq!(UBig::from(0b101u8).ilog2(), 2);
    assert_eq!(UBig::from(u128::MAX).ilog2(), 127);
    assert_eq!(UBig::from(1u128 << 100).ilog2(), 100);
}

#[test]
#[should_panic]
fn ubig_ilog2_zero() {
    let _ = UBig::ZERO.ilog2();
}

#[test]
fn ubig_checked_ilog2() {
    assert_eq!(UBig::ZERO.checked_ilog2(), None);
    assert_eq!(UBig::from(1u8).checked_ilog2(), Some(0));
    assert_eq!(UBig::from(0b101u8).checked_ilog2(), Some(2));
    assert_eq!(UBig::from(1u128 << 100).checked_ilog2(), Some(100));
}

proptest! {
    // A power of two `2^k`: bit_width is `k + 1`, ilog2 is `k`.
    #[test]
    fn ubig_pow2_logs(k in 0usize..1000) {
        let p = ubig_pow2(k);
        prop_assert_eq!(p.bit_width(), k + 1);
        prop_assert_eq!(p.ilog2(), k);
        prop_assert_eq!(p.checked_ilog2(), Some(k));
    }
}

#[test]
fn ibig_ilog2() {
    assert_eq!(IBig::from(1i8).ilog2(), 0);
    assert_eq!(IBig::from(0b101i8).ilog2(), 2);
    assert_eq!(IBig::from(1i128 << 100).ilog2(), 100);
}

#[test]
#[should_panic]
fn ibig_ilog2_zero() {
    let _ = IBig::ZERO.ilog2();
}

#[test]
#[should_panic]
fn ibig_ilog2_negative() {
    let _ = IBig::from(-4i8).ilog2();
}

#[test]
fn ibig_checked_ilog2() {
    assert_eq!(IBig::ZERO.checked_ilog2(), None);
    assert_eq!(IBig::from(1i8).checked_ilog2(), Some(0));
    assert_eq!(IBig::from(0b101i8).checked_ilog2(), Some(2));
    assert_eq!(IBig::from(1i128 << 100).checked_ilog2(), Some(100));
    // Non-positive values have no logarithm.
    assert_eq!(IBig::from(-4i8).checked_ilog2(), None);
}

proptest! {
    #[test]
    fn ibig_pow2_logs(k in 0usize..1000) {
        let p = ibig_pow2(k);
        prop_assert_eq!(p.ilog2(), k);
        prop_assert_eq!(p.checked_ilog2(), Some(k));
    }
}

#[test]
fn ubig_trailing_zeros() {
    assert_eq!(UBig::from(1u8).trailing_zeros(), 0);
    assert_eq!(UBig::from(0b101000u8).trailing_zeros(), 3);
    // Multi-digit: 2^100 has 100 trailing zeros.
    assert_eq!(UBig::from(1u128 << 100).trailing_zeros(), 100);
}

#[test]
#[should_panic]
fn ubig_trailing_zeros_zero() {
    let _ = UBig::ZERO.trailing_zeros();
}

#[test]
fn ibig_trailing_zeros() {
    assert_eq!(IBig::from(0b101000i16).trailing_zeros(), 3);
    assert_eq!(IBig::from(-4i8).trailing_zeros(), 2);
    // Multi-digit negative: -(2^100) has 100 trailing zeros.
    assert_eq!(IBig::from(-1i128 << 100).trailing_zeros(), 100);
}

#[test]
#[should_panic]
fn ibig_trailing_zeros_zero() {
    let _ = IBig::ZERO.trailing_zeros();
}

proptest! {
    // A power of two `2^k` has exactly `k` trailing zeros.
    #[test]
    fn pow2_trailing_zeros(k in 0usize..1000) {
        prop_assert_eq!(ubig_pow2(k).trailing_zeros(), k);
        prop_assert_eq!(ibig_pow2(k).trailing_zeros(), k);
    }
}

#[test]
fn ubig_trailing_ones() {
    assert_eq!(UBig::ZERO.trailing_ones(), 0);
    assert_eq!(UBig::from(0b100111u8).trailing_ones(), 3);
    // All bits of u128::MAX set; the result is its width, and it never panics (finite).
    assert_eq!(UBig::from(u128::MAX).trailing_ones(), 128);
}

#[test]
fn ibig_trailing_ones() {
    assert_eq!(IBig::ZERO.trailing_ones(), 0);
    assert_eq!(IBig::from(0b100111i16).trailing_ones(), 3);
    // -4 = ...11111100, -3 = ...11111101, -2 = ...11111110.
    assert_eq!(IBig::from(-4i8).trailing_ones(), 0);
    assert_eq!(IBig::from(-3i8).trailing_ones(), 1);
    assert_eq!(IBig::from(-2i8).trailing_ones(), 0);
    // Multi-digit (slow path): u64::MAX is 64 set bits below a zero sign digit.
    assert_eq!(IBig::from(u64::MAX).trailing_ones(), 64);
}

#[test]
#[should_panic]
fn ibig_trailing_ones_minus_one() {
    let _ = IBig::from(-1i8).trailing_ones();
}

#[test]
fn ubig_count_ones() {
    assert_eq!(UBig::ZERO.count_ones(), 0);
    assert_eq!(UBig::from(0b10110u8).count_ones(), 3);
    assert_eq!(UBig::from(u128::MAX).count_ones(), 128);
    // Multi-digit values, counting set bits across every digit.
    assert_eq!(UBig::from_le_bytes(&[0xff; 20]).count_ones(), 160);
    assert_eq!(
        UBig::from_le_bytes(&[0b101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b11]).count_ones(),
        4
    );
}

#[test]
fn ubig_is_power_of_two() {
    let cases = [
        (UBig::ZERO, false),
        (UBig::from(1u8), true),
        (UBig::from(8u8), true),
        (UBig::from(6u8), false),
        // Multi-digit.
        (UBig::from(1u128 << 100), true),
        (UBig::from(3u128 << 100), false),
        (UBig::from(u128::MAX), false),
    ];
    for (n, expected) in cases {
        assert_eq!(n.is_power_of_two(), expected);
    }
}

#[test]
fn ubig_next_power_of_two() {
    // (value, smallest power of two >= value).
    let mut cases: Vec<(UBig, UBig)> = vec![
        (UBig::ZERO, UBig::from(1u8)),
        (UBig::from(1u8), UBig::from(1u8)),
        (UBig::from(5u8), UBig::from(8u8)),
        (UBig::from(8u8), UBig::from(8u8)),
        // A digit with the high bit set rounds up past the digit width (fast-path overflow on
        // 64-bit, where `u64::MAX` is a single digit).
        (UBig::from(u64::MAX), UBig::from(1u128 << 64)),
        // Multi-digit and growing past the current width.
        (UBig::from((1u128 << 100) + 1), UBig::from(1u128 << 101)),
        // u128::MAX rounds up to 2^128.
        (UBig::from(u128::MAX), ubig_pow2(128)),
    ];
    // Large exact powers of two round up to themselves; one more rounds to the next power.
    for k in [63usize, 64, 127, 128, 200, 255, 256, 500] {
        cases.push((ubig_pow2(k), ubig_pow2(k)));
        let mut above = ubig_pow2(k); // 2^k + 1 (bit 0 is clear for k > 0)
        above.set_bit(0, true);
        cases.push((above, ubig_pow2(k + 1)));
    }

    for (value, expected) in cases {
        assert_eq!(value.next_power_of_two(), expected);
    }
}

proptest! {
    // The next power of two of any value is a power of two.
    #[test]
    fn ubig_next_power_of_two_is_power(x in ubig_up_to_bits(1000)) {
        prop_assert!(x.next_power_of_two().is_power_of_two());
    }
}

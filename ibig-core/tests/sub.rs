//! Integration tests for subtraction.

use ibig_core::{
    Digit, SignedDigit, add_signed_signed, add_unsigned_signed, add_unsigned_unsigned,
    extend_signed, neg, sign_extension, sub_reverse_signed_sdigit, sub_reverse_signed_signed,
    sub_reverse_unsigned_unsigned_same_len, sub_signed_sdigit, sub_signed_signed, sub_unsigned_1,
    sub_unsigned_borrow, sub_unsigned_digit, sub_unsigned_sdigit, sub_unsigned_signed,
    sub_unsigned_unsigned, sub_unsigned_unsigned_same_len,
};
use proptest::collection::vec;
use proptest::prelude::*;

fn digit(n: u8) -> Digit {
    Digit::from(n)
}

fn sdigit(n: i8) -> SignedDigit {
    SignedDigit::from(n)
}

#[test]
fn sub_unsigned_unsigned_basic() {
    let mut a = [digit(4), digit(9), digit(3)];
    assert!(!sub_unsigned_unsigned(&mut a, &[digit(3), digit(7)]));
    assert_eq!(a, [digit(1), digit(2), digit(3)]);

    // A borrow out of the low digits propagates through the high digits.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(4)];
    assert!(!sub_unsigned_unsigned(&mut a, &[digit(1)]));
    assert_eq!(a, [Digit::MAX, Digit::MAX, digit(3)]);

    // A borrow out of the most-significant digit.
    let mut a = [Digit::ZERO, Digit::ZERO];
    assert!(sub_unsigned_unsigned(&mut a, &[digit(1), Digit::ZERO]));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);

    // An empty `rhs` is allowed.
    let mut a = [digit(5)];
    assert!(!sub_unsigned_unsigned(&mut a, &[]));
    assert_eq!(a, [digit(5)]);
}

#[test]
#[should_panic]
fn sub_unsigned_unsigned_rhs_longer() {
    sub_unsigned_unsigned(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
fn sub_unsigned_unsigned_same_len_basic() {
    let mut a = [digit(4), digit(6)];
    assert!(!sub_unsigned_unsigned_same_len(
        &mut a,
        &[digit(3), digit(4)]
    ));
    assert_eq!(a, [digit(1), digit(2)]);

    // The borrow propagates across digits and out the top.
    let mut a = [Digit::ZERO, Digit::ZERO];
    assert!(sub_unsigned_unsigned_same_len(
        &mut a,
        &[digit(1), Digit::ZERO]
    ));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);

    // Empty slices are allowed.
    assert!(!sub_unsigned_unsigned_same_len(&mut [], &[]));
}

#[test]
#[should_panic]
fn sub_unsigned_unsigned_same_len_mismatched() {
    sub_unsigned_unsigned_same_len(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
fn sub_reverse_unsigned_unsigned_same_len_basic() {
    // a = b - a == [7, 9] - [4, 6] == [3, 3].
    let mut a = [digit(4), digit(6)];
    assert!(!sub_reverse_unsigned_unsigned_same_len(
        &mut a,
        &[digit(7), digit(9)]
    ));
    assert_eq!(a, [digit(3), digit(3)]);

    // A borrow propagates across digits and out the top: [0, 0] - [1, 0].
    let mut a = [digit(1), Digit::ZERO];
    assert!(sub_reverse_unsigned_unsigned_same_len(
        &mut a,
        &[Digit::ZERO, Digit::ZERO]
    ));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);

    // Empty slices are allowed.
    assert!(!sub_reverse_unsigned_unsigned_same_len(&mut [], &[]));
}

#[test]
#[should_panic]
fn sub_reverse_unsigned_unsigned_same_len_mismatched() {
    sub_reverse_unsigned_unsigned_same_len(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
fn sub_unsigned_digit_basic() {
    let mut a = [digit(8), digit(2)];
    assert!(!sub_unsigned_digit(&mut a, digit(7)));
    assert_eq!(a, [digit(1), digit(2)]);

    let mut a = [Digit::ZERO, digit(4)];
    assert!(!sub_unsigned_digit(&mut a, digit(1)));
    assert_eq!(a, [Digit::MAX, digit(3)]);

    let mut a = [Digit::ZERO, Digit::ZERO];
    assert!(sub_unsigned_digit(&mut a, digit(1)));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);
}

#[test]
#[should_panic]
fn sub_unsigned_digit_empty() {
    sub_unsigned_digit(&mut [], digit(1));
}

#[test]
fn sub_unsigned_borrow_basic() {
    let mut a = [Digit::ZERO, digit(1)];
    assert!(!sub_unsigned_borrow(&mut a, true));
    assert_eq!(a, [Digit::MAX, digit(0)]);

    // Without an incoming borrow nothing changes.
    let mut a = [Digit::ZERO];
    assert!(!sub_unsigned_borrow(&mut a, false));
    assert_eq!(a, [Digit::ZERO]);

    // A borrow out of the most-significant digit.
    let mut a = [Digit::ZERO, Digit::ZERO];
    assert!(sub_unsigned_borrow(&mut a, true));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);

    // An empty slice passes the borrow through.
    assert!(sub_unsigned_borrow(&mut [], true));
    assert!(!sub_unsigned_borrow(&mut [], false));
}

#[test]
fn sub_unsigned_1_basic() {
    let mut a = [digit(2), digit(2)];
    assert!(!sub_unsigned_1(&mut a));
    assert_eq!(a, [digit(1), digit(2)]);

    // The decrement ripples through all-zeros digits.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(4)];
    assert!(!sub_unsigned_1(&mut a));
    assert_eq!(a, [Digit::MAX, Digit::MAX, digit(3)]);

    // All zeros overflows.
    let mut a = [Digit::ZERO, Digit::ZERO];
    assert!(sub_unsigned_1(&mut a));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);

    // An empty slice overflows immediately.
    assert!(sub_unsigned_1(&mut []));
}

#[test]
fn sub_unsigned_signed_basic() {
    // Subtracting a non-negative value: 8 - 3 == 5.
    let mut a = [digit(8), digit(2)];
    assert_eq!(sub_unsigned_signed(&mut a, &[digit(3)]), sdigit(0));
    assert_eq!(a, [digit(5), digit(2)]);

    // Subtracting a negative adds: 5 - -1 == 6.
    let mut a = [digit(5)];
    assert_eq!(sub_unsigned_signed(&mut a, &[Digit::MAX]), sdigit(0));
    assert_eq!(a, [digit(6)]);

    // A negative result: 2 - 3 == -1, carry -1.
    let mut a = [digit(2)];
    assert_eq!(sub_unsigned_signed(&mut a, &[digit(3)]), sdigit(-1));
    assert_eq!(a, [Digit::MAX]);

    // An overflow into a +1 carry: (2^bits - 1) - -1 == 2^bits.
    let mut a = [Digit::MAX];
    assert_eq!(sub_unsigned_signed(&mut a, &[Digit::MAX]), sdigit(1));
    assert_eq!(a, [Digit::ZERO]);

    // A borrow ripples through the high zero digits: 2^(2*bits) - 1.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(sub_unsigned_signed(&mut a, &[digit(1)]), sdigit(0));
    assert_eq!(a, [Digit::MAX, Digit::MAX, Digit::ZERO]);
}

#[test]
#[should_panic]
fn sub_unsigned_signed_rhs_longer() {
    sub_unsigned_signed(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
#[should_panic]
fn sub_unsigned_signed_rhs_empty() {
    sub_unsigned_signed(&mut [digit(1)], &[]);
}

#[test]
fn sub_unsigned_sdigit_basic() {
    // 5 - -3 == 8.
    let mut a = [digit(5)];
    assert_eq!(sub_unsigned_sdigit(&mut a, sdigit(-3)), sdigit(0));
    assert_eq!(a, [digit(8)]);

    // A borrow into the high digits: [0, 1] - 1 == [MAX, 0].
    let mut a = [Digit::ZERO, digit(1)];
    assert_eq!(sub_unsigned_sdigit(&mut a, sdigit(1)), sdigit(0));
    assert_eq!(a, [Digit::MAX, digit(0)]);

    // A negative result: 0 - 1 == -1.
    let mut a = [Digit::ZERO];
    assert_eq!(sub_unsigned_sdigit(&mut a, sdigit(1)), sdigit(-1));
    assert_eq!(a, [Digit::MAX]);

    // A positive overflow into a +1 carry: (2^bits - 1) - -1 == 2^bits.
    let mut a = [Digit::MAX];
    assert_eq!(sub_unsigned_sdigit(&mut a, sdigit(-1)), sdigit(1));
    assert_eq!(a, [Digit::ZERO]);
}

#[test]
#[should_panic]
fn sub_unsigned_sdigit_empty() {
    sub_unsigned_sdigit(&mut [], sdigit(1));
}

#[test]
fn sub_signed_signed_basic() {
    // Two non-negative values.
    let mut a = [digit(7), digit(3)];
    assert_eq!(sub_signed_signed(&mut a, &[digit(5)]), sdigit(0));
    assert_eq!(a, [digit(2), digit(3)]);

    // 3 - 5 == -2.
    let mut a = [digit(3)];
    assert_eq!(sub_signed_signed(&mut a, &[digit(5)]), sdigit(-1));
    assert_eq!(a, [Digit::MAX - digit(1)]);

    // -1 - -1 == 0.
    let mut a = [Digit::MAX];
    assert_eq!(sub_signed_signed(&mut a, &[Digit::MAX]), sdigit(0));
    assert_eq!(a, [Digit::ZERO]);

    // Subtracting a negative adds: 5 - -3 == 8 (`Digit::MAX - 2` is -3).
    let mut a = [digit(5)];
    assert_eq!(
        sub_signed_signed(&mut a, &[Digit::MAX - digit(2)]),
        sdigit(0)
    );
    assert_eq!(a, [digit(8)]);

    // A negative result that overflows into the returned digit: signed_min - 1.
    let signed_min = (Digit::MAX >> 1) + digit(1);
    let mut a = [signed_min];
    assert_eq!(sub_signed_signed(&mut a, &[digit(1)]), sdigit(-1));
    assert_eq!(a, [signed_min - digit(1)]);

    // A borrow ripples through the high zero digits: 2^(2*bits) - 1.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(sub_signed_signed(&mut a, &[digit(1)]), sdigit(0));
    assert_eq!(a, [Digit::MAX, Digit::MAX, Digit::ZERO]);
}

#[test]
#[should_panic]
fn sub_signed_signed_rhs_longer() {
    sub_signed_signed(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
#[should_panic]
fn sub_signed_signed_rhs_empty() {
    sub_signed_signed(&mut [digit(1)], &[]);
}

#[test]
fn sub_signed_sdigit_basic() {
    // Two non-negative values.
    let mut a = [digit(7), digit(3)];
    assert_eq!(sub_signed_sdigit(&mut a, sdigit(5)), sdigit(0));
    assert_eq!(a, [digit(2), digit(3)]);

    // 3 - 5 == -2.
    let mut a = [digit(3)];
    assert_eq!(sub_signed_sdigit(&mut a, sdigit(5)), sdigit(-1));
    assert_eq!(a, [Digit::MAX - digit(1)]);

    // Subtracting a negative adds: 5 - -3 == 8.
    let mut a = [digit(5)];
    assert_eq!(sub_signed_sdigit(&mut a, sdigit(-3)), sdigit(0));
    assert_eq!(a, [digit(8)]);

    // A borrow ripples through the high zero digits: 2^(2*bits) - 1.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(sub_signed_sdigit(&mut a, sdigit(1)), sdigit(0));
    assert_eq!(a, [Digit::MAX, Digit::MAX, Digit::ZERO]);
}

#[test]
#[should_panic]
fn sub_signed_sdigit_empty() {
    sub_signed_sdigit(&mut [], sdigit(1));
}

#[test]
fn sub_reverse_signed_signed_basic() {
    // a = b - a == 5 - 3 == 2.
    let mut a = [digit(3)];
    assert_eq!(sub_reverse_signed_signed(&mut a, &[digit(5)]), sdigit(0));
    assert_eq!(a, [digit(2)]);

    // 3 - 5 == -2.
    let mut a = [digit(5)];
    assert_eq!(sub_reverse_signed_signed(&mut a, &[digit(3)]), sdigit(-1));
    assert_eq!(a, [Digit::MAX - digit(1)]);

    // -1 - -1 == 0.
    let mut a = [Digit::MAX];
    assert_eq!(sub_reverse_signed_signed(&mut a, &[Digit::MAX]), sdigit(0));
    assert_eq!(a, [Digit::ZERO]);

    // Overflow into the returned digit: 1 - (-2^(bits-1)) == 2^(bits-1) + 1.
    let signed_min = (Digit::MAX >> 1) + digit(1);
    let mut a = [signed_min];
    assert_eq!(sub_reverse_signed_signed(&mut a, &[digit(1)]), sdigit(0));
    assert_eq!(a, [signed_min + digit(1)]);

    // A borrow ripples through the high zero digits: 1 - 2^(2*bits) == -(2^(2*bits) - 1).
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(sub_reverse_signed_signed(&mut a, &[digit(1)]), sdigit(-1));
    assert_eq!(a, [digit(1), Digit::ZERO, Digit::MAX]);
}

#[test]
#[should_panic]
fn sub_reverse_signed_signed_rhs_longer() {
    sub_reverse_signed_signed(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
#[should_panic]
fn sub_reverse_signed_signed_rhs_empty() {
    sub_reverse_signed_signed(&mut [digit(1)], &[]);
}

#[test]
fn sub_reverse_signed_sdigit_basic() {
    // a = d - a == 5 - 3 == 2.
    let mut a = [digit(3)];
    assert_eq!(sub_reverse_signed_sdigit(&mut a, sdigit(5)), sdigit(0));
    assert_eq!(a, [digit(2)]);

    // 3 - 5 == -2.
    let mut a = [digit(5)];
    assert_eq!(sub_reverse_signed_sdigit(&mut a, sdigit(3)), sdigit(-1));
    assert_eq!(a, [Digit::MAX - digit(1)]);
}

#[test]
#[should_panic]
fn sub_reverse_signed_sdigit_empty() {
    sub_reverse_signed_sdigit(&mut [], sdigit(1));
}

proptest! {
    // Subtraction undoes addition: a + b - b == a, and the borrow cancels the carry.
    #[test]
    fn add_sub_unsigned_unsigned_roundtrip(
        a in vec(any::<Digit>(), 0..20),
        b in vec(any::<Digit>(), 0..20),
    ) {
        let (longer, shorter) = if a.len() >= b.len() { (a, b) } else { (b, a) };

        let mut sum = longer.clone();
        let carry = add_unsigned_unsigned(&mut sum, &shorter);
        let borrow = sub_unsigned_unsigned(&mut sum, &shorter);
        prop_assert_eq!(sum, longer);
        prop_assert_eq!(borrow, carry);
    }

    // Signed subtraction undoes signed addition: a + b - b == a. Each operation's returned sign
    // digit is appended so the running value stays exact even when it overflows a digit.
    #[test]
    fn add_sub_signed_signed_roundtrip(
        a in vec(any::<Digit>(), 1..20),
        b in vec(any::<Digit>(), 1..20),
    ) {
        let (longer, shorter) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let n = longer.len();

        let mut value = longer.clone();
        let add_top = add_signed_signed(&mut value, &shorter);
        value.push(add_top.cast_unsigned());
        let sub_top = sub_signed_signed(&mut value, &shorter);
        value.push(sub_top.cast_unsigned());

        // The original value, sign-extended to the grown length.
        let mut expected = longer;
        expected.resize(value.len(), Digit::ZERO);
        extend_signed(&mut expected, n);
        prop_assert_eq!(value, expected);
    }

    // `sub_unsigned_unsigned` with a zero-extended `rhs` agrees with
    // `sub_unsigned_unsigned_same_len`.
    #[test]
    fn sub_unsigned_unsigned_matches_same_len(
        a in vec(any::<Digit>(), 0..20),
        b in vec(any::<Digit>(), 0..20),
    ) {
        let (mut longer, shorter) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let mut padded = shorter.clone();
        padded.resize(longer.len(), Digit::ZERO);

        let mut same_len = longer.clone();
        let borrow = sub_unsigned_unsigned(&mut longer, &shorter);
        let same_len_borrow = sub_unsigned_unsigned_same_len(&mut same_len, &padded);
        prop_assert_eq!(longer, same_len);
        prop_assert_eq!(borrow, same_len_borrow);
    }
    // Subtracting the signed `b` undoes adding it to the unsigned `a`: `a + b - b == a`.
    #[test]
    fn add_sub_unsigned_signed_roundtrip(
        (a, b) in (1usize..20)
            .prop_flat_map(|n| (vec(any::<Digit>(), n), vec(any::<Digit>(), 1..=n))),
    ) {
        let mut value = a.clone();
        let carry_1 = add_unsigned_signed(&mut value, &b);
        let carry_2 = sub_unsigned_signed(&mut value, &b);
        prop_assert_eq!(value, a);
        prop_assert_eq!(carry_1 + carry_2, SignedDigit::ZERO);
    }

    // `sub_reverse_unsigned_unsigned_same_len(a, b)` (a = b - a) matches the forward
    // `sub_unsigned_unsigned_same_len(b, a)` (b = b - a).
    #[test]
    fn sub_reverse_matches_forward(
        (a, b) in (0usize..20)
            .prop_flat_map(|n| (vec(any::<Digit>(), n), vec(any::<Digit>(), n))),
    ) {
        let mut via_reverse = a.clone();
        let reverse_borrow = sub_reverse_unsigned_unsigned_same_len(&mut via_reverse, &b);
        let mut via_forward = b;
        let forward_borrow = sub_unsigned_unsigned_same_len(&mut via_forward, &a);
        prop_assert_eq!(via_reverse, via_forward);
        prop_assert_eq!(reverse_borrow, forward_borrow);
    }

    // `sub_unsigned_digit` agrees with `sub_unsigned_unsigned` of a one-digit slice.
    #[test]
    fn sub_unsigned_digit_matches_unsigned_unsigned(a in vec(any::<Digit>(), 1..20), d: Digit) {
        let mut via_digit = a.clone();
        let mut via_sub = a;
        let borrow_digit = sub_unsigned_digit(&mut via_digit, d);
        let borrow_sub = sub_unsigned_unsigned(&mut via_sub, &[d]);
        prop_assert_eq!(via_digit, via_sub);
        prop_assert_eq!(borrow_digit, borrow_sub);
    }

    // `sub_unsigned_borrow` agrees with `sub_unsigned_digit` of 0 or 1.
    #[test]
    fn sub_unsigned_borrow_matches_unsigned_digit(a in vec(any::<Digit>(), 1..20), borrow: bool) {
        let mut via_borrow = a.clone();
        let mut via_digit = a;
        let borrow_out = sub_unsigned_borrow(&mut via_borrow, borrow);
        let digit_borrow_out = sub_unsigned_digit(&mut via_digit, Digit::from(u8::from(borrow)));
        prop_assert_eq!(via_borrow, via_digit);
        prop_assert_eq!(borrow_out, digit_borrow_out);
    }

    // `sub_signed_signed` agrees with unsigned subtraction of the operands sign-extended one
    // digit past `lhs` (a signed difference always fits there, so the unsigned borrow can be
    // discarded).
    #[test]
    fn sub_signed_signed_matches_extended(
        a in vec(any::<Digit>(), 1..20),
        b in vec(any::<Digit>(), 1..20),
    ) {
        let (mut longer, shorter) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let n = longer.len() + 1;
        let mut x = longer.clone();
        x.resize(n, Digit::ZERO);
        extend_signed(&mut x, longer.len());
        let mut y = shorter.clone();
        y.resize(n, Digit::ZERO);
        extend_signed(&mut y, shorter.len());
        let _ = sub_unsigned_unsigned_same_len(&mut x, &y);

        let top = sub_signed_signed(&mut longer, &shorter);
        longer.push(top.cast_unsigned());
        prop_assert_eq!(longer, x);
    }

    // `sub_signed_sdigit` agrees with `sub_signed_signed` of a one-digit slice.
    #[test]
    fn sub_signed_sdigit_matches_signed_signed(a in vec(any::<Digit>(), 1..20), d: SignedDigit) {
        let mut via_digit = a.clone();
        let mut via_slice = a;
        let high_digit = sub_signed_sdigit(&mut via_digit, d);
        let high_slice = sub_signed_signed(&mut via_slice, &[d.cast_unsigned()]);
        prop_assert_eq!(via_digit, via_slice);
        prop_assert_eq!(high_digit, high_slice);
    }

    // `sub_unsigned_sdigit` agrees with `sub_unsigned_signed` of a one-digit slice.
    #[test]
    fn sub_unsigned_sdigit_matches_unsigned_signed(a in vec(any::<Digit>(), 1..20), d: SignedDigit) {
        let mut via_digit = a.clone();
        let mut via_slice = a;
        let high_digit = sub_unsigned_sdigit(&mut via_digit, d);
        let high_slice = sub_unsigned_signed(&mut via_slice, &[d.cast_unsigned()]);
        prop_assert_eq!(via_digit, via_slice);
        prop_assert_eq!(high_digit, high_slice);
    }

    // `sub_reverse_signed_signed(a, b)` (a = b - a) matches the forward `sub_signed_signed`
    // with `b` sign-extended to `a`'s length (so the operands can be swapped).
    #[test]
    fn sub_reverse_signed_signed_matches_forward(
        a in vec(any::<Digit>(), 1..20),
        b in vec(any::<Digit>(), 1..20),
    ) {
        let (longer, shorter) = if a.len() >= b.len() { (a, b) } else { (b, a) };

        // Reverse: `longer = shorter - longer`.
        let mut via_reverse = longer.clone();
        let rev_top = sub_reverse_signed_signed(&mut via_reverse, &shorter);
        via_reverse.push(rev_top.cast_unsigned());

        // Forward with `shorter` sign-extended to `longer`'s length.
        let mut shorter_ext = shorter.clone();
        shorter_ext.resize(longer.len(), Digit::ZERO);
        extend_signed(&mut shorter_ext, shorter.len());
        let fwd_top = sub_signed_signed(&mut shorter_ext, &longer);
        shorter_ext.push(fwd_top.cast_unsigned());

        prop_assert_eq!(via_reverse, shorter_ext);
    }

    // `sub_reverse_signed_sdigit` agrees with `sub_reverse_signed_signed` of a one-digit slice.
    #[test]
    fn sub_reverse_signed_sdigit_matches_signed_signed(
        a in vec(any::<Digit>(), 1..20),
        d: SignedDigit,
    ) {
        let mut via_digit = a.clone();
        let mut via_slice = a;
        let high_digit = sub_reverse_signed_sdigit(&mut via_digit, d);
        let high_slice = sub_reverse_signed_signed(&mut via_slice, &[d.cast_unsigned()]);
        prop_assert_eq!(via_digit, via_slice);
        prop_assert_eq!(high_digit, high_slice);
    }

    // `a - b == a + (-b)`: subtraction equals adding the negation. `a` and `b` share a length,
    // and both sides are taken to `a.len() + 2` digits so every result fits exactly.
    #[test]
    fn sub_equals_add_neg(
        (a, b) in (1usize..20)
            .prop_flat_map(|len| (vec(any::<Digit>(), len), vec(any::<Digit>(), len))),
    ) {
        // a - b, sign-extended to a.len() + 2 digits.
        let mut diff = a.clone();
        let top = sub_signed_signed(&mut diff, &b);
        diff.push(top.cast_unsigned());
        diff.push(sign_extension(&diff).cast_unsigned());

        // -b, exact in b.len() + 1 digits.
        let mut neg_b = b;
        let neg_top = neg(&mut neg_b);
        neg_b.push(neg_top.cast_unsigned());

        // a + (-b), with `a` sign-extended to the same length as `neg_b`.
        let mut sum = a;
        sum.push(sign_extension(&sum).cast_unsigned());
        let sum_top = add_signed_signed(&mut sum, &neg_b);
        sum.push(sum_top.cast_unsigned());

        prop_assert_eq!(diff, sum);
    }
}

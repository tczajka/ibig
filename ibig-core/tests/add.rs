//! Integration tests for addition.

use ibig_core::{
    Digit, IDigit, add_digit_idigit, add_idigit_idigit, add_signed_digit, add_signed_idigit,
    add_signed_signed, add_signed_unsigned, add_unsigned_1, add_unsigned_carry, add_unsigned_digit,
    add_unsigned_icarry, add_unsigned_idigit, add_unsigned_signed, add_unsigned_unsigned,
    add_unsigned_unsigned_same_len, extend_signed,
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
fn add_unsigned_unsigned_basic() {
    let mut a = [digit(1), digit(2), digit(3)];
    assert!(!add_unsigned_unsigned(&mut a, &[digit(3), digit(7)]));
    assert_eq!(a, [digit(4), digit(9), digit(3)]);

    // A carry out of the low digits propagates through the high digits.
    let mut a = [Digit::MAX, Digit::MAX, digit(3)];
    assert!(!add_unsigned_unsigned(&mut a, &[digit(1)]));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO, digit(4)]);

    // A carry out of the most-significant digit.
    let mut a = [Digit::MAX, Digit::MAX];
    assert!(add_unsigned_unsigned(&mut a, &[digit(1), Digit::ZERO]));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO]);

    // An empty `rhs` is allowed.
    let mut a = [digit(5)];
    assert!(!add_unsigned_unsigned(&mut a, &[]));
    assert_eq!(a, [digit(5)]);
}

#[test]
#[should_panic]
fn add_unsigned_unsigned_rhs_longer() {
    add_unsigned_unsigned(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
fn add_unsigned_unsigned_same_len_basic() {
    let mut a = [digit(1), digit(2)];
    assert!(!add_unsigned_unsigned_same_len(
        &mut a,
        &[digit(3), digit(4)]
    ));
    assert_eq!(a, [digit(4), digit(6)]);

    // The carry propagates across digits and out the top.
    let mut a = [Digit::MAX, Digit::MAX];
    assert!(add_unsigned_unsigned_same_len(
        &mut a,
        &[digit(1), Digit::ZERO]
    ));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO]);

    // Empty slices are allowed.
    assert!(!add_unsigned_unsigned_same_len(&mut [], &[]));
}

#[test]
#[should_panic]
fn add_unsigned_unsigned_same_len_mismatched() {
    add_unsigned_unsigned_same_len(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
fn add_unsigned_digit_basic() {
    let mut a = [digit(1), digit(2)];
    assert!(!add_unsigned_digit(&mut a, digit(7)));
    assert_eq!(a, [digit(8), digit(2)]);

    let mut a = [Digit::MAX, digit(3)];
    assert!(!add_unsigned_digit(&mut a, digit(1)));
    assert_eq!(a, [Digit::ZERO, digit(4)]);

    let mut a = [Digit::MAX, Digit::MAX];
    assert!(add_unsigned_digit(&mut a, digit(1)));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO]);
}

#[test]
#[should_panic]
fn add_unsigned_digit_empty() {
    add_unsigned_digit(&mut [], digit(1));
}

#[test]
fn add_unsigned_carry_basic() {
    let mut a = [Digit::MAX, digit(0)];
    assert!(!add_unsigned_carry(&mut a, true));
    assert_eq!(a, [Digit::ZERO, digit(1)]);

    // Without an incoming carry nothing changes.
    let mut a = [Digit::MAX];
    assert!(!add_unsigned_carry(&mut a, false));
    assert_eq!(a, [Digit::MAX]);

    // A carry out of the most-significant digit.
    let mut a = [Digit::MAX, Digit::MAX];
    assert!(add_unsigned_carry(&mut a, true));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO]);

    // An empty slice passes the carry through.
    assert!(add_unsigned_carry(&mut [], true));
    assert!(!add_unsigned_carry(&mut [], false));
}

#[test]
fn add_unsigned_1_basic() {
    let mut a = [digit(1), digit(2)];
    assert!(!add_unsigned_1(&mut a));
    assert_eq!(a, [digit(2), digit(2)]);

    // The increment ripples through all-ones digits.
    let mut a = [Digit::MAX, Digit::MAX, digit(3)];
    assert!(!add_unsigned_1(&mut a));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO, digit(4)]);

    // All ones overflows.
    let mut a = [Digit::MAX, Digit::MAX];
    assert!(add_unsigned_1(&mut a));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO]);

    // An empty slice overflows immediately.
    assert!(add_unsigned_1(&mut []));
}

#[test]
fn add_unsigned_icarry_basic() {
    // Carry 0 leaves the slice unchanged.
    let mut a = [digit(7), digit(2)];
    assert_eq!(add_unsigned_icarry(&mut a, idigit(0)), idigit(0));
    assert_eq!(a, [digit(7), digit(2)]);

    // Carry +1 adds 1; here it ripples out the top.
    let mut a = [Digit::MAX, Digit::MAX];
    assert_eq!(add_unsigned_icarry(&mut a, idigit(1)), idigit(1));
    assert_eq!(a, [Digit::ZERO, Digit::ZERO]);

    // Carry -1 subtracts 1; here it borrows out the top.
    let mut a = [Digit::ZERO, Digit::ZERO];
    assert_eq!(add_unsigned_icarry(&mut a, idigit(-1)), idigit(-1));
    assert_eq!(a, [Digit::MAX, Digit::MAX]);

    // No carry/borrow out the top.
    let mut a = [Digit::ZERO, digit(1)];
    assert_eq!(add_unsigned_icarry(&mut a, idigit(-1)), idigit(0));
    assert_eq!(a, [Digit::MAX, digit(0)]);

    // An empty slice passes the carry straight through.
    assert_eq!(add_unsigned_icarry(&mut [], idigit(1)), idigit(1));
    assert_eq!(add_unsigned_icarry(&mut [], idigit(-1)), idigit(-1));
    assert_eq!(add_unsigned_icarry(&mut [], idigit(0)), idigit(0));
}

#[test]
#[should_panic]
fn add_unsigned_icarry_out_of_range() {
    add_unsigned_icarry(&mut [digit(1)], idigit(2));
}

#[test]
fn add_unsigned_signed_basic() {
    // Two non-negative values: 5 + 3 == 8.
    let mut a = [digit(5), digit(2)];
    assert_eq!(add_unsigned_signed(&mut a, &[digit(3)]), idigit(0));
    assert_eq!(a, [digit(8), digit(2)]);

    // A negative `rhs` reduces `lhs`: 5 + -1 == 4.
    let mut a = [digit(5)];
    assert_eq!(add_unsigned_signed(&mut a, &[Digit::MAX]), idigit(0));
    assert_eq!(a, [digit(4)]);

    // A negative result: 2 + -3 == -1, carry -1.
    let mut a = [digit(2)];
    assert_eq!(
        add_unsigned_signed(&mut a, &[Digit::MAX - digit(2)]),
        idigit(-1)
    );
    assert_eq!(a, [Digit::MAX]);

    // An overflow: (2^bits - 1) + 1 == 2^bits, carry +1.
    let mut a = [Digit::MAX];
    assert_eq!(add_unsigned_signed(&mut a, &[digit(1)]), idigit(1));
    assert_eq!(a, [Digit::ZERO]);

    // A negative `rhs` sign-extends and borrows through the high digits: 2^(2*bits) + -1.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(add_unsigned_signed(&mut a, &[Digit::MAX]), idigit(0));
    assert_eq!(a, [Digit::MAX, Digit::MAX, Digit::ZERO]);
}

#[test]
#[should_panic]
fn add_unsigned_signed_rhs_longer() {
    add_unsigned_signed(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
#[should_panic]
fn add_unsigned_signed_rhs_empty() {
    add_unsigned_signed(&mut [digit(1)], &[]);
}

#[test]
fn add_signed_signed_basic() {
    // Two non-negative values.
    let mut a = [digit(2), digit(3)];
    assert_eq!(add_signed_signed(&mut a, &[digit(5)]), idigit(0));
    assert_eq!(a, [digit(7), digit(3)]);

    // -1 + -1 == -2.
    let mut a = [Digit::MAX];
    assert_eq!(add_signed_signed(&mut a, &[Digit::MAX]), idigit(-1));
    assert_eq!(a, [Digit::MAX - digit(1)]);

    // A positive sum that overflows into the returned digit.
    let signed_max = Digit::MAX >> 1;
    let mut a = [signed_max];
    assert_eq!(add_signed_signed(&mut a, &[digit(1)]), idigit(0));
    assert_eq!(a, [signed_max + digit(1)]);

    // A negative sum that overflows into the returned digit: -2^(bits-1) + -2^(bits-1).
    let signed_min = signed_max + digit(1);
    let mut a = [signed_min];
    assert_eq!(add_signed_signed(&mut a, &[signed_min]), idigit(-1));
    assert_eq!(a, [Digit::ZERO]);

    // A negative `rhs` sign-extends across the high digits of `lhs` and borrows through
    // all-zeros digits: 2^(2*bits) + -1.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(add_signed_signed(&mut a, &[Digit::MAX]), idigit(0));
    assert_eq!(a, [Digit::MAX, Digit::MAX, Digit::ZERO]);

    // Mixed signs without overflow: 5 + -3 == 2.
    let mut a = [digit(5)];
    assert_eq!(
        add_signed_signed(&mut a, &[Digit::MAX - digit(2)]),
        idigit(0)
    );
    assert_eq!(a, [digit(2)]);
}

#[test]
fn add_signed_idigit_basic() {
    // Two non-negative values.
    let mut a = [digit(2), digit(3)];
    assert_eq!(add_signed_idigit(&mut a, idigit(5)), idigit(0));
    assert_eq!(a, [digit(7), digit(3)]);

    // -1 + -1 == -2.
    let mut a = [Digit::MAX];
    assert_eq!(add_signed_idigit(&mut a, idigit(-1)), idigit(-1));
    assert_eq!(a, [Digit::MAX - digit(1)]);

    // The carry propagates into the high digits.
    let mut a = [Digit::MAX, Digit::ZERO];
    assert_eq!(add_signed_idigit(&mut a, idigit(1)), idigit(0));
    assert_eq!(a, [Digit::ZERO, digit(1)]);

    // A negative digit borrows through all-zeros digits: 2^(2*bits) + -1.
    let mut a = [Digit::ZERO, Digit::ZERO, digit(1)];
    assert_eq!(add_signed_idigit(&mut a, idigit(-1)), idigit(0));
    assert_eq!(a, [Digit::MAX, Digit::MAX, Digit::ZERO]);
}

#[test]
#[should_panic]
fn add_signed_idigit_empty() {
    add_signed_idigit(&mut [], idigit(1));
}

#[test]
#[should_panic]
fn add_signed_signed_rhs_longer() {
    add_signed_signed(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
#[should_panic]
fn add_signed_signed_rhs_empty() {
    add_signed_signed(&mut [digit(1)], &[]);
}

#[test]
fn add_unsigned_idigit_basic() {
    // 5 + -3 == 2.
    let mut a = [digit(5)];
    assert_eq!(add_unsigned_idigit(&mut a, idigit(-3)), idigit(0));
    assert_eq!(a, [digit(2)]);

    // A positive carry into the high digits.
    let mut a = [Digit::MAX, digit(1)];
    assert_eq!(add_unsigned_idigit(&mut a, idigit(1)), idigit(0));
    assert_eq!(a, [Digit::ZERO, digit(2)]);

    // A negative result: 0 + -1 == -1.
    let mut a = [Digit::ZERO];
    assert_eq!(add_unsigned_idigit(&mut a, idigit(-1)), idigit(-1));
    assert_eq!(a, [Digit::MAX]);

    // A positive overflow into a +1 carry: MAX + 1 == 2^bits.
    let mut a = [Digit::MAX];
    assert_eq!(add_unsigned_idigit(&mut a, idigit(1)), idigit(1));
    assert_eq!(a, [Digit::ZERO]);
}

#[test]
#[should_panic]
fn add_unsigned_idigit_empty() {
    add_unsigned_idigit(&mut [], idigit(1));
}

#[test]
fn add_digit_idigit_basic() {
    // 5 + -3 == 2, no carry.
    assert_eq!(
        add_digit_idigit(digit(5), idigit(-3)),
        (digit(2), idigit(0))
    );

    // 0 + -1 == -1: the low digit wraps and the carry is -1.
    assert_eq!(
        add_digit_idigit(Digit::ZERO, idigit(-1)),
        (Digit::MAX, idigit(-1))
    );

    // A positive overflow into a +1 carry: MAX + 1 == 2^bits.
    assert_eq!(
        add_digit_idigit(Digit::MAX, idigit(1)),
        (Digit::ZERO, idigit(1))
    );

    // The largest positive idigit added to MAX still carries only +1.
    assert_eq!(
        add_digit_idigit(Digit::MAX, IDigit::MAX),
        (IDigit::MAX.cast_unsigned() - digit(1), idigit(1))
    );
}

#[test]
fn add_idigit_idigit_basic() {
    // 2 + 3 == 5, no carry.
    assert_eq!(
        add_idigit_idigit(idigit(2), idigit(3)),
        (digit(5), idigit(0))
    );

    // 1 + -1 == 0.
    assert_eq!(
        add_idigit_idigit(idigit(1), idigit(-1)),
        (Digit::ZERO, idigit(0))
    );

    // -1 + -1 == -2: the low digit wraps and the carry is -1.
    assert_eq!(
        add_idigit_idigit(idigit(-1), idigit(-1)),
        (Digit::MAX - digit(1), idigit(-1))
    );

    // Two large positives overflow the signed range but stay non-negative: carry 0.
    assert_eq!(
        add_idigit_idigit(IDigit::MAX, IDigit::MAX),
        (Digit::MAX - digit(1), idigit(0))
    );

    // The two most-negative digits sum to -2^bits: low digit 0, carry -1.
    assert_eq!(
        add_idigit_idigit(IDigit::MIN, IDigit::MIN),
        (Digit::ZERO, idigit(-1))
    );
}

#[test]
fn add_signed_digit_basic() {
    // -1 + 5 == 4.
    let mut a = [Digit::MAX];
    assert_eq!(add_signed_digit(&mut a, digit(5)), idigit(0));
    assert_eq!(a, [digit(4)]);

    // A negative result stays negative: -2 + 1 == -1.
    let mut a = [Digit::MAX - digit(1)];
    assert_eq!(add_signed_digit(&mut a, digit(1)), idigit(-1));
    assert_eq!(a, [Digit::MAX]);

    // A positive overflow into a +1 digit: largest positive single digit + MAX.
    let signed_max = Digit::MAX >> 1;
    let mut a = [signed_max];
    assert_eq!(add_signed_digit(&mut a, Digit::MAX), idigit(1));
    assert_eq!(a, [signed_max - digit(1)]);
}

#[test]
#[should_panic]
fn add_signed_digit_empty() {
    add_signed_digit(&mut [], digit(1));
}

#[test]
fn add_signed_unsigned_basic() {
    // Two non-negative values: 3 + 5 == 8.
    let mut a = [digit(3), digit(2)];
    assert_eq!(add_signed_unsigned(&mut a, &[digit(5)]), idigit(0));
    assert_eq!(a, [digit(8), digit(2)]);

    // A negative `lhs`: -1 + 5 == 4.
    let mut a = [Digit::MAX];
    assert_eq!(add_signed_unsigned(&mut a, &[digit(5)]), idigit(0));
    assert_eq!(a, [digit(4)]);

    // A negative result: -2 + 1 == -1, sign digit -1.
    let mut a = [Digit::MAX - digit(1)];
    assert_eq!(add_signed_unsigned(&mut a, &[digit(1)]), idigit(-1));
    assert_eq!(a, [Digit::MAX]);

    // A positive overflow into a +1 digit: the largest positive single digit plus all-ones.
    let signed_max = Digit::MAX >> 1;
    let mut a = [signed_max];
    assert_eq!(add_signed_unsigned(&mut a, &[Digit::MAX]), idigit(1));
    assert_eq!(a, [signed_max - digit(1)]);

    // An empty `rhs` is a no-op returning `lhs`'s sign.
    let mut a = [Digit::MAX]; // -1
    assert_eq!(add_signed_unsigned(&mut a, &[]), idigit(-1));
    assert_eq!(a, [Digit::MAX]);
}

#[test]
#[should_panic]
fn add_signed_unsigned_rhs_longer() {
    add_signed_unsigned(&mut [digit(1)], &[digit(1), digit(2)]);
}

#[test]
#[should_panic]
fn add_signed_unsigned_lhs_empty() {
    add_signed_unsigned(&mut [], &[]);
}

proptest! {
    // `add_unsigned_unsigned` with a zero-extended `rhs` agrees with
    // `add_unsigned_unsigned_same_len`.
    #[test]
    fn add_unsigned_unsigned_matches_same_len(
        a in vec(any::<Digit>(), 0..20),
        b in vec(any::<Digit>(), 0..20),
    ) {
        let (mut longer, shorter) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let mut padded = shorter.clone();
        padded.resize(longer.len(), Digit::ZERO);

        let mut same_len = longer.clone();
        let carry = add_unsigned_unsigned(&mut longer, &shorter);
        let same_len_carry = add_unsigned_unsigned_same_len(&mut same_len, &padded);
        prop_assert_eq!(longer, same_len);
        prop_assert_eq!(carry, same_len_carry);
    }

    // Addition of zero-padded equal-length slices is commutative.
    #[test]
    fn add_unsigned_unsigned_same_len_commutes(
        a in vec(any::<Digit>(), 0..20),
        b in vec(any::<Digit>(), 0..20),
    ) {
        let n = a.len().max(b.len());
        let mut x = a;
        x.resize(n, Digit::ZERO);
        let mut y = b;
        y.resize(n, Digit::ZERO);

        let mut x_plus_y = x.clone();
        let carry_xy = add_unsigned_unsigned_same_len(&mut x_plus_y, &y);
        let mut y_plus_x = y.clone();
        let carry_yx = add_unsigned_unsigned_same_len(&mut y_plus_x, &x);
        prop_assert_eq!(x_plus_y, y_plus_x);
        prop_assert_eq!(carry_xy, carry_yx);
    }

    // `add_unsigned_digit` agrees with `add_unsigned_unsigned` of a one-digit slice.
    #[test]
    fn add_unsigned_digit_matches_unsigned_unsigned(a in vec(any::<Digit>(), 1..20), d: Digit) {
        let mut via_digit = a.clone();
        let mut via_add = a;
        let carry_digit = add_unsigned_digit(&mut via_digit, d);
        let carry_add = add_unsigned_unsigned(&mut via_add, &[d]);
        prop_assert_eq!(via_digit, via_add);
        prop_assert_eq!(carry_digit, carry_add);
    }

    // `add_unsigned_carry` agrees with `add_unsigned_digit` of 0 or 1.
    #[test]
    fn add_unsigned_carry_matches_unsigned_digit(a in vec(any::<Digit>(), 1..20), carry: bool) {
        let mut via_carry = a.clone();
        let mut via_digit = a;
        let carry_out = add_unsigned_carry(&mut via_carry, carry);
        let digit_carry_out = add_unsigned_digit(&mut via_digit, Digit::from(u8::from(carry)));
        prop_assert_eq!(via_carry, via_digit);
        prop_assert_eq!(carry_out, digit_carry_out);
    }

    // `add_signed_signed` agrees with unsigned addition of the operands sign-extended one digit
    // past `lhs` (a signed sum always fits there, so the unsigned carry can be discarded).
    #[test]
    fn add_signed_signed_matches_extended(
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
        let _ = add_unsigned_unsigned_same_len(&mut x, &y);

        let top = add_signed_signed(&mut longer, &shorter);
        longer.push(top.cast_unsigned());
        prop_assert_eq!(longer, x);
    }

    // `add_signed_idigit` agrees with `add_signed_signed` of a one-digit slice.
    #[test]
    fn add_signed_idigit_matches_signed_signed(a in vec(any::<Digit>(), 1..20), d: IDigit) {
        let mut via_digit = a.clone();
        let mut via_slice = a;
        let high_digit = add_signed_idigit(&mut via_digit, d);
        let high_slice = add_signed_signed(&mut via_slice, &[d.cast_unsigned()]);
        prop_assert_eq!(via_digit, via_slice);
        prop_assert_eq!(high_digit, high_slice);
    }

    // `add_unsigned_idigit` agrees with `add_unsigned_signed` of a one-digit slice.
    #[test]
    fn add_unsigned_idigit_matches_unsigned_signed(a in vec(any::<Digit>(), 1..20), d: IDigit) {
        let mut via_digit = a.clone();
        let mut via_slice = a;
        let high_digit = add_unsigned_idigit(&mut via_digit, d);
        let high_slice = add_unsigned_signed(&mut via_slice, &[d.cast_unsigned()]);
        prop_assert_eq!(via_digit, via_slice);
        prop_assert_eq!(high_digit, high_slice);
    }

    // `add_digit_idigit(a, d)` agrees with `add_unsigned_idigit` on the one-digit slice `[a]`.
    #[test]
    fn add_digit_idigit_matches_unsigned_idigit(a: Digit, d: IDigit) {
        let (low, carry) = add_digit_idigit(a, d);
        let mut slice = [a];
        let slice_carry = add_unsigned_idigit(&mut slice, d);
        prop_assert_eq!(low, slice[0]);
        prop_assert_eq!(carry, slice_carry);
    }

    // `add_idigit_idigit(a, d)` agrees with `add_signed_signed` on the one-digit slices `[a]`, `[d]`.
    #[test]
    fn add_idigit_idigit_matches_signed_signed(a: IDigit, d: IDigit) {
        let (low, carry) = add_idigit_idigit(a, d);
        let mut slice = [a.cast_unsigned()];
        let slice_carry = add_signed_signed(&mut slice, &[d.cast_unsigned()]);
        prop_assert_eq!(low, slice[0]);
        prop_assert_eq!(carry, slice_carry);
    }

    // `add_signed_digit` agrees with `add_signed_unsigned` of a one-digit slice.
    #[test]
    fn add_signed_digit_matches_signed_unsigned(a in vec(any::<Digit>(), 1..20), d: Digit) {
        let mut via_digit = a.clone();
        let mut via_slice = a;
        let high_digit = add_signed_digit(&mut via_digit, d);
        let high_slice = add_signed_unsigned(&mut via_slice, &[d]);
        prop_assert_eq!(via_digit, via_slice);
        prop_assert_eq!(high_digit, high_slice);
    }

    // `add_signed_unsigned(a, b)` (a signed, b unsigned) equals `add_unsigned_signed(b, a)`
    // (the same sum, commuted). `a` and `b` share a length so both length preconditions hold.
    #[test]
    fn add_signed_unsigned_commutes(
        (a, b) in (1usize..20).prop_flat_map(|n| (vec(any::<Digit>(), n), vec(any::<Digit>(), n))),
    ) {
        let mut via_signed = a.clone();
        let top = add_signed_unsigned(&mut via_signed, &b);
        via_signed.push(top.cast_unsigned());

        let mut via_unsigned = b;
        let carry = add_unsigned_signed(&mut via_unsigned, &a);
        via_unsigned.push(carry.cast_unsigned());

        prop_assert_eq!(via_signed, via_unsigned);
    }
}

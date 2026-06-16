//! Integration tests for `UBig` and `IBig` bitwise operators.

use ibig::proptest::{ibig_up_to_bits, ubig_up_to_bits};
use ibig::{IBig, UBig};
use proptest::prelude::*;

// Checks `big(a) <op> big(b) == big((a as wide) <op> (b as wide))` over random primitives `a:
// $a` and `b: $b`, where `$wide` is a primitive wide enough to hold both. Letting the operands
// have different widths exercises the single-/multi-digit and unequal-length code paths.
macro_rules! binop_vs_primitive {
    ($name:ident, $big:ty, $op:tt, $op_assign:tt, $a:ty, $b:ty, $wide:ty) => {
        proptest! {
            #[test]
            // `as $wide` is the identity for the same-width operands.
            #[allow(clippy::unnecessary_cast)]
            fn $name(a: $a, b: $b) {
                let big_a = <$big>::from(a);
                let big_b = <$big>::from(b);
                let expected = <$big>::from(<$wide>::from(a) $op <$wide>::from(b));
                prop_assert_eq!(big_a $op big_b, expected);
            }
        }
    };
}

// AND vs primitive, across operand widths.
binop_vs_primitive!(ubig_bitand_16_16, UBig, &, &=, u16, u16, u128);
binop_vs_primitive!(ubig_bitand_16_128, UBig, &, &=, u16, u128, u128);
binop_vs_primitive!(ubig_bitand_128_16, UBig, &, &=, u128, u16, u128);
binop_vs_primitive!(ubig_bitand_128_128, UBig, &, &=, u128, u128, u128);
binop_vs_primitive!(ibig_bitand_16_16, IBig, &, &=, i16, i16, i128);
binop_vs_primitive!(ibig_bitand_16_128, IBig, &, &=, i16, i128, i128);
binop_vs_primitive!(ibig_bitand_128_16, IBig, &, &=, i128, i16, i128);
binop_vs_primitive!(ibig_bitand_128_128, IBig, &, &=, i128, i128, i128);

// OR vs primitive, across operand widths.
binop_vs_primitive!(ubig_bitor_16_16, UBig, |, |=, u16, u16, u128);
binop_vs_primitive!(ubig_bitor_16_128, UBig, |, |=, u16, u128, u128);
binop_vs_primitive!(ubig_bitor_128_16, UBig, |, |=, u128, u16, u128);
binop_vs_primitive!(ubig_bitor_128_128, UBig, |, |=, u128, u128, u128);
binop_vs_primitive!(ibig_bitor_16_16, IBig, |, |=, i16, i16, i128);
binop_vs_primitive!(ibig_bitor_16_128, IBig, |, |=, i16, i128, i128);
binop_vs_primitive!(ibig_bitor_128_16, IBig, |, |=, i128, i16, i128);
binop_vs_primitive!(ibig_bitor_128_128, IBig, |, |=, i128, i128, i128);

// XOR vs primitive, across operand widths.
binop_vs_primitive!(ubig_bitxor_16_16, UBig, ^, ^=, u16, u16, u128);
binop_vs_primitive!(ubig_bitxor_16_128, UBig, ^, ^=, u16, u128, u128);
binop_vs_primitive!(ubig_bitxor_128_16, UBig, ^, ^=, u128, u16, u128);
binop_vs_primitive!(ubig_bitxor_128_128, UBig, ^, ^=, u128, u128, u128);
binop_vs_primitive!(ibig_bitxor_16_16, IBig, ^, ^=, i16, i16, i128);
binop_vs_primitive!(ibig_bitxor_16_128, IBig, ^, ^=, i16, i128, i128);
binop_vs_primitive!(ibig_bitxor_128_16, IBig, ^, ^=, i128, i16, i128);
binop_vs_primitive!(ibig_bitxor_128_128, IBig, ^, ^=, i128, i128, i128);

proptest! {
    // Bitwise NOT vs the primitive: `!IBig::from(v) == IBig::from(!v)`.
    #[test]
    fn ibig_not_vs_i16(v: i16) {
        let expected = IBig::from(!v);
        prop_assert_eq!(&!IBig::from(v), &expected);
    }

    #[test]
    fn ibig_not_vs_i128(v: i128) {
        let expected = IBig::from(!v);
        prop_assert_eq!(&!IBig::from(v), &expected);
    }

    // Bitwise NOT is an involution: `!!x == x`.
    #[test]
    fn ibig_not_not(x in ibig_up_to_bits(1000)) {
        prop_assert_eq!(!!&x, x);
    }

    // De Morgan over arbitrary sizes: `a | b == !(!a & !b)`.
    #[test]
    fn ibig_de_morgan(a in ibig_up_to_bits(1000), b in ibig_up_to_bits(1000)) {
        prop_assert_eq!(&a | &b, !(!&a & !&b));
    }

    // XOR is self-inverse: `a ^ b ^ b == a`.
    #[test]
    fn ubig_xor_inverse(a in ubig_up_to_bits(1000), b in ubig_up_to_bits(1000)) {
        prop_assert_eq!((&a ^ &b) ^ &b, a);
    }

    #[test]
    fn ibig_xor_inverse(a in ibig_up_to_bits(1000), b in ibig_up_to_bits(1000)) {
        prop_assert_eq!((&a ^ &b) ^ &b, a);
    }

    // A value XORed with itself is zero (exercises high-digit cancellation).
    #[test]
    fn ubig_xor_self(a in ubig_up_to_bits(1000)) {
        prop_assert_eq!(&a ^ &a, UBig::ZERO);
    }

    #[test]
    fn ibig_xor_self(a in ibig_up_to_bits(1000)) {
        prop_assert_eq!(&a ^ &a, IBig::ZERO);
    }

    // XOR in terms of AND, OR and NOT: `a ^ b == (a & !b) | (!a & b)`.
    #[test]
    fn ibig_xor_identity(a in ibig_up_to_bits(1000), b in ibig_up_to_bits(1000)) {
        let expected = (&a & !&b) | (!&a & &b);
        prop_assert_eq!(&a ^ &b, expected);
    }

    // `UBig::bit_and_not(a, b) == a & !b`, for every pair of `u16`/`u128` operand widths.
    #[test]
    fn ubig_bit_and_not_16_16(a: u16, b: u16) {
        prop_assert_eq!(UBig::from(a).bit_and_not(&UBig::from(b)), UBig::from(a & !b));
    }

    #[test]
    fn ubig_bit_and_not_16_128(a: u16, b: u128) {
        prop_assert_eq!(UBig::from(a).bit_and_not(&UBig::from(b)), UBig::from(u128::from(a) & !b));
    }

    #[test]
    fn ubig_bit_and_not_128_16(a: u128, b: u16) {
        prop_assert_eq!(UBig::from(a).bit_and_not(&UBig::from(b)), UBig::from(a & !u128::from(b)));
    }

    #[test]
    fn ubig_bit_and_not_128_128(a: u128, b: u128) {
        prop_assert_eq!(UBig::from(a).bit_and_not(&UBig::from(b)), UBig::from(a & !b));
    }

    // `UBig::bit_and_not` agrees with the signed `a & !b`.
    #[test]
    fn ubig_bit_and_not_vs_signed(a in ubig_up_to_bits(1000), b in ubig_up_to_bits(1000)) {
        prop_assert_eq!(IBig::from(&a.bit_and_not(&b)), IBig::from(&a) & !IBig::from(&b));
    }
}

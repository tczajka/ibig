//! Internal traits describing unary and binary operations.

use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
    Digits,
};
use ibig_core::Digit;

/// A unary operation.
///
/// The operand can be borrowed or owned.
pub(crate) trait UnaryOp {
    /// The type of the operand.
    type Operand;

    /// The type of the result.
    type Output;

    /// The operand is borrowed.
    fn apply_ref(value: &Self::Operand) -> Self::Output;

    /// The operand is owned.
    fn apply_val(value: Self::Operand) -> Self::Output;
}

/// A unary operation implemented on a big number.
///
/// The operand appears in one of three forms: a single digit (`digit`), a borrowed slice
/// (`ref`), or an owned buffer (`val`).
pub(crate) trait UnaryOpBig {
    /// The type of the operand.
    type Operand: AsDigits;

    /// The type of the result.
    type Output;

    /// The operand is a single digit.
    fn apply_digit(operand: <Self::Operand as AsDigits>::SingleDigit) -> Self::Output;

    /// The operand is a borrowed slice.
    fn apply_ref(operand: &[Digit]) -> Self::Output;

    /// The operand is an owned buffer.
    fn apply_val(operand: Digits) -> Self::Output;
}

/// Every [`UnaryOpBig`] induces a [`UnaryOp`].
impl<Op: UnaryOpBig> UnaryOp for Op {
    type Operand = Op::Operand;
    type Output = Op::Output;

    fn apply_ref(value: &Self::Operand) -> Self::Output {
        match value.as_digits() {
            Small(d) => <Op as UnaryOpBig>::apply_digit(d),
            Large(digits) => <Op as UnaryOpBig>::apply_ref(digits),
        }
    }

    fn apply_val(value: Self::Operand) -> Self::Output {
        match value.into_digits() {
            Small(d) => <Op as UnaryOpBig>::apply_digit(d),
            Large(digits) => <Op as UnaryOpBig>::apply_val(digits),
        }
    }
}

/// Implements a unary operator for a value type `$t`, deriving the owned and borrowed forms from
/// an [`UnaryOp`] implemented by the marker type `$op`.
///
/// `$trait`/`$method` is the operator trait; it must be in scope at the call site.
macro_rules! impl_unary_operator {
    ($trait:ident :: $method:ident($operand:ty) -> $output:ty, $op:ty) => {
        impl $trait for $operand {
            type Output = $output;

            fn $method(self) -> Self::Output {
                <$op as $crate::ops::UnaryOp>::apply_val(self)
            }
        }

        impl $trait for &$operand {
            type Output = $output;

            fn $method(self) -> Self::Output {
                <$op as $crate::ops::UnaryOp>::apply_ref(self)
            }
        }
    };
}

pub(crate) use impl_unary_operator;

/// A binary operation taking a left operand of type `L` and a right operand of type `R`,
/// producing a value of type `L`.
///
/// Each operand appears can be borrowed or owned.
pub(crate) trait BinaryOp<L: Default, R> {
    /// Both operands are borrowed.
    fn apply_ref_ref(lhs: &L, rhs: &R) -> L;

    /// Left operand is borrowed, right operand owned.
    fn apply_ref_val(lhs: &L, rhs: R) -> L;

    /// Left operand is owned, right operand borrowed.
    fn apply_val_ref(lhs: L, rhs: &R) -> L;

    /// Both operands are owned.
    fn apply_val_val(lhs: L, rhs: R) -> L;
}

/// A binary operation implemented on the digit representation of a number.
///
/// Each operand appears in one of three forms: a single digit (`digit`), a borrowed slice
/// (`ref`), or an owned buffer (`val`).
pub(crate) trait BinaryOpDigits<T: AsDigits> {
    /// Both operands are single digits.
    fn apply_digit_digit(lhs: T::SingleDigit, rhs: T::SingleDigit) -> T;

    /// Left operand is a single digit, right operand a borrowed slice.
    fn apply_digit_ref(lhs: T::SingleDigit, rhs: &[Digit]) -> T;

    /// Left operand is a single digit, right operand an owned buffer.
    fn apply_digit_val(lhs: T::SingleDigit, rhs: Digits) -> T;

    /// Left operand is a borrowed slice, right operand a single digit.
    fn apply_ref_digit(lhs: &[Digit], rhs: T::SingleDigit) -> T;

    /// Both operands are borrowed slices.
    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> T;

    /// Left operand is a borrowed slice, right operand an owned buffer.
    fn apply_ref_val(lhs: &[Digit], rhs: Digits) -> T;

    /// Left operand is an owned buffer, right operand a single digit.
    fn apply_val_digit(lhs: Digits, rhs: T::SingleDigit) -> T;

    /// Left operand is an owned buffer, right operand a borrowed slice.
    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> T;

    /// Both operands are owned buffers.
    fn apply_val_val(lhs: Digits, rhs: Digits) -> T;
}

/// A binary operation implemented on the digit representation of a number, with a primitive
/// `Copy` right operand (e.g. a shift amount).
///
/// The left operand appears in one of three forms: a single digit (`digit`), a borrowed
/// slice (`ref`), or an owned buffer (`val`); the right operand is passed by value.
pub(crate) trait BinaryOpDigitsPrimitive<L: AsDigits, R: Copy> {
    /// The left operand is a single digit.
    fn apply_digit(lhs: L::SingleDigit, rhs: R) -> L;

    /// The left operand is a borrowed slice.
    fn apply_ref(lhs: &[Digit], rhs: R) -> L;

    /// The left operand is an owned buffer.
    fn apply_val(lhs: Digits, rhs: R) -> L;
}

/// A commutative binary operation implemented on the digit representation of a number.
///
/// Each operand appears in one of three forms: a single digit (`digit`), a borrowed slice
/// (`ref`), or an owned buffer (`val`).
pub(crate) trait CommutativeBinaryOpDigits<T: AsDigits> {
    /// Both operands are single digits.
    fn apply_digit_digit(lhs: T::SingleDigit, rhs: T::SingleDigit) -> T;

    /// One operand is a borrowed slice, the other a single digit.
    fn apply_ref_digit(lhs: &[Digit], rhs: T::SingleDigit) -> T;

    /// Both operands are borrowed slices.
    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> T;

    /// One operand is an owned buffer, the other a single digit.
    fn apply_val_digit(lhs: Digits, rhs: T::SingleDigit) -> T;

    /// One operand is an owned buffer, the other a borrowed slice.
    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> T;

    /// Both operands are owned buffers.
    fn apply_val_val(lhs: Digits, rhs: Digits) -> T;
}

/// Selects the [`BinaryOpDigits`] derivation of [`BinaryOp`] for the marker type `Op`.
pub(crate) struct DigitsRhs<Op>(Op);

/// Selects the [`BinaryOpDigitsPrimitive`] derivation of [`BinaryOp`] for the marker type
/// `Op`.
pub(crate) struct PrimitiveRhs<Op>(Op);

/// Every [`BinaryOpDigits`] induces a [`BinaryOp`] over a single type.
impl<T: AsDigits, Op: BinaryOpDigits<T>> BinaryOp<T, T> for DigitsRhs<Op> {
    fn apply_ref_ref(lhs: &T, rhs: &T) -> T {
        match (lhs.as_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpDigits<T>>::apply_digit_digit(a, b),
            (Small(a), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_digit_ref(a, rhs),
            (Large(lhs), Small(b)) => <Op as BinaryOpDigits<T>>::apply_ref_digit(lhs, b),
            (Large(lhs), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_ref_ref(lhs, rhs),
        }
    }

    fn apply_ref_val(lhs: &T, rhs: T) -> T {
        match (lhs.as_digits(), rhs.into_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpDigits<T>>::apply_digit_digit(a, b),
            (Small(a), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_digit_val(a, rhs),
            (Large(lhs), Small(b)) => <Op as BinaryOpDigits<T>>::apply_ref_digit(lhs, b),
            (Large(lhs), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_ref_val(lhs, rhs),
        }
    }

    fn apply_val_ref(lhs: T, rhs: &T) -> T {
        match (lhs.into_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpDigits<T>>::apply_digit_digit(a, b),
            (Small(a), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_digit_ref(a, rhs),
            (Large(lhs), Small(b)) => <Op as BinaryOpDigits<T>>::apply_val_digit(lhs, b),
            (Large(lhs), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_val_ref(lhs, rhs),
        }
    }

    fn apply_val_val(lhs: T, rhs: T) -> T {
        match (lhs.into_digits(), rhs.into_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpDigits<T>>::apply_digit_digit(a, b),
            (Small(a), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_digit_val(a, rhs),
            (Large(lhs), Small(b)) => <Op as BinaryOpDigits<T>>::apply_val_digit(lhs, b),
            (Large(lhs), Large(rhs)) => <Op as BinaryOpDigits<T>>::apply_val_val(lhs, rhs),
        }
    }
}

/// Every [`BinaryOpDigitsPrimitive`] induces a [`BinaryOp`] with a primitive right operand.
impl<L: AsDigits, R: Copy, Op: BinaryOpDigitsPrimitive<L, R>> BinaryOp<L, R> for PrimitiveRhs<Op> {
    fn apply_ref_ref(lhs: &L, rhs: &R) -> L {
        Self::apply_ref_val(lhs, *rhs)
    }

    fn apply_ref_val(lhs: &L, rhs: R) -> L {
        match lhs.as_digits() {
            Small(digit) => <Op as BinaryOpDigitsPrimitive<L, R>>::apply_digit(digit, rhs),
            Large(digits) => <Op as BinaryOpDigitsPrimitive<L, R>>::apply_ref(digits, rhs),
        }
    }

    fn apply_val_ref(lhs: L, rhs: &R) -> L {
        Self::apply_val_val(lhs, *rhs)
    }

    fn apply_val_val(lhs: L, rhs: R) -> L {
        match lhs.into_digits() {
            Small(digit) => <Op as BinaryOpDigitsPrimitive<L, R>>::apply_digit(digit, rhs),
            Large(digits) => <Op as BinaryOpDigitsPrimitive<L, R>>::apply_val(digits, rhs),
        }
    }
}

/// Every [`CommutativeBinaryOpDigits`] is a [`BinaryOpDigits`].
impl<T: AsDigits, Op: CommutativeBinaryOpDigits<T>> BinaryOpDigits<T> for Op {
    fn apply_digit_digit(lhs: T::SingleDigit, rhs: T::SingleDigit) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_digit_digit(lhs, rhs)
    }

    fn apply_digit_ref(lhs: T::SingleDigit, rhs: &[Digit]) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_ref_digit(rhs, lhs)
    }

    fn apply_digit_val(lhs: T::SingleDigit, rhs: Digits) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_val_digit(rhs, lhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: T::SingleDigit) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_ref_digit(lhs, rhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_ref_ref(lhs, rhs)
    }

    fn apply_ref_val(lhs: &[Digit], rhs: Digits) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_val_ref(rhs, lhs)
    }

    fn apply_val_digit(lhs: Digits, rhs: T::SingleDigit) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_val_digit(lhs, rhs)
    }

    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_val_ref(lhs, rhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> T {
        <Op as CommutativeBinaryOpDigits<T>>::apply_val_val(lhs, rhs)
    }
}

/// Implements a binary operator and its assigning counterpart for a left operand type `$left` and
/// right operand type `$right`, deriving every owned/borrowed operand combination from a
/// [`BinaryOp`] implemented by the marker type `$op`. The output type is `$left`.
///
/// `$trait`/`$method` and `$assign_trait`/`$assign_method` are the operator and assigning-operator
/// traits; they must be in scope at the call site.
macro_rules! impl_binary_operator {
    ($left:ty, $right:ty, $trait:ident :: $method:ident, $assign_trait:ident :: $assign_method:ident, $op:ty) => {
        impl $trait<$right> for $left {
            type Output = $left;

            fn $method(self, rhs: $right) -> $left {
                <$op as $crate::ops::BinaryOp<$left, $right>>::apply_val_val(self, rhs)
            }
        }

        impl $trait<&$right> for $left {
            type Output = $left;

            fn $method(self, rhs: &$right) -> $left {
                <$op as $crate::ops::BinaryOp<$left, $right>>::apply_val_ref(self, rhs)
            }
        }

        impl $trait<$right> for &$left {
            type Output = $left;

            fn $method(self, rhs: $right) -> $left {
                <$op as $crate::ops::BinaryOp<$left, $right>>::apply_ref_val(self, rhs)
            }
        }

        impl $trait<&$right> for &$left {
            type Output = $left;

            fn $method(self, rhs: &$right) -> $left {
                <$op as $crate::ops::BinaryOp<$left, $right>>::apply_ref_ref(self, rhs)
            }
        }

        impl $assign_trait<$right> for $left {
            fn $assign_method(&mut self, rhs: $right) {
                *self = <$op as $crate::ops::BinaryOp<$left, $right>>::apply_val_val(
                    ::core::mem::take(self),
                    rhs,
                );
            }
        }

        impl $assign_trait<&$right> for $left {
            fn $assign_method(&mut self, rhs: &$right) {
                *self = <$op as $crate::ops::BinaryOp<$left, $right>>::apply_val_ref(
                    ::core::mem::take(self),
                    rhs,
                );
            }
        }
    };
}

pub(crate) use impl_binary_operator;

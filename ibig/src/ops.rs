//! Internal traits describing unary and binary operations.

use crate::repr::{
    AsDigits,
    AsDigitsResult::{Large, Small},
    Digits,
};
use ibig_core::Digit;

/// A unary operation where the operand is borrowed.
pub(crate) trait UnaryOpRef {
    /// The type of the operand.
    type Operand;

    /// The type of the result.
    type Output;

    /// The operand is borrowed.
    fn apply_ref(value: &Self::Operand) -> Self::Output;
}

/// A unary operation where the operand can be borrowed or owned.
pub(crate) trait UnaryOpRefVal {
    /// The type of the operand.
    type Operand;

    /// The type of the result.
    type Output;

    /// The operand is borrowed.
    fn apply_ref(value: &Self::Operand) -> Self::Output;

    /// The operand is owned.
    fn apply_val(value: Self::Operand) -> Self::Output;
}

/// A unary operation implemented on a big number, reading it without consuming it.
///
/// The operand appears in one of two forms: a single digit (`digit`) or a borrowed slice
/// (`ref`).
pub(crate) trait UnaryOpRefBig {
    /// The type of the operand.
    type Operand: AsDigits;

    /// The type of the result.
    type Output;

    /// The operand is a single digit.
    fn apply_digit(operand: <Self::Operand as AsDigits>::SingleDigit) -> Self::Output;

    /// The operand is a borrowed slice.
    fn apply_ref(operand: &[Digit]) -> Self::Output;
}

/// A unary operation implemented on a big number.
///
/// The operand appears in one of three forms: a single digit (`digit`), a borrowed slice
/// (`ref`), or an owned buffer (`val`).
pub(crate) trait UnaryOpRefValBig {
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

/// Every [`UnaryOpRefBig`] induces a [`UnaryOpRef`].
impl<Op: UnaryOpRefBig> UnaryOpRef for Op {
    type Operand = Op::Operand;
    type Output = Op::Output;

    fn apply_ref(value: &Self::Operand) -> Self::Output {
        match value.as_digits() {
            Small(d) => <Op as UnaryOpRefBig>::apply_digit(d),
            Large(digits) => <Op as UnaryOpRefBig>::apply_ref(digits),
        }
    }
}

/// Every [`UnaryOpRefValBig`] induces a [`UnaryOpRefVal`].
impl<Op: UnaryOpRefValBig> UnaryOpRefVal for Op {
    type Operand = Op::Operand;
    type Output = Op::Output;

    fn apply_ref(value: &Self::Operand) -> Self::Output {
        match value.as_digits() {
            Small(d) => <Op as UnaryOpRefValBig>::apply_digit(d),
            Large(digits) => <Op as UnaryOpRefValBig>::apply_ref(digits),
        }
    }

    fn apply_val(value: Self::Operand) -> Self::Output {
        match value.into_digits() {
            Small(d) => <Op as UnaryOpRefValBig>::apply_digit(d),
            Large(digits) => <Op as UnaryOpRefValBig>::apply_val(digits),
        }
    }
}

/// Implements a unary operator for a value type `$t`, deriving the owned and borrowed forms from
/// an [`UnaryOpRefVal`] implemented by the marker type `$op`.
///
/// `$trait`/`$method` is the operator trait; it must be in scope at the call site.
macro_rules! impl_unary_operator {
    ($trait:ident :: $method:ident($operand:ty) -> $output:ty, $op:ty) => {
        impl $trait for $operand {
            type Output = $output;

            fn $method(self) -> Self::Output {
                <$op as $crate::ops::UnaryOpRefVal>::apply_val(self)
            }
        }

        impl $trait for &$operand {
            type Output = $output;

            fn $method(self) -> Self::Output {
                <$op as $crate::ops::UnaryOpRefVal>::apply_ref(self)
            }
        }
    };
}

pub(crate) use impl_unary_operator;

/// A binary operation where the left operand is borrowed and the right is a `Copy` value.
pub(crate) trait BinaryOpRefCopy {
    /// The type of the left operand.
    type Left;

    /// The type of the right operand.
    type Right: Copy;

    /// The type of the result.
    type Output;

    /// The left operand is borrowed, the right operand a `Copy` value.
    fn apply_ref(lhs: &Self::Left, rhs: Self::Right) -> Self::Output;
}

/// A binary operation where the left operand is owned and the right is a `Copy` value.
pub(crate) trait BinaryOpValCopy {
    /// The type of the left operand.
    type Left;

    /// The type of the right operand.
    type Right: Copy;

    /// The type of the result.
    type Output;

    /// The left operand is owned, the right operand a `Copy` value.
    fn apply_val(lhs: Self::Left, rhs: Self::Right) -> Self::Output;
}

/// A binary operation where both operands are borrowed.
pub(crate) trait BinaryOpRef {
    /// The type of the left operand.
    type Left;

    /// The type of the right operand.
    type Right;

    /// The type of the result.
    type Output;

    /// Both operands are borrowed.
    fn apply_ref_ref(lhs: &Self::Left, rhs: &Self::Right) -> Self::Output;
}

/// A binary operation where each operand can be borrowed or owned.
pub(crate) trait BinaryOpRefVal {
    /// The type of the left operand.
    type Left;

    /// The type of the right operand.
    type Right;

    /// The type of the result.
    type Output;

    /// Both operands are borrowed.
    fn apply_ref_ref(lhs: &Self::Left, rhs: &Self::Right) -> Self::Output;

    /// Left operand is borrowed, right operand owned.
    fn apply_ref_val(lhs: &Self::Left, rhs: Self::Right) -> Self::Output;

    /// Left operand is owned, right operand borrowed.
    fn apply_val_ref(lhs: Self::Left, rhs: &Self::Right) -> Self::Output;

    /// Both operands are owned.
    fn apply_val_val(lhs: Self::Left, rhs: Self::Right) -> Self::Output;
}

/// A binary operation on a big number and a `Copy` right operand, reading the big number
/// without consuming it.
///
/// The big number appears as either a single digit (`digit`) or a borrowed slice (`ref`);
/// the right operand is a `Copy` value.
pub(crate) trait BinaryOpRefBigCopy {
    /// The type of the left operand.
    type Left: AsDigits;

    /// The type of the right operand.
    type Right: Copy;

    /// The type of the result.
    type Output;

    /// The left operand is a single digit.
    fn apply_digit(lhs: <Self::Left as AsDigits>::SingleDigit, rhs: Self::Right) -> Self::Output;

    /// The left operand is a borrowed slice.
    fn apply_ref(lhs: &[Digit], rhs: Self::Right) -> Self::Output;
}

/// A binary operation on a big number and a `Copy` right operand, consuming the big number.
///
/// The big number appears as either a single digit (`digit`) or an owned buffer (`val`);
/// the right operand is a `Copy` value.
pub(crate) trait BinaryOpValBigCopy {
    /// The type of the left operand.
    type Left: AsDigits;

    /// The type of the right operand.
    type Right: Copy;

    /// The type of the result.
    type Output;

    /// The left operand is a single digit.
    fn apply_digit(lhs: <Self::Left as AsDigits>::SingleDigit, rhs: Self::Right) -> Self::Output;

    /// The left operand is an owned buffer.
    fn apply_val(lhs: Digits, rhs: Self::Right) -> Self::Output;
}

/// A binary operation on big numbers, reading them without consuming them.
///
/// Each operand appears as either a single digit (`digit`) or a borrowed slice (`ref`).
pub(crate) trait BinaryOpRefBigBig {
    /// The type of the left operand.
    type Left: AsDigits;

    /// The type of the right operand.
    type Right: AsDigits;

    /// The type of the result.
    type Output;

    /// Both operands are single digits.
    fn apply_digit_digit(
        lhs: <Self::Left as AsDigits>::SingleDigit,
        rhs: <Self::Right as AsDigits>::SingleDigit,
    ) -> Self::Output;

    /// Left operand is a single digit, right operand a borrowed slice.
    fn apply_digit_ref(lhs: <Self::Left as AsDigits>::SingleDigit, rhs: &[Digit]) -> Self::Output;

    /// Left operand is a borrowed slice, right operand a single digit.
    fn apply_ref_digit(lhs: &[Digit], rhs: <Self::Right as AsDigits>::SingleDigit) -> Self::Output;

    /// Both operands are borrowed slices.
    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Self::Output;
}

/// A commutative binary operation on big numbers, reading them without consuming them.
///
/// Each operand appears as either a single digit (`digit`) or a borrowed slice (`ref`).
pub(crate) trait CommutativeBinaryOpRefBigBig {
    /// The type of operands.
    type Operand: AsDigits;

    /// The type of the result.
    type Output;

    /// Both operands are single digits.
    fn apply_digit_digit(
        lhs: <Self::Operand as AsDigits>::SingleDigit,
        rhs: <Self::Operand as AsDigits>::SingleDigit,
    ) -> Self::Output;

    /// One operand is a borrowed slice, the other a single digit.
    fn apply_ref_digit(
        lhs: &[Digit],
        rhs: <Self::Operand as AsDigits>::SingleDigit,
    ) -> Self::Output;

    /// Both operands are borrowed slices.
    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Self::Output;
}

/// A binary operation implemented on a big number and a `Copy` right operand.
///
/// The big number appears in one of three forms: a single digit (`digit`), a borrowed
/// slice (`ref`), or an owned buffer (`val`); the right operand is a `Copy` value.
pub(crate) trait BinaryOpRefValBigCopy {
    /// The type of the left operand.
    type Left: AsDigits;

    /// The type of the right operand.
    type Right: Copy;

    /// The type of the result.
    type Output;

    /// The left operand is a single digit.
    fn apply_digit(lhs: <Self::Left as AsDigits>::SingleDigit, rhs: Self::Right) -> Self::Output;

    /// The left operand is a borrowed slice.
    fn apply_ref(lhs: &[Digit], rhs: Self::Right) -> Self::Output;

    /// The left operand is an owned buffer.
    fn apply_val(lhs: Digits, rhs: Self::Right) -> Self::Output;
}

/// A binary operation on big numbers.
///
/// Each operand appears in one of three forms: a single digit (`digit`), a borrowed slice
/// (`ref`), or an owned buffer (`val`).
pub(crate) trait BinaryOpRefValBigBig {
    /// The type of the left operand.
    type Left: AsDigits;

    /// The type of the right operand.
    type Right: AsDigits;

    /// The type of the result.
    type Output;

    /// Both operands are single digits.
    fn apply_digit_digit(
        lhs: <Self::Left as AsDigits>::SingleDigit,
        rhs: <Self::Right as AsDigits>::SingleDigit,
    ) -> Self::Output;

    /// Left operand is a single digit, right operand a borrowed slice.
    fn apply_digit_ref(lhs: <Self::Left as AsDigits>::SingleDigit, rhs: &[Digit]) -> Self::Output;

    /// Left operand is a single digit, right operand an owned buffer.
    fn apply_digit_val(lhs: <Self::Left as AsDigits>::SingleDigit, rhs: Digits) -> Self::Output;

    /// Left operand is a borrowed slice, right operand a single digit.
    fn apply_ref_digit(lhs: &[Digit], rhs: <Self::Right as AsDigits>::SingleDigit) -> Self::Output;

    /// Both operands are borrowed slices.
    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Self::Output;

    /// Left operand is a borrowed slice, right operand an owned buffer.
    fn apply_ref_val(lhs: &[Digit], rhs: Digits) -> Self::Output;

    /// Left operand is an owned buffer, right operand a single digit.
    fn apply_val_digit(lhs: Digits, rhs: <Self::Right as AsDigits>::SingleDigit) -> Self::Output;

    /// Left operand is an owned buffer, right operand a borrowed slice.
    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> Self::Output;

    /// Both operands are owned buffers.
    fn apply_val_val(lhs: Digits, rhs: Digits) -> Self::Output;
}

/// A commutative binary operation on big numbers.
///
/// Each operand appears in one of three forms: a single digit (`digit`), a borrowed slice
/// (`ref`), or an owned buffer (`val`).
pub(crate) trait CommutativeBinaryOpRefValBigBig {
    /// The type of operands.
    type Operand: AsDigits;

    /// The type of the result.
    type Output;

    /// Both operands are single digits.
    fn apply_digit_digit(
        lhs: <Self::Operand as AsDigits>::SingleDigit,
        rhs: <Self::Operand as AsDigits>::SingleDigit,
    ) -> Self::Output;

    /// One operand is a borrowed slice, the other a single digit.
    fn apply_ref_digit(
        lhs: &[Digit],
        rhs: <Self::Operand as AsDigits>::SingleDigit,
    ) -> Self::Output;

    /// Both operands are borrowed slices.
    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Self::Output;

    /// One operand is an owned buffer, the other a single digit.
    fn apply_val_digit(lhs: Digits, rhs: <Self::Operand as AsDigits>::SingleDigit) -> Self::Output;

    /// One operand is an owned buffer, the other a borrowed slice.
    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> Self::Output;

    /// Both operands are owned buffers.
    fn apply_val_val(lhs: Digits, rhs: Digits) -> Self::Output;
}

/// Every [`BinaryOpRefBigCopy`] induces a [`BinaryOpRefCopy`].
impl<Op: BinaryOpRefBigCopy> BinaryOpRefCopy for Op {
    type Left = Op::Left;
    type Right = Op::Right;
    type Output = Op::Output;

    fn apply_ref(lhs: &Self::Left, rhs: Self::Right) -> Self::Output {
        match lhs.as_digits() {
            Small(a) => <Op as BinaryOpRefBigCopy>::apply_digit(a, rhs),
            Large(a) => <Op as BinaryOpRefBigCopy>::apply_ref(a, rhs),
        }
    }
}

/// Every [`BinaryOpValBigCopy`] induces a [`BinaryOpValCopy`].
impl<Op: BinaryOpValBigCopy> BinaryOpValCopy for Op {
    type Left = Op::Left;
    type Right = Op::Right;
    type Output = Op::Output;

    fn apply_val(lhs: Self::Left, rhs: Self::Right) -> Self::Output {
        match lhs.into_digits() {
            Small(a) => <Op as BinaryOpValBigCopy>::apply_digit(a, rhs),
            Large(a) => <Op as BinaryOpValBigCopy>::apply_val(a, rhs),
        }
    }
}

/// Every [`BinaryOpRefBigBig`] induces a [`BinaryOpRef`].
impl<Op: BinaryOpRefBigBig> BinaryOpRef for Op {
    type Left = Op::Left;
    type Right = Op::Right;
    type Output = Op::Output;

    fn apply_ref_ref(lhs: &Self::Left, rhs: &Self::Right) -> Self::Output {
        match (lhs.as_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpRefBigBig>::apply_digit_digit(a, b),
            (Small(a), Large(b)) => <Op as BinaryOpRefBigBig>::apply_digit_ref(a, b),
            (Large(a), Small(b)) => <Op as BinaryOpRefBigBig>::apply_ref_digit(a, b),
            (Large(a), Large(b)) => <Op as BinaryOpRefBigBig>::apply_ref_ref(a, b),
        }
    }
}

/// Every [`CommutativeBinaryOpRefBigBig`] is a [`BinaryOpRefBigBig`].
impl<Op: CommutativeBinaryOpRefBigBig> BinaryOpRefBigBig for Op {
    type Left = Op::Operand;
    type Right = Op::Operand;
    type Output = Op::Output;

    fn apply_digit_digit(
        lhs: <Op::Operand as AsDigits>::SingleDigit,
        rhs: <Op::Operand as AsDigits>::SingleDigit,
    ) -> Self::Output {
        <Op as CommutativeBinaryOpRefBigBig>::apply_digit_digit(lhs, rhs)
    }

    fn apply_digit_ref(lhs: <Op::Operand as AsDigits>::SingleDigit, rhs: &[Digit]) -> Self::Output {
        <Op as CommutativeBinaryOpRefBigBig>::apply_ref_digit(rhs, lhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: <Op::Operand as AsDigits>::SingleDigit) -> Self::Output {
        <Op as CommutativeBinaryOpRefBigBig>::apply_ref_digit(lhs, rhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Self::Output {
        <Op as CommutativeBinaryOpRefBigBig>::apply_ref_ref(lhs, rhs)
    }
}

/// Wrapper indicating a [`BinaryOpRefValBigCopy`].
pub(crate) struct BigCopy<Op>(Op);

/// Wrapper indicating a [`BinaryOpRefValBigBig`].
pub(crate) struct BigBig<Op>(Op);

/// Every [`BinaryOpRefValBigCopy`] induces a [`BinaryOpRefVal`].
impl<Op: BinaryOpRefValBigCopy> BinaryOpRefVal for BigCopy<Op> {
    type Left = Op::Left;
    type Right = Op::Right;
    type Output = Op::Output;

    fn apply_ref_ref(lhs: &Self::Left, rhs: &Self::Right) -> Self::Output {
        Self::apply_ref_val(lhs, *rhs)
    }

    fn apply_ref_val(lhs: &Self::Left, rhs: Self::Right) -> Self::Output {
        match lhs.as_digits() {
            Small(a) => <Op as BinaryOpRefValBigCopy>::apply_digit(a, rhs),
            Large(a) => <Op as BinaryOpRefValBigCopy>::apply_ref(a, rhs),
        }
    }

    fn apply_val_ref(lhs: Self::Left, rhs: &Self::Right) -> Self::Output {
        Self::apply_val_val(lhs, *rhs)
    }

    fn apply_val_val(lhs: Self::Left, rhs: Self::Right) -> Self::Output {
        match lhs.into_digits() {
            Small(a) => <Op as BinaryOpRefValBigCopy>::apply_digit(a, rhs),
            Large(a) => <Op as BinaryOpRefValBigCopy>::apply_val(a, rhs),
        }
    }
}

/// Every [`BinaryOpRefValBigBig`] induces a [`BinaryOpRefVal`].
impl<Op: BinaryOpRefValBigBig> BinaryOpRefVal for BigBig<Op> {
    type Left = Op::Left;
    type Right = Op::Right;
    type Output = Op::Output;

    fn apply_ref_ref(lhs: &Self::Left, rhs: &Self::Right) -> Self::Output {
        match (lhs.as_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_digit(a, b),
            (Small(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_ref(a, b),
            (Large(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_ref_digit(a, b),
            (Large(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_ref_ref(a, b),
        }
    }

    fn apply_ref_val(lhs: &Self::Left, rhs: Self::Right) -> Self::Output {
        match (lhs.as_digits(), rhs.into_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_digit(a, b),
            (Small(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_val(a, b),
            (Large(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_ref_digit(a, b),
            (Large(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_ref_val(a, b),
        }
    }

    fn apply_val_ref(lhs: Self::Left, rhs: &Self::Right) -> Self::Output {
        match (lhs.into_digits(), rhs.as_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_digit(a, b),
            (Small(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_ref(a, b),
            (Large(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_val_digit(a, b),
            (Large(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_val_ref(a, b),
        }
    }

    fn apply_val_val(lhs: Self::Left, rhs: Self::Right) -> Self::Output {
        match (lhs.into_digits(), rhs.into_digits()) {
            (Small(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_digit(a, b),
            (Small(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_digit_val(a, b),
            (Large(a), Small(b)) => <Op as BinaryOpRefValBigBig>::apply_val_digit(a, b),
            (Large(a), Large(b)) => <Op as BinaryOpRefValBigBig>::apply_val_val(a, b),
        }
    }
}

/// Every [`CommutativeBinaryOpRefValBigBig`] is a [`BinaryOpRefValBigBig`].
impl<Op: CommutativeBinaryOpRefValBigBig> BinaryOpRefValBigBig for Op {
    type Left = Op::Operand;
    type Right = Op::Operand;
    type Output = Op::Output;

    fn apply_digit_digit(
        lhs: <Op::Operand as AsDigits>::SingleDigit,
        rhs: <Op::Operand as AsDigits>::SingleDigit,
    ) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_digit_digit(lhs, rhs)
    }

    fn apply_digit_ref(lhs: <Op::Operand as AsDigits>::SingleDigit, rhs: &[Digit]) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_ref_digit(rhs, lhs)
    }

    fn apply_digit_val(lhs: <Op::Operand as AsDigits>::SingleDigit, rhs: Digits) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_val_digit(rhs, lhs)
    }

    fn apply_ref_digit(lhs: &[Digit], rhs: <Op::Operand as AsDigits>::SingleDigit) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_ref_digit(lhs, rhs)
    }

    fn apply_ref_ref(lhs: &[Digit], rhs: &[Digit]) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_ref_ref(lhs, rhs)
    }

    fn apply_ref_val(lhs: &[Digit], rhs: Digits) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_val_ref(rhs, lhs)
    }

    fn apply_val_digit(lhs: Digits, rhs: <Op::Operand as AsDigits>::SingleDigit) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_val_digit(lhs, rhs)
    }

    fn apply_val_ref(lhs: Digits, rhs: &[Digit]) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_val_ref(lhs, rhs)
    }

    fn apply_val_val(lhs: Digits, rhs: Digits) -> Self::Output {
        <Op as CommutativeBinaryOpRefValBigBig>::apply_val_val(lhs, rhs)
    }
}

/// Implements a binary operator for a left operand type `$left` and
/// right operand type `$right`, deriving every owned/borrowed operand combination from a
/// [`BinaryOpRefVal`] implemented by the marker type `$op`.
///
/// `$trait`/`$method` is the operator trait; it must be in scope at the call site.
macro_rules! impl_binary_operator {
    ($trait:ident :: $method:ident($left:ty, $right:ty) -> $output:ty, $assign_trait:ident :: $assign_method:ident, $op:ty) => {
        impl $trait<$right> for $left {
            type Output = $output;

            fn $method(self, rhs: $right) -> $output {
                <$op as $crate::ops::BinaryOpRefVal>::apply_val_val(self, rhs)
            }
        }

        impl $trait<&$right> for $left {
            type Output = $output;

            fn $method(self, rhs: &$right) -> $output {
                <$op as $crate::ops::BinaryOpRefVal>::apply_val_ref(self, rhs)
            }
        }

        impl $trait<$right> for &$left {
            type Output = $output;

            fn $method(self, rhs: $right) -> $output {
                <$op as $crate::ops::BinaryOpRefVal>::apply_ref_val(self, rhs)
            }
        }

        impl $trait<&$right> for &$left {
            type Output = $output;

            fn $method(self, rhs: &$right) -> $output {
                <$op as $crate::ops::BinaryOpRefVal>::apply_ref_ref(self, rhs)
            }
        }

        impl $assign_trait<$right> for $left {
            fn $assign_method(&mut self, rhs: $right) {
                *self = <$op as $crate::ops::BinaryOpRefVal>::apply_val_val(
                    ::core::mem::take(self),
                    rhs,
                );
            }
        }

        impl $assign_trait<&$right> for $left {
            fn $assign_method(&mut self, rhs: &$right) {
                *self = <$op as $crate::ops::BinaryOpRefVal>::apply_val_ref(
                    ::core::mem::take(self),
                    rhs,
                );
            }
        }
    };
}

pub(crate) use impl_binary_operator;

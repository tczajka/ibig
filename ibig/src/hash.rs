//! Hashing.

use crate::repr::AsDigits;
use crate::repr::AsDigitsResult::{Large, Small};
use crate::{IBig, UBig};
use core::hash::{Hash, Hasher};

impl Hash for UBig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.as_digits() {
            Small(digit) => [digit].hash(state),
            Large(digits) => digits.hash(state),
        }
    }
}

impl Hash for IBig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.as_digits() {
            Small(idigit) => [idigit.cast_unsigned()].hash(state),
            Large(digits) => digits.hash(state),
        }
    }
}

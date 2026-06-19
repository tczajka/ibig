//! Integration tests for hashing.

use ibig::{IBig, UBig};
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn ubig() {
    // Equal values hash equally — single digit and multi-digit.
    assert_eq!(hash(&UBig::from(5u8)), hash(&UBig::from(5u8)));
    let a = (UBig::from(1u8) << 200) + UBig::from(7u8);
    let b = (UBig::from(1u8) << 200) + UBig::from(7u8);
    assert_eq!(hash(&a), hash(&b));

    // Works as a `HashSet` key (relies on `Hash`/`Eq` agreeing).
    let mut set = HashSet::new();
    set.insert(UBig::from(5u8));
    set.insert(a.clone());
    assert!(set.contains(&UBig::from(5u8)));
    assert!(set.contains(&((UBig::from(1u8) << 200) + UBig::from(7u8))));
    assert!(!set.contains(&UBig::from(6u8)));
}

#[test]
fn ibig() {
    assert_eq!(hash(&IBig::from(-5)), hash(&IBig::from(-5)));
    let a = (IBig::from(-1) << 200) + IBig::from(3);
    let b = (IBig::from(-1) << 200) + IBig::from(3);
    assert_eq!(hash(&a), hash(&b));

    let mut set = HashSet::new();
    set.insert(IBig::from(-5));
    set.insert(a.clone());
    assert!(set.contains(&IBig::from(-5)));
    assert!(set.contains(&((IBig::from(-1) << 200) + IBig::from(3))));
    assert!(!set.contains(&IBig::from(5)));
}

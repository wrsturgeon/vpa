/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Range of values that, unlike `core::ops::Range...`, implements `Ord`.

use core::cmp::Ordering;

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

/// Range of values that, unlike `core::ops::Range...`, implements `Ord`.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Range<T: Ord> {
    /// First value (inclusive).
    pub first: T,
    /// Last value (INCLUSIVE).
    pub last: T,
}

impl<T: Ord> Range<T> {
    /// Check if a value lies within this range.
    #[inline(always)]
    pub fn contains(&self, value: &T) -> Ordering {
        if *value < self.first {
            Ordering::Less
        } else if *value > self.last {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    /// Check if any value lies in both of these ranges simultaneously.
    #[inline]
    pub fn overlap(&self, other: &Self) -> bool {
        other.last >= self.first || other.first <= self.last
    }
}

impl<T: Clone + Ord> Range<T> {
    /// Check if any value lies in both of these ranges simultaneously.
    #[inline]
    pub fn union(&self, other: &Self) -> Option<Self> {
        let first = self.first.clone().max(other.first.clone());
        let last = self.first.clone().min(other.first.clone());
        (first <= last).then_some(Self { first, last })
    }
}

impl<T: Clone + Ord> Range<T> {
    /// Construct a range with only one element.
    #[inline]
    pub fn unit(first_and_last: T) -> Self {
        Self {
            first: first_and_last.clone(),
            last: first_and_last,
        }
    }
}

/// Check if all ranges in a list are disjoint.
#[inline]
#[allow(dead_code)] // <-- TODO: Use somewhere!!!
#[allow(unused_unsafe)] // <-- false positive: nested macros that each use `unsafe`
fn disjoint<T: Ord>(array: &[Range<T>]) -> bool {
    array.iter().enumerate().all(|(i, a)| {
        get!(array, unwrap!(i.checked_add(1))..)
            .iter()
            .all(|b| !a.overlap(b))
    })
}

#[cfg(feature = "quickcheck")]
impl<T: Arbitrary + Ord> Arbitrary for Range<T> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let (a, b) = <(T, T)>::arbitrary(g);
        if a < b {
            Self { first: a, last: b }
        } else {
            Self { first: b, last: a }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.first.clone(), self.last.clone())
                .shrink()
                .map(|(a, b)| {
                    if a < b {
                        Self { first: a, last: b }
                    } else {
                        Self { first: b, last: a }
                    }
                }),
        )
    }
}

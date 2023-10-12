/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::IllFormed;
use core::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

/// Trait to fallibly combine multiple values into one value with identical semantics.
pub trait Merge: Sized {
    /// Fallibly combine multiple values into one value with identical semantics.
    /// # Errors
    /// Implementation-defined: if the merge as we define it can't work.
    fn merge(self, other: &Self) -> Result<Self, IllFormed>;
}

impl Merge for usize {
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed> {
        if self == *other {
            Ok(self)
        } else {
            Err(IllFormed)
        }
    }
}

impl<T: Clone + Merge> Merge for Option<T> {
    #[inline(always)]
    fn merge(self, other: &Self) -> Result<Self, IllFormed> {
        Ok(match (self, other) {
            (None, &None) => None,
            (Some(a), &None) => Some(a),
            (None, &Some(ref b)) => Some(b.clone()),
            (Some(a), &Some(ref b)) => Some(a.merge(b)?),
        })
    }
}

impl<T: Clone + Ord> Merge for BTreeSet<T> {
    #[inline(always)]
    fn merge(mut self, other: &Self) -> Result<Self, IllFormed> {
        self.extend(other.iter().cloned());
        Ok(self)
    }
}

impl<K: Clone + Ord, V: Clone> Merge for BTreeMap<K, V> {
    #[inline(always)]
    fn merge(mut self, other: &Self) -> Result<Self, IllFormed> {
        for (k, v) in other {
            if self.insert(k.clone(), v.clone()).is_some() {
                return Err(IllFormed);
            }
        }
        Ok(self)
    }
}

/// Merge an entire iterator into a value.
#[inline]
pub fn merge<M: Clone + Merge, I: IntoIterator>(i: I) -> Option<Result<M, IllFormed>>
where
    I::Item: Borrow<M>,
{
    let mut iter = i.into_iter();
    let first = iter.next()?;
    Some(iter.try_fold(first.borrow().clone(), |acc, m| acc.merge(m.borrow())))
}

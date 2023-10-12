/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::IllFormed;
use std::collections::BTreeSet;

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

impl<T: Clone + Merge + Ord> Merge for BTreeSet<T> {
    #[inline(always)]
    fn merge(mut self, other: &Self) -> Result<Self, IllFormed> {
        self.extend(other.iter().cloned());
        Ok(self)
    }
}

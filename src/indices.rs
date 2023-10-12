/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Collection of indices.

use core::iter::{once, Once};
use std::collections::{btree_set::Iter, BTreeSet};

/// Anything that can act as one or more state indices for an automaton.
pub trait Indices: Clone {
    /// Iterator over references to elements without consuming the collection.
    type View<'a>: Iterator<Item = &'a usize>
    where
        Self: 'a;
    /// Iterate over references to elements without consuming the collection.
    #[must_use]
    fn iter(&self) -> Self::View<'_>;
    /// Apply a function to each index.
    #[must_use]
    fn map<F: FnMut(usize) -> usize>(self, f: F) -> Self;
    /// Apply a function to each index, then synthesize the rest into this type again.
    #[must_use]
    fn flat_map<F: FnMut(usize) -> Self>(self, f: F) -> Self;
    /// Collect an iterator into this type.
    /// # Errors
    /// If the iterator is empty.
    fn collect<I: IntoIterator<Item = usize>>(iter: I) -> Result<Self, bool>;
}

impl Indices for usize {
    type View<'a> = Once<&'a usize>;
    #[inline(always)]
    fn iter(&self) -> Self::View<'_> {
        once(self)
    }
    #[inline(always)]
    fn map<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        f(self)
    }
    #[inline(always)]
    fn flat_map<F: FnMut(usize) -> Self>(self, mut f: F) -> Self {
        f(self)
    }
    #[inline]
    fn collect<I: IntoIterator<Item = usize>>(iter: I) -> Result<Self, bool> {
        let mut i = iter.into_iter();
        let rtn = i.next().ok_or(false);
        #[cfg(any(test, debug_assertions))]
        {
            let leftovers: Vec<_> = i.collect();
            debug_assert_eq!(
                leftovers,
                vec![],
                "Tried to collect an iterator into a single index but there was {leftovers:?} left over"
            );
        }
        rtn
    }
}

impl Indices for BTreeSet<usize> {
    type View<'a> = Iter<'a, usize>;
    #[inline(always)]
    fn iter(&self) -> Self::View<'_> {
        self.iter()
    }
    #[inline(always)]
    fn map<F: FnMut(usize) -> usize>(self, f: F) -> Self {
        self.into_iter().map(f).collect()
    }
    #[inline(always)]
    fn flat_map<F: FnMut(usize) -> Self>(self, f: F) -> Self {
        self.into_iter().flat_map(f).collect()
    }
    #[inline]
    fn collect<I: IntoIterator<Item = usize>>(iter: I) -> Result<Self, bool> {
        let mut i = iter.into_iter();
        i.next()
            .ok_or(false)
            .map(|first| once(first).chain(i).collect())
    }
}

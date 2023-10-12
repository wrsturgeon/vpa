/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to define fallible lookup.

use std::collections::BTreeMap;

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

/// Trait to define fallible lookup.
pub trait Lookup {
    /// Input to a lookup.
    type Key<'k>: Copy;
    /// Output of a successful lookup.
    type Value;
    /// Look up an element.
    #[must_use]
    fn get(&self, key: Self::Key<'_>) -> Option<&Self::Value>;
    /// Apply a function to each value.
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, f: F);
}

impl<K: 'static + Ord, V: 'static> Lookup for BTreeMap<K, V> {
    type Key<'k> = &'k K;
    type Value = V;
    #[inline(always)]
    fn get(&self, key: Self::Key<'_>) -> Option<&Self::Value> {
        BTreeMap::get(self, key)
    }
    #[inline]
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, mut f: F) {
        for v in self.values_mut() {
            f(v);
        }
    }
}

/// Trivial lookup to end currying: just return this value.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Return<T>(pub T);

impl<T: 'static> Lookup for Return<T> {
    type Key<'k> = ();
    type Value = T;
    #[inline(always)]
    fn get(&self, (): Self::Key<'_>) -> Option<&Self::Value> {
        Some(&self.0)
    }
    #[inline(always)]
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, mut f: F) {
        f(&mut self.0);
    }
}

#[cfg(feature = "quickcheck")]
impl<T: Arbitrary> Arbitrary for Return<T> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self(Arbitrary::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}
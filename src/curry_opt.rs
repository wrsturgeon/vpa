/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from an optional top-of-stack symbol (optional b/c it might be empty) to _another map_ that matches input tokens.
//! # Why is this necessary?
//! Using a `BTreeMap<Option<T>, BTreeMap<...` means we need to call `get` with an `&Option<T>`, not an `Option<&T>`.
//! This is generally extremely difficult to do without cloning the `T`, and
//! I don't want to impose a `Clone` bound on a type that never actually needs to be cloned
//! just because an interpreter would be easier to write if it were `Clone`.

use crate::{Lookup, Merge};
use core::{iter::*, option};
use std::collections::{
    btree_map::{IntoIter, Iter},
    BTreeMap,
};

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

/// Map from an optional top-of-stack symbol (optional b/c it might be empty) to _another map_ that matches input tokens.
/// # Why is this necessary?
/// Using a `BTreeMap<Option<T>, BTreeMap<...` means we need to call `get` with an `&Option<T>`, not an `Option<&T>`.
/// This is generally extremely difficult to do without cloning the `T`, and
/// I don't want to impose a `Clone` bound on a type that never actually needs to be cloned
/// just because an interpreter would be easier to write if it were `Clone`.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurryOpt<Arg: Ord, Etc: Lookup> {
    /// First, try to match this, no matter what the argument was.
    pub wildcard: Option<Etc>,
    /// If the wildcard match didn't work, try this if the argument is `None`.
    pub none: Option<Etc>,
    /// If the wildcard match didn't work, try this if the argument is `Some(..)`.
    pub some: BTreeMap<Arg, Etc>,
}

impl<Arg: Ord, Etc: Lookup> Default for CurryOpt<Arg, Etc> {
    #[inline]
    fn default() -> Self {
        Self {
            wildcard: None,
            none: None,
            some: BTreeMap::new(),
        }
    }
}

impl<Arg: 'static + Ord, Etc: 'static + Lookup> Lookup for CurryOpt<Arg, Etc> {
    type Key<'k> = (Option<&'k Arg>, Etc::Key<'k>);
    type Value = Etc::Value;
    #[inline]
    fn get(&self, key: Self::Key<'_>) -> Option<&Self::Value> {
        let (head, tail) = key;
        if let woohoo @ Some(_) = self.wildcard.as_ref().and_then(|etc| etc.get(tail)) {
            #[allow(clippy::panic)]
            #[cfg(any(test, debug_assertions))]
            {
                if head
                    .map_or(self.none.as_ref().and_then(|etc| etc.get(tail)), |some| {
                        self.some.get(some).and_then(|etc| etc.get(tail))
                    })
                    .is_some()
                {
                    panic!("Duplicate value as both a wildcard and a non-wildcard");
                }
            }
            woohoo
        } else {
            head.map_or(self.none.as_ref().and_then(|etc| etc.get(tail)), |some| {
                self.some.get(some).and_then(|etc| etc.get(tail))
            })
        }
    }
    #[inline]
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, mut f: F) {
        if let Some(ref mut wild) = self.wildcard {
            wild.map_values(&mut f);
        }
        if let Some(ref mut none) = self.none {
            none.map_values(&mut f);
        }
        for v in self.some.values_mut() {
            v.map_values(&mut f);
        }
    }
}

impl<Arg: Clone + Ord, Etc: Clone + Lookup + Merge> Merge for CurryOpt<Arg, Etc> {
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, crate::IllFormed> {
        Ok(Self {
            wildcard: self.wildcard.merge(&other.wildcard)?,
            none: self.none.merge(&other.none)?,
            some: self.some.merge(&other.some)?,
        })
    }
}

impl<Arg: Ord, Etc: Lookup> IntoIterator for CurryOpt<Arg, Etc> {
    type Item = (Option<Option<Arg>>, Etc);
    type IntoIter = Chain<
        Chain<
            Map<option::IntoIter<Etc>, fn(Etc) -> (Option<Option<Arg>>, Etc)>,
            Map<option::IntoIter<Etc>, fn(Etc) -> (Option<Option<Arg>>, Etc)>,
        >,
        Map<IntoIter<Arg, Etc>, fn((Arg, Etc)) -> (Option<Option<Arg>>, Etc)>,
    >;
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        self.wildcard
            .into_iter()
            .map((|some| (None, some)) as _)
            .chain(self.none.into_iter().map((|some| (Some(None), some)) as _))
            .chain(
                self.some
                    .into_iter()
                    .map((|(k, v)| (Some(Some(k)), v)) as _),
            )
    }
}

impl<'a, Arg: Ord, Etc: Lookup> IntoIterator for &'a CurryOpt<Arg, Etc> {
    type Item = (Option<Option<&'a Arg>>, &'a Etc);
    type IntoIter = Chain<
        Chain<
            Map<option::Iter<'a, Etc>, fn(&'a Etc) -> (Option<Option<&'a Arg>>, &'a Etc)>,
            Map<option::Iter<'a, Etc>, fn(&'a Etc) -> (Option<Option<&'a Arg>>, &'a Etc)>,
        >,
        Map<Iter<'a, Arg, Etc>, fn((&'a Arg, &'a Etc)) -> (Option<Option<&'a Arg>>, &'a Etc)>,
    >;
    #[inline]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        self.wildcard
            .iter()
            .map((|some| (None, some)) as _)
            .chain(self.none.iter().map((|some| (Some(None), some)) as _))
            .chain(self.some.iter().map((|(k, v)| (Some(Some(k)), v)) as _))
    }
}

impl<Arg: Ord, Etc: Lookup> CurryOpt<Arg, Etc> {
    /// Iterate over values only, ignoring keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Etc> {
        self.wildcard
            .iter()
            .chain(self.none.iter())
            .chain(self.some.values())
    }
}

#[cfg(feature = "quickcheck")]
impl<Arg: Arbitrary + Ord, Etc: Arbitrary + Lookup> Arbitrary for CurryOpt<Arg, Etc> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            wildcard: Arbitrary::arbitrary(g),
            none: Arbitrary::arbitrary(g),
            some: Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.wildcard.clone(), self.none.clone(), self.some.clone())
                .shrink()
                .map(|(wildcard, none, some)| Self {
                    wildcard,
                    none,
                    some,
                }),
        )
    }
}

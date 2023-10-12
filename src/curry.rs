/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from a potential wildcard to _another map_.

use crate::{Lookup, Merge};
use core::{iter::*, option};
use std::collections::{
    btree_map::{IntoIter, Iter},
    BTreeMap,
};

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

/// Map from a potential wildcard to _another map_.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Curry<Arg: Ord, Etc: Lookup> {
    /// First, try to match this, no matter what the argument was.
    pub wildcard: Option<Etc>,
    /// If the wildcard match didn't work, try this.
    pub specific: BTreeMap<Arg, Etc>,
}

impl<Arg: 'static + Ord, Etc: 'static + Lookup> Lookup for Curry<Arg, Etc> {
    type Key<'k> = (&'k Arg, Etc::Key<'k>);
    type Value = Etc::Value;
    #[inline]
    fn get(&self, key: Self::Key<'_>) -> Option<&Self::Value> {
        let (head, tail) = key;
        if let woohoo @ Some(_) = self.wildcard.as_ref().and_then(|etc| etc.get(tail)) {
            #[allow(clippy::panic)]
            #[cfg(any(test, debug_assertions))]
            {
                if self
                    .specific
                    .get(head)
                    .and_then(|etc| etc.get(tail))
                    .is_some()
                {
                    panic!("Duplicate value as both a wildcard and a non-wildcard");
                }
            }
            woohoo
        } else {
            self.specific.get(head).and_then(|etc| etc.get(tail))
        }
    }
    #[inline]
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, mut f: F) {
        if let Some(ref mut wild) = self.wildcard {
            wild.map_values(&mut f);
        }
        for v in self.specific.values_mut() {
            v.map_values(&mut f);
        }
    }
}

impl<Arg: Clone + Ord, Etc: Clone + Lookup + Merge> Merge for Curry<Arg, Etc> {
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, crate::IllFormed> {
        Ok(Self {
            wildcard: self.wildcard.merge(&other.wildcard)?,
            specific: self.specific.merge(&other.specific)?,
        })
    }
}

impl<Arg: Ord, Etc: Lookup> IntoIterator for Curry<Arg, Etc> {
    type Item = (Option<Arg>, Etc);
    type IntoIter = Chain<
        Map<option::IntoIter<Etc>, fn(Etc) -> (Option<Arg>, Etc)>,
        Map<IntoIter<Arg, Etc>, fn((Arg, Etc)) -> (Option<Arg>, Etc)>,
    >;
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        self.wildcard
            .into_iter()
            .map((|some| (None, some)) as _)
            .chain(self.specific.into_iter().map((|(k, v)| (Some(k), v)) as _))
    }
}

impl<'a, Arg: Ord, Etc: Lookup> IntoIterator for &'a Curry<Arg, Etc> {
    type Item = (Option<&'a Arg>, &'a Etc);
    type IntoIter = Chain<
        Map<option::Iter<'a, Etc>, fn(&'a Etc) -> (Option<&'a Arg>, &'a Etc)>,
        Map<Iter<'a, Arg, Etc>, fn((&'a Arg, &'a Etc)) -> (Option<&'a Arg>, &'a Etc)>,
    >;
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        self.wildcard
            .iter()
            .map((|some| (None, some)) as _)
            .chain(self.specific.iter().map((|(k, v)| (Some(k), v)) as _))
    }
}

#[cfg(feature = "quickcheck")]
impl<Arg: Arbitrary + Ord, Etc: Arbitrary + Lookup> Arbitrary for Curry<Arg, Etc> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            wildcard: Arbitrary::arbitrary(g),
            specific: Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.wildcard.clone(), self.specific.clone())
                .shrink()
                .map(|(wildcard, specific)| Self { wildcard, specific }),
        )
    }
}

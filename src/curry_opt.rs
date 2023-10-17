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

use crate::{Curry, Edge, IllFormed, Indices, Lookup, Merge, Return};
use core::{fmt, iter::*, option};
use std::collections::{
    btree_map::{IntoIter, Iter},
    BTreeMap,
};

#[cfg(any(test, feature = "quickcheck"))]
use core::num::NonZeroUsize;

/// Map from an optional top-of-stack symbol (optional b/c it might be empty) to _another map_ that matches input tokens.
/// # Why is this necessary?
/// Using a `BTreeMap<Option<T>, BTreeMap<...` means we need to call `get` with an `&Option<T>`, not an `Option<&T>`.
/// This is generally extremely difficult to do without cloning the `T`, and
/// I don't want to impose a `Clone` bound on a type that never actually needs to be cloned
/// just because an interpreter would be easier to write if it were `Clone`.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl<Arg: fmt::Debug + Ord, Etc: fmt::Debug + Lookup> fmt::Debug for CurryOpt<Arg, Etc> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CurryOpt {{ wildcard: {:?}, none: {:?}, some: {:?}.into_iter().collect() }}",
            self.wildcard,
            self.none,
            self.some.iter().collect::<Vec<_>>(),
        )
    }
}

impl<Arg: 'static + fmt::Debug + Ord, Etc: 'static + Lookup> Lookup for CurryOpt<Arg, Etc>
where
    Etc::Value: fmt::Debug + PartialEq,
{
    type Key<'k> = (Option<&'k Arg>, Etc::Key<'k>);
    type Value = Etc::Value;
    #[inline]
    fn get(&self, key: Self::Key<'_>) -> Option<&Self::Value> {
        let (head, tail) = key;
        if let woohoo @ Some(_) = self.wildcard.as_ref().and_then(|etc| etc.get(tail)) {
            #[cfg(any(test, debug_assertions))]
            {
                assert_eq!(
                    self.get_if_no_wildcard(head, tail),
                    None,
                    "Duplicate value ({head:?}) as both a wildcard and a non-wildcard",
                );
            }
            woohoo
        } else {
            self.get_if_no_wildcard(head, tail)
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

impl<A: 'static + fmt::Debug + Clone + Ord, S: 'static + Copy + Ord, Ctrl: Indices<A, S>>
    Merge<A, S, Ctrl> for CurryOpt<S, Curry<A, Return<Edge<A, S, Ctrl>>>>
{
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        let wildcard = self.wildcard.merge(&other.wildcard)?;
        let none = self.none.merge(&other.none)?;
        let some = self.some.merge(&other.some)?;
        if let Some(wild) = wildcard {
            for curry in &none {
                if let Some(overlap) = wild.disjoint(curry) {
                    // TODO: Doesn't need to reject this generally! Only if there's actually a conflict
                    return Err(IllFormed::CurryOptMergeConflict(None, overlap));
                }
            }
            for (arg, curry) in &some {
                if let Some(overlap) = wild.disjoint(curry) {
                    // TODO: Doesn't need to reject this generally! Only if there's actually a conflict
                    return Err(IllFormed::CurryOptMergeConflict(Some(*arg), overlap));
                }
            }
            Ok(Self {
                wildcard: Some(wild),
                none,
                some,
            })
        } else {
            Ok(Self {
                wildcard,
                none,
                some,
            })
        }
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
    /// Iterate over keys only, ignoring values.
    #[inline]
    pub fn keys_without_wildcard(&self) -> impl Iterator<Item = Option<&Arg>> {
        (if self.none.is_some() {
            &[None][..]
        } else {
            &[]
        })
        .iter()
        .copied()
        .chain(self.some.keys().map(Some))
    }

    /// Iterate over values only, ignoring keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Etc> {
        self.wildcard
            .iter()
            .chain(self.none.iter())
            .chain(self.some.values())
    }

    /// Iterate over values only, WITHOUT a wildcard value, ignoring keys.
    #[inline]
    pub fn values_without_wildcard(&self) -> impl Iterator<Item = &Etc> {
        self.none.iter().chain(self.some.values())
    }

    /// If there's no wildcard, look elsewhere.
    #[inline]
    fn get_if_no_wildcard(&self, head: Option<&Arg>, tail: Etc::Key<'_>) -> Option<&Etc::Value> {
        head.map_or(self.none.as_ref().and_then(|etc| etc.get(tail)), |some| {
            self.some.get(some).and_then(|etc| etc.get(tail))
        })
    }
}

impl<A: 'static + fmt::Debug + Clone + Ord, S: 'static + Copy + Ord, Ctrl: Indices<A, S>>
    CurryOpt<S, Curry<A, Return<Edge<A, S, Ctrl>>>>
{
    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[allow(clippy::never_loop)]
    #[cfg(any(test, feature = "quickcheck"))]
    pub(crate) fn deabsurdify(&mut self, size: NonZeroUsize) {
        if let Some(ref mut none) = self.none {
            none.deabsurdify(size);
        }
        if let Some(ref mut wild) = self.wildcard {
            wild.deabsurdify(size);
            'dont_delete_none: loop {
                'delete_none: loop {
                    if let Some(ref mut none) = self.none {
                        while let Some(overlap) = wild.disjoint(none) {
                            match overlap {
                                Some(ref not_wild) => none.remove(not_wild),
                                None => break 'delete_none,
                            }
                        }
                    }
                    break 'dont_delete_none;
                }
                self.none = None;
                break 'dont_delete_none;
            }
        }
        for etc in self.some.values_mut() {
            etc.deabsurdify(size);
            while let Some(overlap) = self
                .wildcard
                .as_ref()
                .and_then(|wc| wc.disjoint(etc))
                .or_else(|| self.none.as_ref().and_then(|none| none.disjoint(etc)))
            {
                etc.remove(
                    overlap
                        .as_ref()
                        .expect("Internal error: please open an issue on GitHub!"),
                );
            }
        }
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Match either (a) literally anything or (b) certain ranges of values.

use crate::{Edge, IllFormed, Indices, Lookup, Merge, Range, Return};
use core::{fmt, num::NonZeroUsize};

/// Match either (a) literally anything or (b) certain ranges of values.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Wildcard<Arg: fmt::Debug + Ord, Etc: Lookup> {
    /// Match literally anything.
    Any(Etc),
    /// Match specific ranges of values.
    Specific(Vec<(Range<Arg>, Etc)>),
}

impl<Arg: fmt::Debug + Ord, Etc: Lookup> fmt::Debug for Wildcard<Arg, Etc> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Any(ref etc) => write!(f, "Wildcard::Any({etc:?})"),
            Self::Specific(ref map) => {
                write!(f, "Wildcard::Specific({map:?}.into_iter().collect())")
            }
        }
    }
}

impl<Arg: 'static + fmt::Debug + Ord, Etc: Lookup> Lookup for Wildcard<Arg, Etc> {
    type Key<'k> = (&'k Arg, Etc::Key<'k>);
    type Value = Etc::Value;
    #[inline]
    fn get(&self, (arg, args): Self::Key<'_>) -> Option<&Self::Value> {
        match *self {
            Self::Any(ref etc) => etc.get(args),
            Self::Specific(ref map) => map
                .iter()
                .fold(None, |acc, &(ref k, ref v)| {
                    // TODO: binary search?
                    if k.contains(arg).is_eq() {
                        assert!(
                            acc.is_none(),
                            "`Wildcard` with overlapping ranges: e.g. on argument `{arg:?}`",
                        );
                        Some(v)
                    } else {
                        acc
                    }
                })
                .and_then(|etc| etc.get(args)),
        }
    }
    #[inline]
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, mut f: F) {
        match *self {
            Self::Any(ref mut etc) => etc.map_values(f),
            Self::Specific(ref mut map) => {
                for &mut (_, ref mut etc) in map {
                    etc.map_values(&mut f);
                }
            }
        }
    }
}

impl<
        A: 'static + fmt::Debug + Clone + Ord,
        S: 'static + fmt::Debug + Copy + Ord,
        Ctrl: Indices<A, S>,
    > Merge<A, S, Ctrl> for Wildcard<A, Return<Edge<A, S, Ctrl>>>
{
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        match (self, other) {
            (Self::Any(Return(lhs)), &Self::Any(Return(ref rhs))) => {
                Ok(Self::Any(Return(lhs.merge(rhs)?)))
            }
            (Self::Any(Return(lhs)), &Self::Specific(ref rhs)) => {
                if rhs.is_empty() {
                    Ok(Self::Any(Return(lhs)))
                } else {
                    Err(IllFormed::WildcardMergeConflict(
                        rhs.iter().map(|&(ref k, _)| k.clone()).collect(),
                    ))
                }
            }
            (Self::Specific(lhs), &Self::Any(Return(ref rhs))) => {
                if lhs.is_empty() {
                    Ok(Self::Any(Return(rhs.clone())))
                } else {
                    Err(IllFormed::WildcardMergeConflict(
                        lhs.into_iter().map(|(k, _)| k).collect(),
                    ))
                }
            }
            (Self::Specific(lhs), &Self::Specific(ref rhs)) => Ok(Self::Specific(lhs.merge(rhs)?)),
        }
    }
}

impl<
        A: 'static + fmt::Debug + Clone + Ord,
        S: 'static + fmt::Debug + Copy + Ord,
        Ctrl: Indices<A, S>,
    > Wildcard<A, Return<Edge<A, S, Ctrl>>>
{
    /// Check for structural errors.
    /// # Errors
    /// If this automaton is not well-formed.
    #[inline]
    pub fn check(&self, size: NonZeroUsize) -> Result<(), IllFormed<A, S, Ctrl>> {
        match *self {
            Self::Any(Return(ref edge)) => edge.check(size),
            Self::Specific(ref map) => {
                map.iter()
                    .enumerate()
                    .try_fold((), |(), (i, &(ref k, Return(ref edge)))| {
                        get!(map, ..i)
                            .iter()
                            .fold(None, |acc, &(ref range, _)| acc.or_else(|| range.union(k)))
                            .map_or(Ok(()), |union| Err(IllFormed::VecMergeConflict(union)))
                            .and_then(|()| edge.check(size))
                    })
            }
        }
    }

    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    pub fn deabsurdify(&mut self, size: Option<NonZeroUsize>) {
        match *self {
            Self::Any(Return(ref mut etc)) => {
                if let Some(s) = size {
                    etc.deabsurdify(s);
                }
            }
            Self::Specific(ref mut v) => {
                {
                    let mut rm = vec![];
                    for (i, &(ref k, _)) in v.iter().enumerate() {
                        if get!(v, ..i).iter().any(|&(ref range, _)| range.overlap(k)) {
                            rm.push(i);
                        }
                    }
                    for i in rm.into_iter().rev() {
                        drop(v.swap_remove(i));
                    }
                }
                if let Some(s) = size {
                    for &mut (_, Return(ref mut edge)) in v {
                        edge.deabsurdify(s);
                    }
                }
            }
        }
    }

    /// Remove a value by its key.
    /// # Panics
    /// If this is a wildcard.
    #[inline]
    #[allow(clippy::panic)]
    pub fn remove(&mut self, key: &Range<A>) {
        match *self {
            Self::Any(..) => panic!("Trying to remove a key from a wildcard"),
            Self::Specific(ref mut v) => {
                drop(v.swap_remove(unwrap!(v.iter().position(|&(ref k, _)| k == key))));
            }
        }
    }

    /// Iterate over values only, ignoring keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Edge<A, S, Ctrl>> {
        match *self {
            Self::Any(Return(ref etc)) => vec![etc],
            Self::Specific(ref v) => v.iter().map(|&(_, Return(ref edge))| edge).collect(),
        }
        .into_iter()
    }
}

impl<
        A: 'static + fmt::Debug + Clone + Ord,
        S: 'static + fmt::Debug + Copy + Ord,
        Ctrl: Indices<A, S>,
    > Wildcard<A, Return<Edge<A, S, Ctrl>>>
{
    /// Find any key in common if any exist.
    #[inline]
    pub fn disjoint(&self, other: &Self) -> Option<Option<Range<A>>> {
        match (self, other) {
            (&Self::Any(Return(ref lhs)), &Self::Any(Return(ref rhs))) => {
                (lhs != rhs).then_some(None)
            }
            (&Self::Any(..), &Self::Specific(ref v)) | (&Self::Specific(ref v), &Self::Any(..)) => {
                v.first().map(|&(ref k, _)| Some(k.clone()))
            }
            (&Self::Specific(ref lhs), &Self::Specific(ref rhs)) => {
                lhs.iter().fold(None, |acc, &(ref lk, _)| {
                    acc.or_else(|| {
                        rhs.iter()
                            .any(|&(ref rk, _)| lk == rk)
                            .then(|| Some(lk.clone()))
                    })
                })
            }
        }
    }
}

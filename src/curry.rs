/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from a potential wildcard to _another map_.

use crate::{Edge, Indices, Lookup, Merge, Range, Return};
use core::{fmt, iter::*, option, slice::Iter};
use std::vec::IntoIter;

#[cfg(any(test, feature = "quickcheck"))]
use core::num::NonZeroUsize;

/// Map from a potential wildcard to _another map_.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Curry<Arg: Ord, Etc: Lookup> {
    /// First, try to match this, no matter what the argument was.
    pub wildcard: Option<Etc>,
    /// If the wildcard match didn't work, try this.
    pub specific: Vec<(Range<Arg>, Etc)>,
}

impl<Arg: Ord, Etc: Lookup> Default for Curry<Arg, Etc> {
    #[inline]
    fn default() -> Self {
        Self {
            wildcard: None,
            specific: vec![],
        }
    }
}

impl<Arg: fmt::Debug + Ord, Etc: fmt::Debug + Lookup> fmt::Debug for Curry<Arg, Etc> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Curry {{ wildcard: {:?}, specific: {:?}.into_iter().collect() }}",
            self.wildcard,
            self.specific.iter().collect::<Vec<_>>(),
        )
    }
}

impl<Arg: 'static + Ord, Etc: 'static + Lookup> Lookup for Curry<Arg, Etc> {
    type Key<'k> = (&'k Arg, Etc::Key<'k>);
    type Value = Etc::Value;
    #[inline]
    #[allow(clippy::unwrap_in_result)]
    fn get(&self, key: Self::Key<'_>) -> Option<&Self::Value> {
        let (head, tail) = key;
        if let woohoo @ Some(_) = self.wildcard.as_ref().and_then(|etc| etc.get(tail)) {
            #[allow(clippy::panic)]
            #[cfg(any(test, debug_assertions))]
            {
                let _ = self
                    .specific
                    .binary_search_by(|&(ref range, ref etc)| {
                        let cmp = range.contains(head);
                        #[allow(clippy::manual_assert)]
                        if cmp.is_eq() && etc.get(tail).is_some() {
                            panic!("Duplicate value as both a wildcard and a non-wildcard");
                        }
                        cmp
                    })
                    .ok()?;
            }
            woohoo
        } else {
            // TODO: break this apart and simplify it to remove throwing away a value then indexing to get it back again
            let i = self
                .specific
                .binary_search_by(|&(ref range, _)| range.contains(head))
                .ok()?;
            self.specific.get(i).and_then(|&(_, ref etc)| etc.get(tail))
        }
    }
    #[inline]
    fn map_values<F: FnMut(&mut Self::Value)>(&mut self, mut f: F) {
        if let Some(ref mut wild) = self.wildcard {
            wild.map_values(&mut f);
        }
        for &mut (_, ref mut v) in &mut self.specific {
            v.map_values(&mut f);
        }
    }
}

impl<A: 'static + Clone + Ord, S: 'static + Copy + Ord, Ctrl: Indices<A, S>> Merge<A, S, Ctrl>
    for Curry<A, Return<Edge<A, S, Ctrl>>>
{
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, crate::IllFormed<A, S, Ctrl>> {
        Ok(Self {
            wildcard: self.wildcard.merge(&other.wildcard)?,
            specific: self.specific.merge(&other.specific)?,
        })
    }
}

impl<Arg: Ord, Etc: Lookup> IntoIterator for Curry<Arg, Etc> {
    type Item = (Option<Range<Arg>>, Etc);
    type IntoIter = Chain<
        Map<option::IntoIter<Etc>, fn(Etc) -> (Option<Range<Arg>>, Etc)>,
        Map<IntoIter<(Range<Arg>, Etc)>, fn((Range<Arg>, Etc)) -> (Option<Range<Arg>>, Etc)>,
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

impl<Arg: Ord, Etc: Lookup> Curry<Arg, Etc> {
    /// Iterate over values only, ignoring keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Etc> {
        self.wildcard
            .iter()
            .chain(self.specific.iter().map(|&(_, ref etc)| etc))
    }

    /// Remove a value by its key.
    #[inline]
    pub fn remove(&mut self, key: &Range<Arg>) {
        drop(
            self.specific
                .iter()
                .position(|&(ref k, _)| k == key)
                .map(|i| self.specific.remove(i)),
        );
    }
}

impl<A: 'static + Clone + Ord, S: 'static + Copy + Ord, Ctrl: Indices<A, S> + PartialEq>
    Curry<A, Return<Edge<A, S, Ctrl>>>
{
    /// Find any value in common if any exist.
    #[inline]
    pub fn disjoint(&self, other: &Self) -> Option<Option<Range<A>>> {
        #[allow(clippy::else_if_without_else)]
        if let Some(Return(ref wild)) = self.wildcard {
            match other.wildcard {
                Some(Return(ref rhs_wild)) => {
                    if wild == rhs_wild {
                        return Some(None);
                    }
                }
                None => {
                    for &(ref k, Return(ref v)) in &self.specific {
                        if wild == v {
                            return Some(Some(k.clone()));
                        }
                    }
                }
            }
        } else if let Some(Return(ref wild)) = other.wildcard {
            for &(ref k, Return(ref v)) in &self.specific {
                if wild == v {
                    return Some(Some(k.clone()));
                }
            }
        }
        for &(ref k, _) in &self.specific {
            if let Some(union) = other
                .specific
                .iter()
                .fold(None, |acc, &(ref rk, _)| acc.or_else(|| rk.union(k)))
            {
                return Some(Some(union));
            }
        }
        None
    }

    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(any(test, feature = "quickcheck"))]
    pub(crate) fn deabsurdify(&mut self, size: NonZeroUsize) {
        if let Some(Return(ref mut edge)) = self.wildcard {
            edge.deabsurdify(size);
            for i in self
                .specific
                .iter()
                .enumerate()
                .fold(vec![], |mut acc, (i, &(_, Return(ref v)))| {
                    if edge == v {
                        acc.push(i);
                    }
                    acc
                })
                .into_iter()
                .rev()
            {
                drop(self.specific.remove(i));
            }
        }
        for &mut (_, Return(ref mut etc)) in &mut self.specific {
            etc.deabsurdify(size);
        }
    }
}

impl<'a, Arg: Ord, Etc: Lookup> IntoIterator for &'a Curry<Arg, Etc> {
    type Item = (Option<&'a Range<Arg>>, &'a Etc);
    type IntoIter = Chain<
        Map<option::Iter<'a, Etc>, fn(&'a Etc) -> (Option<&'a Range<Arg>>, &'a Etc)>,
        Map<
            Iter<'a, (Range<Arg>, Etc)>,
            fn(&'a (Range<Arg>, Etc)) -> (Option<&'a Range<Arg>>, &'a Etc),
        >,
    >;
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::as_conversions, clippy::as_underscore, trivial_casts)]
    fn into_iter(self) -> Self::IntoIter {
        self.wildcard.iter().map((|some| (None, some)) as _).chain(
            self.specific
                .iter()
                .map((|&(ref k, ref v)| (Some(k), v)) as _),
        )
    }
}

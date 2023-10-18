/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::{Edge, IllFormed, Indices, Range, Return, Wildcard};
use core::{borrow::Borrow, fmt};
use std::collections::{BTreeMap, BTreeSet};

/// Trait to fallibly combine multiple values into one value with identical semantics.
pub trait Merge<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>>:
    Sized
{
    /// Fallibly combine multiple values into one value with identical semantics.
    /// # Errors
    /// Implementation-defined: if the merge as we define it can't work.
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>>;
}

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>> Merge<A, S, Ctrl>
    for usize
{
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        if self == *other {
            Ok(self)
        } else {
            Err(IllFormed::IndexMergeConflict(self, *other))
        }
    }
}

impl<A: fmt::Debug + Clone + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>> Merge<A, S, Ctrl>
    for Option<Return<Edge<A, S, Ctrl>>>
{
    #[inline(always)]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        Ok(match (self, other) {
            (None, &None) => None,
            (Some(a), &None) => Some(a),
            (None, &Some(ref b)) => Some(b.clone()),
            (Some(a), &Some(ref b)) => Some(a.merge(b)?),
        })
    }
}

impl<
        A: 'static + fmt::Debug + Clone + Ord,
        S: 'static + fmt::Debug + Copy + Ord,
        Ctrl: Indices<A, S>,
    > Merge<A, S, Ctrl> for Option<Wildcard<A, Return<Edge<A, S, Ctrl>>>>
{
    #[inline(always)]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        Ok(match (self, other) {
            (None, &None) => None,
            (Some(a), &None) => Some(a),
            (None, &Some(ref b)) => Some(b.clone()),
            (Some(a), &Some(ref b)) => Some(a.merge(b)?),
        })
    }
}

impl<A: fmt::Debug + Clone + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>> Merge<A, S, Ctrl>
    for Vec<(Range<A>, Return<Edge<A, S, Ctrl>>)>
{
    #[inline(always)]
    fn merge(mut self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        for &(ref rk, Return(ref rv)) in other {
            for &(ref lk, _) in &self {
                if let Some(union) = lk.union(rk) {
                    return Err(IllFormed::VecMergeConflict(union));
                }
            }
            self.push((rk.clone(), Return(rv.clone())));
        }
        Ok(self)
    }
}

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord> Merge<A, S, BTreeSet<usize>>
    for BTreeSet<usize>
{
    #[inline(always)]
    fn merge(mut self, other: &Self) -> Result<Self, IllFormed<A, S, BTreeSet<usize>>> {
        self.extend(other.iter().copied());
        Ok(self)
    }
}

impl<
        A: 'static + fmt::Debug + Clone + Ord,
        S: 'static + fmt::Debug + Copy + Ord,
        Ctrl: Indices<A, S>,
    > Merge<A, S, Ctrl> for BTreeMap<S, Wildcard<A, Return<Edge<A, S, Ctrl>>>>
{
    #[inline(always)]
    fn merge(mut self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        for (k, v) in other {
            if let Some(pre_v) = self.insert(*k, v.clone()) {
                return Err(IllFormed::MapMergeConflict(*k, pre_v, v.clone()));
            }
        }
        Ok(self)
    }
}

/// Merge an entire iterator into a value.
#[inline]
pub fn merge<
    A: fmt::Debug + Ord,
    S: fmt::Debug + Copy + Ord,
    Ctrl: Indices<A, S>,
    M: Clone + Merge<A, S, Ctrl>,
    I: IntoIterator,
>(
    i: I,
) -> Option<Result<M, IllFormed<A, S, Ctrl>>>
where
    I::Item: Borrow<M>,
{
    let mut iter = i.into_iter();
    let first = iter.next()?;
    Some(iter.try_fold(first.borrow().clone(), |acc, m| acc.merge(m.borrow())))
}

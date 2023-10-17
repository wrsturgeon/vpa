/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Calls both now and, in textual form, in the autogenerated source code.

use crate::{IllFormed, Indices, Merge};
use core::{cmp, fmt};

/// Both a function pointer and a source-code representation.
#[derive(Clone, Hash)]
#[allow(clippy::derived_hash_with_manual_eq, clippy::exhaustive_structs)]
pub struct Call<I, O> {
    /// Function pointer.
    pub ptr: fn(I) -> O,
    /// Source-code representation.
    pub src: String,
}

impl<I, O> PartialEq for Call<I, O> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src
    }
}

impl<I, O> Eq for Call<I, O> {}

impl<I, O> Ord for Call<I, O> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.src.cmp(&other.src)
    }
}

impl<I, O> PartialOrd for Call<I, O> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I, O> fmt::Debug for Call<I, O> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "call!({})", self.src)
    }
}

impl<A: Ord, S: Copy + Ord, Ctrl: Indices<A, S>, I, O> Merge<A, S, Ctrl> for Call<I, O> {
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
        if self == *other {
            Ok(self)
        } else {
            Err(IllFormed::CallMergeConflict(self.src, other.src.clone()))
        }
    }
}

impl<I, O> Call<I, O> {
    /// Construct a new `Call` from a function pointer and a source-code representation.
    #[inline(always)]
    pub const fn new(ptr: fn(I) -> O, src: String) -> Self {
        Self { ptr, src }
    }
}

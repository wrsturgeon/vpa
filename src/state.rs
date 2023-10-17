/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A state in a visibly pushdown automaton.

use crate::{Curry, CurryOpt, Edge, Indices, Merge, Return};
use core::fmt;

#[cfg(any(test, feature = "quickcheck"))]
use core::num::NonZeroUsize;

/// A state in a visibly pushdown automaton.
#[allow(clippy::exhaustive_structs, clippy::type_complexity)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<A: 'static + fmt::Debug + Ord, S: 'static + Copy + Ord, Ctrl: Indices<A, S>> {
    /// State transitions.
    pub transitions: CurryOpt<S, Curry<A, Return<Edge<A, S, Ctrl>>>>,
    /// Whether an automaton in this state should accept when input ends.
    pub accepting: bool,
}

impl<A: fmt::Debug + Ord, S: Copy + Ord, Ctrl: Indices<A, S>> Default for State<A, S, Ctrl> {
    #[inline]
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        Self {
            transitions: Default::default(),
            accepting: false,
        }
    }
}

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord, Ctrl: fmt::Debug + Indices<A, S>> fmt::Debug
    for State<A, S, Ctrl>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "State {{ transitions: {:?}, accepting: {:?} }}",
            self.transitions, self.accepting,
        )
    }
}

impl<A: fmt::Debug + Clone + Ord, S: Copy + Ord, Ctrl: Indices<A, S>> Merge<A, S, Ctrl>
    for State<A, S, Ctrl>
{
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, crate::IllFormed<A, S, Ctrl>> {
        Ok(Self {
            transitions: self.transitions.merge(&other.transitions)?,
            accepting: self.accepting || other.accepting,
        })
    }
}

impl<A: fmt::Debug + Clone + Ord, S: Copy + Ord, Ctrl: Indices<A, S>> State<A, S, Ctrl> {
    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(any(test, feature = "quickcheck"))]
    pub(crate) fn deabsurdify(&mut self, size: NonZeroUsize) {
        self.transitions.deabsurdify(size);
    }
}

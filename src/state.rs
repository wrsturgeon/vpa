/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A state in a visibly pushdown automaton.

use crate::{Curry, CurryOpt, Edge, Indices, Return};

#[cfg(feature = "quickcheck")]
use {
    crate::Lookup,
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// A state in a visibly pushdown automaton.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<A: 'static + Ord, S: 'static + Ord, Ctrl: 'static + Indices> {
    /// State transitions.
    pub transitions: CurryOpt<S, Curry<A, Return<Edge<S, Ctrl>>>>,
    /// Whether an automaton in this state should accept when input ends.
    pub accepting: bool,
}

impl<A: Ord, S: Ord, Ctrl: Indices> State<A, S, Ctrl> {
    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(feature = "quickcheck")]
    pub(crate) fn deabsurdify(&mut self, size: NonZeroUsize) {
        self.transitions.map_values(|edge| edge.deabsurdify(size));
    }
}

#[cfg(feature = "quickcheck")]
impl<A: Arbitrary + Ord, S: Arbitrary + Ord, Ctrl: 'static + Arbitrary + Indices> Arbitrary
    for State<A, S, Ctrl>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            transitions: Arbitrary::arbitrary(g),
            accepting: Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((self.transitions.clone(), self.accepting).shrink().map(
            |(transitions, accepting)| Self {
                transitions,
                accepting,
            },
        ))
    }
}

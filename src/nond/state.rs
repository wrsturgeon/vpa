/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A state in a visibly pushdown automaton.

use crate::{nond::edge::Edge, Alphabet};
use std::collections::BTreeMap;

#[cfg(any(test, debug_assertions))]
use crate::Kind;

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// A state in a visibly pushdown automaton.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct State<A: Alphabet> {
    /// State transitions.
    pub(crate) transitions: BTreeMap<A, Edge>,
    /// Whether an automaton in this state should accept when input ends.
    pub(crate) accepting: bool,
}

impl<A: Alphabet> State<A> {
    /// Ensure that each local token causes a local transition and so on.
    #[inline]
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn check_consistency(&self) -> Result<(), (usize, Kind, Kind)> {
        for (i, (token, edge)) in self.transitions.iter().enumerate() {
            let tk = token.kind();
            let ek = edge.kind;
            if tk != ek {
                return Err((i, tk, ek));
            }
        }
        Ok(())
    }

    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(feature = "quickcheck")]
    pub(crate) fn deabsurdify(self, size: NonZeroUsize) -> Self {
        Self {
            transitions: self
                .transitions
                .into_iter()
                .map(|(token, edge)| (token, edge.deabsurdify(size)))
                .collect(),
            ..self
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<A: Alphabet + Arbitrary> Arbitrary for State<A> {
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

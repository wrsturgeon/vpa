/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A state in a visibly pushdown automaton.

use crate::{Edge, Indices};
use std::collections::BTreeMap;

#[cfg(any(test, debug_assertions))]
use crate::Kind;

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// A state in a visibly pushdown automaton.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<A: Ord, Ctrl: Indices> {
    /// State transitions.
    pub transitions: BTreeMap<A, Edge<Ctrl>>,
    /// Whether an automaton in this state should accept when input ends.
    pub accepting: bool,
}

impl<A: Ord, Ctrl: Indices> State<A, Ctrl> {
    /// Ensure that each local token causes a local transition and so on.
    #[inline]
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn check_consistency(&self) -> Result<(), (usize, Kind, Kind)> {
        use std::collections::btree_map::Entry;

        let mut map = BTreeMap::new();
        for (i, (token, edge)) in self.transitions.iter().enumerate() {
            let edge_kind = edge.kind();
            match map.entry(token) {
                Entry::Vacant(vacant) => drop(vacant.insert(edge_kind)),
                Entry::Occupied(occupied) => {
                    let kind = *occupied.get();
                    if kind != edge_kind {
                        return Err((i, kind, edge_kind));
                    }
                }
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
impl<A: Arbitrary + Ord, Ctrl: 'static + Arbitrary + Indices> Arbitrary for State<A, Ctrl> {
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

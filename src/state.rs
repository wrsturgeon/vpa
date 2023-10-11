/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A state in a visibly pushdown automaton.

use crate::{edge::Edge, Alphabet, Kind};
use std::collections::BTreeMap;

#[cfg(feature = "quickcheck")]
use quickcheck::{Arbitrary, Gen};

/// A state in a visibly pushdown automaton.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct State<A: Alphabet> {
    /// State transitions.
    transitions: BTreeMap<A, Edge>,
}

impl<A: Alphabet> State<A> {
    /// Ensure that each local token causes a local transition and so on.
    #[inline]
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn check_consistency(&self) -> Result<(), (usize, Kind, Kind)> {
        for (i, (token, edge)) in self.transitions.iter().enumerate() {
            let tk = token.kind();
            let ek = edge.kind();
            if tk != ek {
                return Err((i, tk, ek));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "quickcheck")]
impl<A: Alphabet + Arbitrary> Arbitrary for Automaton<A> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            transitions: Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        todo!()
    }
}

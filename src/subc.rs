/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Subset construction algorithm for determinizing nondeterministic automata.

// TODO:

/*

use crate::{merge, Deterministic, IllFormed, Nondeterministic, State};
use core::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

impl<A: Clone + Ord, S: Copy + Ord> Nondeterministic<A, S> {
    /// Turn an iterator over indices into an iterator over references to states.
    #[inline]
    fn get_states<I: IntoIterator>(
        &self,
        i: I,
    ) -> impl Iterator<Item = &State<A, S, BTreeSet<usize>>>
    where
        I::Item: Borrow<usize>,
    {
        i.into_iter().map(|j| get!(self.states, *j.borrow()))
    }

    /// Subset construction algorithm for determinizing nondeterministic automata.
    /// # Errors
    /// If there's an ambiguity (that would have crashed the nondeterministic automaton anyway).
    #[inline]
    pub fn determinize(&self) -> Result<Deterministic<A, S>, IllFormed> {
        let current = self.initial.clone();
        let mut subsets_as_states: BTreeMap<BTreeSet<usize>, State<A, S, BTreeSet<usize>>> =
            BTreeMap::new();
        subsets_as_states.insert(
            current.clone(),
            merge(self.get_states(current)).unwrap_or_else(|| todo!())?,
        );
        todo!()
    }
}

*/

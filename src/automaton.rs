/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

use crate::{Execute, Indices, Lookup, State};
use std::collections::BTreeSet;

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// Deterministic visibly pushdown automaton: each token causes exactly one transition.
pub type Deterministic<A, S> = Automaton<A, S, usize>;
/// Deterministic visibly pushdown automaton: each token can cause many transitions, and if any accept, the automaton accepts.
pub type Nondeterministic<A, S> = Automaton<A, S, BTreeSet<usize>>;

/// Visibly pushdown automaton containing all states.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Automaton<A: 'static + Ord, S: 'static + Ord, Ctrl: 'static + Indices> {
    /// Every state in the automaton.
    pub states: Vec<State<A, S, Ctrl>>,
    /// Index of the state of the machine before parsing any input.
    pub initial: Ctrl,
}

impl<A: 'static + Ord, S: 'static + Ord, Ctrl: Indices> Execute<A, S> for Automaton<A, S, Ctrl> {
    type Ctrl = Ctrl;
    #[inline]
    fn initial(&self) -> Self::Ctrl {
        self.initial.clone()
    }
    #[inline]
    fn step(
        &self,
        ctrl: Self::Ctrl,
        stack_top: Option<&S>,
        maybe_token: Option<&A>,
    ) -> Result<Self::Ctrl, bool> {
        let mut states = ctrl.iter().map(|&i| get!(self.states, i));
        match maybe_token {
            None => Err(states.any(|s| s.accepting)),
            Some(token) => Ctrl::collect(
                states
                    .filter_map(|s| s.transitions.get((stack_top, (token, ()))))
                    .flat_map(|edge| edge.dst().iter().copied()),
            ),
        }
    }
}

impl<A: Ord, S: Ord, Ctrl: Indices> Automaton<A, S, Ctrl> {
    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(feature = "quickcheck")]
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn deabsurdify(&mut self) {
        let size =
            NonZeroUsize::new(self.states.len()).expect("Zero-state automaton: can't do anything");
        for state in &mut self.states {
            state.deabsurdify(size);
        }
        self.initial.map(|i| *i = *i % size);
    }
}

#[cfg(feature = "quickcheck")]
impl<A: Arbitrary + Ord, S: Arbitrary + Ord, Ctrl: 'static + Arbitrary + Indices> Arbitrary
    for Automaton<A, S, Ctrl>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        loop {
            let states = Vec::arbitrary(g);
            if states.is_empty() {
                continue;
            }
            #[allow(unsafe_code)]
            // SAFETY: Just checked above to be non-empty.
            let size = unsafe { NonZeroUsize::new_unchecked(states.len()) };
            let mut initial = Ctrl::arbitrary(g);
            #[allow(clippy::arithmetic_side_effects)] // <-- false positive
            initial.map(|i| *i = *i % size);
            return Self { states, initial };
        }
    }
    #[inline]
    #[allow(unsafe_code)]
    #[allow(clippy::arithmetic_side_effects)]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.states.clone(), self.initial.clone())
                .shrink()
                .filter(|&(ref states, _)| !states.is_empty())
                .map(|(states, initial)| {
                    let mut acc = Self { states, initial };
                    acc.deabsurdify();
                    acc
                }),
        )
    }
}

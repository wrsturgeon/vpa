/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

use crate::{exec::Execute, indices::Indices, state::State, Alphabet};
use std::collections::BTreeSet;

#[cfg(any(test, debug_assertions))]
use crate::Kind;

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// Deterministic visibly pushdown automaton: each token causes exactly one transition.
pub type Deterministic<A> = Automaton<A, usize>;
/// Deterministic visibly pushdown automaton: each token can cause many transitions, and if any accept, the automaton accepts.
pub type Nondeterministic<A> = Automaton<A, BTreeSet<usize>>;

/// Visibly pushdown automaton containing all states.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Automaton<A: Alphabet, Ctrl: Indices> {
    /// Every state in the automaton.
    pub(crate) states: Vec<State<A, Ctrl>>,
    /// Index of the state of the machine before parsing any input.
    pub(crate) initial: Ctrl,
}

impl<A: Alphabet, Ctrl: Indices> Execute<A> for Automaton<A, Ctrl> {
    type Ctrl = Ctrl;
    #[inline]
    fn initial(&self) -> Self::Ctrl {
        self.initial.clone()
    }
    #[inline]
    fn step(&self, ctrl: Self::Ctrl, maybe_token: Option<&A>) -> Result<Self::Ctrl, bool> {
        let mut states = ctrl.iter().map(|&i| get!(self.states, i));
        match maybe_token {
            None => Err(states.any(|s| s.accepting)),
            Some(token) => Ctrl::collect(
                states
                    .filter_map(|s| s.transitions.get(token))
                    .flat_map(|edge| edge.dst.iter().copied()),
            ),
        }
    }
    #[inline]
    #[cfg(any(test, debug_assertions))]
    fn check_consistency(&self) -> Result<(), (usize, Kind, Kind)> {
        for state in &self.states {
            state.check_consistency()?;
        }
        Ok(())
    }
}

impl<A: Alphabet, Ctrl: Indices> Automaton<A, Ctrl> {
    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(feature = "quickcheck")]
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn deabsurdify(self) -> Self {
        let size =
            NonZeroUsize::new(self.states.len()).expect("Zero-state automaton: can't do anything");
        Self {
            states: self
                .states
                .into_iter()
                .map(|s| s.deabsurdify(size))
                .collect(),
            initial: self.initial.map(|i| i % size),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<A: Alphabet + Arbitrary, Ctrl: 'static + Arbitrary + Indices> Arbitrary
    for Automaton<A, Ctrl>
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
            #[allow(clippy::arithmetic_side_effects)] // <-- false positive
            let initial = Ctrl::arbitrary(g).map(|i| i % size);
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
                .map(|(states, initial)| Self { states, initial }.deabsurdify()),
        )
    }
}

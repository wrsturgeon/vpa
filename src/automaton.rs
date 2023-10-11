/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

use crate::{exec::Execute, state::State, Alphabet};

#[cfg(any(test, debug_assertions))]
use crate::Kind;

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// Visibly pushdown automaton containing all states.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Automaton<A: Alphabet> {
    /// Every state in the automaton.
    pub(crate) states: Vec<State<A>>,
    /// Index of the state of the machine before parsing any input.
    pub(crate) initial: usize,
}

impl<A: Alphabet> Execute<A> for Automaton<A> {
    type Ctrl = usize;
    #[inline]
    fn initial(&self) -> Self::Ctrl {
        self.initial
    }
    #[inline]
    fn step(&self, ctrl: Self::Ctrl, maybe_token: Option<&A>) -> Result<Self::Ctrl, bool> {
        let state = get!(self.states, ctrl);
        maybe_token.map_or(Err(state.accepting), |token| {
            state
                .transitions
                .get(token)
                .map_or(Err(false), |edge| Ok(edge.dst))
        })
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

impl<A: Alphabet> Automaton<A> {
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
            initial: self.initial % size,
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<A: Alphabet + Arbitrary> Arbitrary for Automaton<A> {
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
            let initial = usize::arbitrary(g) % size;
            return Self { states, initial };
        }
    }
    #[inline]
    #[allow(unsafe_code)]
    #[allow(clippy::arithmetic_side_effects)]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.states.clone(), self.initial)
                .shrink()
                .filter(|&(ref states, _)| !states.is_empty())
                .map(|(states, initial)| Self { states, initial }.deabsurdify()),
        )
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

use crate::{exec::Execute, nond::state::State, Alphabet};
use std::collections::BTreeSet;

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
    pub(crate) initial: BTreeSet<usize>,
}

impl<A: Alphabet> Execute<A> for Automaton<A> {
    type Ctrl = BTreeSet<usize>;
    #[inline]
    fn initial(&self) -> Self::Ctrl {
        self.initial.clone()
    }
    #[inline]
    fn step(&self, ctrl: Self::Ctrl, maybe_token: Option<&A>) -> Result<Self::Ctrl, bool> {
        let mut states = ctrl.into_iter().map(|i| get!(self.states, i));
        match maybe_token {
            None => Err(states.any(|s| s.accepting)),
            Some(token) => Ok({
                let mut set = BTreeSet::new();
                for s in states {
                    if let Some(edge) = s.transitions.get(token) {
                        set.extend(edge.dst.iter().copied());
                    }
                }
                set
            }),
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
            initial: self.initial.into_iter().map(|i: usize| i % size).collect(),
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
            let initial = BTreeSet::arbitrary(g)
                .into_iter()
                .map(|i: usize| i % size)
                .collect();
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

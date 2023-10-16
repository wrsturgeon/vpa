/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

use crate::{Execute, IllFormed, Indices, Lookup, Merge, State};
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
pub struct Automaton<A: 'static + Ord, S: 'static + Copy + Ord, Ctrl: 'static + Indices> {
    /// Every state in the automaton.
    pub states: Vec<State<A, S, Ctrl>>,
    /// Index of the state of the machine before parsing any input.
    pub initial: Ctrl,
}

impl<A: 'static + Ord, S: 'static + Copy + Ord, Ctrl: Indices> Execute<A, S>
    for Automaton<A, S, Ctrl>
{
    type Ctrl = Ctrl;
    #[inline]
    fn initial(&self) -> Self::Ctrl {
        self.initial.clone()
    }
    #[inline]
    fn step(
        &self,
        ctrl: Self::Ctrl,
        stack: &mut Vec<S>,
        maybe_token: Option<&A>,
    ) -> Result<Result<Self::Ctrl, bool>, IllFormed> {
        let mut states = ctrl.iter().map(|&i| get!(self.states, i));
        let Some(token) = maybe_token else {
            return Ok(Err(stack.is_empty() && states.any(|s| s.accepting)));
        };
        let maybe_stack_top = stack.last();
        let mut edges = states.filter_map(|s| s.transitions.get((maybe_stack_top, (token, ()))));
        let Some(first_edge) = edges.next() else {
            return Ok(Err(false));
        };
        let init = Ok(first_edge.clone());
        let mega_edge = edges.fold(init, |r, e| r.and_then(|acc| acc.merge(e)))?;
        Ok(mega_edge.invoke(stack))
    }
}

impl<A: Ord, S: Copy + Ord, Ctrl: Indices> Automaton<A, S, Ctrl> {
    /// Run to completion and return whether or not the input was valid.
    /// # Errors
    /// If the parser itself is ill-formed and tries to take a nonsensical action.
    #[inline]
    #[allow(clippy::unreachable)]
    pub fn accept<I: IntoIterator<Item = A>>(&self, i: I) -> Result<bool, IllFormed> {
        use crate::Run;
        let mut run = i.into_iter().run(self);
        while run.next().is_some() {}
        run.ctrl
            .map(|r| if let Err(b) = r { b } else { unreachable!() })
    }

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
impl<A: Arbitrary + Ord, S: Arbitrary + Copy + Ord, Ctrl: 'static + Arbitrary + Indices> Arbitrary
    for Automaton<A, S, Ctrl>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        loop {
            let states = Vec::arbitrary(g);
            if states.is_empty() {
                continue;
            }
            let mut wip = Self {
                states,
                initial: Ctrl::arbitrary(g),
            };
            wip.deabsurdify();
            return wip;
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
                    let mut wip = Self { states, initial };
                    wip.deabsurdify();
                    wip
                }),
        )
    }
}

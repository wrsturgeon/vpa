/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Visibly pushdown automata.

use crate::{state::State, Alphabet, Kind};

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

impl<A: Alphabet> Automaton<A> {
    /// Ensure that each local token causes a local transition and so on.
    #[inline]
    #[cfg(any(test, debug_assertions))]
    pub(crate) fn check_consistency(&self) -> Result<(), (usize, Kind, Kind)> {
        for state in &self.states {
            state.check_consistency()?;
        }
        Ok(())
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
            let size = unsafe { NonZeroUsize::new_unchecked(states.len()) };
            let initial = usize::arbitrary(g) % size;
            Self { states, initial }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        todo!()
    }
}

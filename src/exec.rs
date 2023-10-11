/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execution of a visibly pushdown automaton on an input sequence.

use crate::{indices::Indices, Alphabet};
use core::mem::replace;

#[cfg(any(test, debug_assertions))]
use crate::Kind;

/// Any executable automaton.
pub trait Execute<A: Alphabet> {
    /// Record of control flow (usually a state or a set of states).
    type Ctrl: Indices;
    /// Initial control flow.
    #[must_use]
    fn initial(&self) -> Self::Ctrl;
    /// Read a token and update accordingly.
    /// # Errors
    /// If the automaton decides to accept or not to (check the Boolean).
    fn step(&self, ctrl: Self::Ctrl, maybe_token: Option<&A>) -> Result<Self::Ctrl, bool>;
    /// Ensure that each local token causes a local transition and so on.
    /// # Errors
    /// If a transition is inconsistent.
    #[cfg(any(test, debug_assertions))]
    fn check_consistency(&self) -> Result<(), (usize, Kind, Kind)>;
}

/// Execution of a visibly pushdown automaton on an input sequence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Execution<'a, A: Alphabet, E: Execute<A>, Iter: Iterator<Item = A>> {
    /// Reference to the automaton we're running.
    pub(crate) graph: &'a E,
    /// Input sequence as an iterator.
    pub(crate) iter: Iter,
    /// Current state in the automaton.
    pub(crate) ctrl: Result<E::Ctrl, bool>,
}

impl<A: Alphabet, E: Execute<A>, Iter: Iterator<Item = A>> Iterator for Execution<'_, A, E, Iter> {
    type Item = A;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.iter.next();
        self.ctrl = replace(&mut self.ctrl, Err(false))
            .and_then(|ctrl| self.graph.step(ctrl, maybe_token.as_ref()));
        maybe_token // <-- Propagate the iterator's input
    }
}

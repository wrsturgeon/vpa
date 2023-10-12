/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execution of a visibly pushdown automaton on an input sequence.

use crate::Indices;
use core::mem::replace;

/// Any executable automaton.
pub trait Execute<A: Ord, S: Ord> {
    /// Record of control flow (usually a state or a set of states).
    type Ctrl: Indices;
    /// Initial control flow.
    #[must_use]
    fn initial(&self) -> Self::Ctrl;
    /// Read a token and update accordingly.
    /// # Errors
    /// If the automaton decides to accept or not to (check the Boolean).
    fn step(
        &self,
        ctrl: Self::Ctrl,
        stack_top: Option<&S>,
        maybe_token: Option<&A>,
    ) -> Result<Self::Ctrl, bool>;
}

/// Execution of a visibly pushdown automaton on an input sequence.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Execution<'a, A: Ord, S: Ord, E: Execute<A, S>, Iter: Iterator<Item = A>> {
    /// Reference to the automaton we're running.
    pub graph: &'a E,
    /// Input sequence as an iterator.
    pub iter: Iter,
    /// Current state in the automaton.
    pub ctrl: Result<E::Ctrl, bool>,
    /// Current stack.
    pub stack: Vec<S>,
}

impl<A: Ord, S: Ord, E: Execute<A, S>, Iter: Iterator<Item = A>> Iterator
    for Execution<'_, A, S, E, Iter>
{
    type Item = A;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.iter.next();
        self.ctrl = replace(&mut self.ctrl, Err(false)).and_then(|ctrl| {
            self.graph
                .step(ctrl, self.stack.last(), maybe_token.as_ref())
        });
        maybe_token // <-- Propagate the iterator's input
    }
}

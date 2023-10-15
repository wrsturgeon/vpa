/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execution of a visibly pushdown automaton on an input sequence.

use crate::Indices;
use core::{fmt, mem::replace};

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
        stack: &mut Vec<S>,
        maybe_token: Option<&A>,
    ) -> Result<Result<Self::Ctrl, bool>, IllFormed>;
}

/// Ran an automaton that tried to take a nonsensical action.
/// TODO: Add fields to describe what went wrong.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IllFormed {
    /// Gave up after a set limit. Feel free to try again with a more permissive limit.
    TimedOut,
    /// Parsing ambiguity.
    /// TODO: Messages / data.
    Ambiguity,
}

/// Execution of a visibly pushdown automaton on an input sequence.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Execution<'a, A: Ord, S: Ord, E: Execute<A, S>, Iter: Iterator<Item = A>> {
    /// Reference to the automaton we're running.
    pub graph: &'a E,
    /// Input sequence as an iterator.
    pub iter: Iter,
    /// Current state in the automaton.
    pub ctrl: Result<Result<E::Ctrl, bool>, IllFormed>,
    /// Current stack.
    pub stack: Vec<S>,
}

impl<A: Ord, S: fmt::Debug + Ord, E: Execute<A, S>, Iter: Iterator<Item = A>> fmt::Debug
    for Execution<'_, A, S, E, Iter>
where
    E::Ctrl: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Execution {{ stack: {:?}, ctrl: {:?} }}",
            self.stack, self.ctrl,
        )
    }
}

impl<A: Ord, S: Ord, E: Execute<A, S>, Iter: Iterator<Item = A>> Iterator
    for Execution<'_, A, S, E, Iter>
{
    type Item = A;
    #[inline]
    #[allow(clippy::unwrap_in_result)]
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_token = self.iter.next();
        if matches!(self.ctrl, Ok(Ok(_))) {
            self.ctrl = self.graph.step(
                #[allow(unused_unsafe)] // the macro nests two `unsafe` blocks
                {
                    unwrap!(unwrap!(replace(&mut self.ctrl, Err(IllFormed::Ambiguity))))
                },
                &mut self.stack,
                maybe_token.as_ref(),
            );
        }
        maybe_token // <-- Propagate the iterator's input
    }
}

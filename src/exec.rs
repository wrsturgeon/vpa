/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execution of a visibly pushdown automaton on an input sequence.

use crate::{Curry, Edge, Indices, Return};
use core::{fmt, mem::replace};

/// Any executable automaton.
pub trait Execute<A: Ord, S: Copy + Ord> {
    /// Record of control flow (usually a state or a set of states).
    type Ctrl: Indices<A, S>;
    /// Initial control flow.
    #[must_use]
    fn initial(&self) -> Self::Ctrl;
    /// Read a token and update accordingly.
    /// # Errors
    /// If the automaton decides to accept or not to (check the Boolean).
    #[allow(clippy::type_complexity)]
    fn step(
        &self,
        ctrl: Self::Ctrl,
        stack: &mut Vec<S>,
        maybe_token: Option<&A>,
    ) -> Result<Result<Self::Ctrl, bool>, IllFormed<A, S, Self::Ctrl>>;
}

/// Ran an automaton that tried to take a nonsensical action.
/// TODO: Add fields to describe what went wrong.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IllFormed<A: 'static + Ord, S: 'static + Copy + Ord, Ctrl: Indices<A, S>> {
    /// Temporary value to be used with `core::mem::replace`.
    Temporary,
    /// Two different `usize`s trying to merge into a single `usize`.
    IndexMergeConflict(usize, usize),
    /// Same key mapped to different outputs in two `BTreeMap`s being merged.
    MapMergeConflict(
        S,
        Curry<A, Return<Edge<A, S, Ctrl>>>,
        Curry<A, Return<Edge<A, S, Ctrl>>>,
    ),
    /// Merging two incompatible edges.
    EdgeMergeConflict(Edge<A, S, Ctrl>, Edge<A, S, Ctrl>),
    /// Merging two incompatible calls.
    CallMergeConflict(String, String),
    /// Merging two incompatible stack symbols.
    PushMergeConflict(S, S),
    /// FIXME: phantom data.
    Bullshit(A, Ctrl),
}

/// Execution of a visibly pushdown automaton on an input sequence.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Execution<
    'a,
    A: 'static + fmt::Debug + Ord,
    S: 'static + fmt::Debug + Copy + Ord,
    E: Execute<A, S>,
    Iter: Iterator<Item = A>,
> {
    /// Reference to the automaton we're running.
    pub graph: &'a E,
    /// Input sequence as an iterator.
    pub iter: Iter,
    /// Current state in the automaton.
    #[allow(clippy::type_complexity)]
    pub ctrl: Result<Result<E::Ctrl, bool>, IllFormed<A, S, E::Ctrl>>,
    /// Current stack.
    pub stack: Vec<S>,
}

impl<
        A: fmt::Debug + Ord,
        S: fmt::Debug + Copy + Ord,
        E: Execute<A, S>,
        Iter: Iterator<Item = A>,
    > fmt::Debug for Execution<'_, A, S, E, Iter>
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

impl<
        A: fmt::Debug + Ord,
        S: fmt::Debug + Copy + Ord,
        E: Execute<A, S>,
        Iter: Iterator<Item = A>,
    > Iterator for Execution<'_, A, S, E, Iter>
where
    E::Ctrl: fmt::Debug,
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
                    unwrap!(unwrap!(replace(&mut self.ctrl, Err(IllFormed::Temporary))))
                },
                &mut self.stack,
                maybe_token.as_ref(),
            );
        }
        maybe_token // <-- Propagate the iterator's input
    }
}

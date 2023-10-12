/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to run a visibly pushdown automaton on an input sequence.

use crate::{Execute, Execution};

/// Trait to run a visibly pushdown automaton on an input sequence.
pub trait Run<A: Ord>: Iterator<Item = A> + Sized {
    /// Run a visibly pushdown automaton on this input sequence.
    #[must_use]
    fn run<S: Ord, E: Execute<A, S>>(self, graph: &E) -> Execution<'_, A, S, E, Self>;
}

impl<A: Ord, Iter: Iterator<Item = A>> Run<A> for Iter {
    #[inline]
    #[must_use]
    fn run<S: Ord, E: Execute<A, S>>(self, graph: &E) -> Execution<'_, A, S, E, Self> {
        Execution {
            graph,
            iter: self,
            ctrl: Ok(Ok(graph.initial())),
            stack: vec![],
        }
    }
}

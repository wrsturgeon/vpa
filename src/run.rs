/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to run a visibly pushdown automaton on an input sequence.

use crate::{exec::Execute, Alphabet, Execution};

/// Trait to run a visibly pushdown automaton on an input sequence.
pub trait Run<A: Alphabet>: Iterator<Item = A> + Sized {
    /// Run a visibly pushdown automaton on this input sequence.
    #[must_use]
    fn run<E: Execute<A>>(self, graph: &E) -> Execution<'_, A, E, Self>;
}

impl<A: Alphabet, Iter: Iterator<Item = A>> Run<A> for Iter {
    #[inline]
    #[must_use]
    fn run<E: Execute<A>>(self, graph: &E) -> Execution<'_, A, E, Self> {
        #[allow(clippy::panic)]
        #[cfg(any(test, debug_assertions))]
        if let Err((i, a, b)) = graph.check_consistency() {
            panic!("Internal error: state #{i} triggers a {b} transition on a {a} token");
        }
        Execution {
            graph,
            iter: self,
            ctrl: Ok(graph.initial()),
        }
    }
}

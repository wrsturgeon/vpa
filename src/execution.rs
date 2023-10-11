/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Execution of a visibly pushdown automaton on an input sequence.

use crate::{Alphabet, Automaton};

/// Execution of a visibly pushdown automaton on an input sequence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Execution<'vpa, A: Alphabet, Iter: Iterator<Item = A>> {
    /// Reference to the automaton we're running.
    pub(crate) vpa: &'vpa Automaton<A>,
    /// Input sequence as an iterator.
    pub(crate) iter: Iter,
    /// Current state in the automaton.
    pub(crate) state: usize,
}

impl<A: Alphabet, Iter: Iterator<Item = A>> Iterator for Execution<'_, A, Iter> {
    type Item = A;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.iter.next();
        todo!();
        token // <-- Propagate the iterator's input
    }
}

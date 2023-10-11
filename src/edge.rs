/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).

use crate::Kind;

/// Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum Edge {
    /// Transition that performs a stack push.
    Call {
        /// Index of the machine's state after this transition.
        dst: usize,
    },
    /// Transition that performs a stack pop.
    Return {
        /// Index of the machine's state after this transition.
        dst: usize,
    },
    /// Transition that performs neither a stack push nor a stack pop.
    Local {
        /// Index of the machine's state after this transition.
        dst: usize,
    },
}

impl Edge {
    /// Classify as a symbol (mostly to check consistency with the symbol that triggers it).
    #[inline(always)]
    pub(crate) const fn kind(&self) -> Kind {
        match *self {
            Self::Call { .. } => Kind::Call,
            Self::Return { .. } => Kind::Return,
            Self::Local { .. } => Kind::Local,
        }
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).

use crate::indices::Indices;

#[cfg(any(test, debug_assertions))]
use crate::Kind;

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Edge<Ctrl: Indices> {
    /// Index of the machine's state after this transition.
    pub(crate) dst: Ctrl,
    /// Function to call when compiled to a source file.
    /// - `Call   => state -> token ->                 state * stack`
    /// - `Return => state -> token -> stack option -> state`
    /// - `Local  => state -> token ->                 state`
    pub(crate) call: String,
    /// Classify as a symbol (mostly to check consistency with the symbol that triggers it).
    #[cfg(any(test, debug_assertions))]
    pub(crate) kind: Kind,
}

impl<Ctrl: Indices> Edge<Ctrl> {
    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(feature = "quickcheck")]
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn deabsurdify(self, size: NonZeroUsize) -> Self {
        Self {
            dst: self.dst.map(|i| i % size),
            ..self
        }
    }
}

#[cfg(feature = "quickcheck")]
impl<Ctrl: Arbitrary + Indices> Arbitrary for Edge<Ctrl> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            dst: Arbitrary::arbitrary(g),
            call: Arbitrary::arbitrary(g),
            #[cfg(any(test, debug_assertions))]
            kind: Kind::arbitrary(g),
        }
    }
    #[inline]
    #[cfg(any(test, debug_assertions))]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.dst.clone(), self.call.clone(), self.kind)
                .shrink()
                .map(|(dst, call, kind)| Self { dst, call, kind }),
        )
    }
    #[inline]
    #[cfg(not(any(test, debug_assertions)))]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.dst.clone(), self.call.clone())
                .shrink()
                .map(|(dst, call)| Self { dst, call }),
        )
    }
}

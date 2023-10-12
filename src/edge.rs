/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).

use crate::{Call, Indices};

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Edge<S: Ord, Ctrl: Indices> {
    /// Transition that causes a stack push.
    Call {
        /// Index of the machine's state after this transition.
        dst: Ctrl,
        /// Function to call when compiled to a source file.
        call: Call<(), ()>,
        /// Symbol to push onto the stack.
        push: S,
    },
    /// Transition that causes a stack pop.
    Return {
        /// Index of the machine's state after this transition.
        dst: Ctrl,
        /// Function to call when compiled to a source file.
        call: Call<(), ()>,
    },
    /// Transition that causes neither a stack push nor a stack pop.
    Local {
        /// Index of the machine's state after this transition.
        dst: Ctrl,
        /// Function to call when compiled to a source file.
        call: Call<(), ()>,
    },
}

impl<S: Ord, Ctrl: Indices> Edge<S, Ctrl> {
    /// Index of the machine's state after this transition.
    #[inline]
    pub const fn dst(&self) -> &Ctrl {
        match *self {
            Self::Call { ref dst, .. }
            | Self::Return { ref dst, .. }
            | Self::Local { ref dst, .. } => dst,
        }
    }

    /// Index of the machine's state after this transition.
    #[inline]
    pub fn dst_mut(&mut self) -> &mut Ctrl {
        match *self {
            Self::Call { ref mut dst, .. }
            | Self::Return { ref mut dst, .. }
            | Self::Local { ref mut dst, .. } => dst,
        }
    }

    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[cfg(feature = "quickcheck")]
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn deabsurdify(&mut self, size: NonZeroUsize) {
        let dst = self.dst_mut();
        *dst = unwrap!(Indices::collect(dst.iter().map(|&i| i % size)));
    }
}

#[cfg(feature = "quickcheck")]
impl<S: Arbitrary + Ord, Ctrl: Arbitrary + Indices> Arbitrary for Edge<S, Ctrl> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let f: [fn(&mut Gen) -> Self; 3] = [
            |r| Self::Call {
                dst: Arbitrary::arbitrary(r),
                call: Arbitrary::arbitrary(r),
                push: S::arbitrary(r),
            },
            |r| Self::Return {
                dst: Arbitrary::arbitrary(r),
                call: Arbitrary::arbitrary(r),
            },
            |r| Self::Local {
                dst: Arbitrary::arbitrary(r),
                call: Arbitrary::arbitrary(r),
            },
        ];
        unwrap!(g.choose(&f))(g)
    }
    #[inline]
    #[allow(clippy::shadow_unrelated)]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match *self {
            Self::Local { ref dst, ref call } => Box::new(
                (dst.clone(), call.clone())
                    .shrink()
                    .map(|(dst, call)| Self::Local { dst, call }),
            ),
            Self::Return { ref dst, ref call } => Box::new(
                (dst.clone(), call.clone())
                    .shrink()
                    .map(|(dst, call)| Self::Return { dst, call }),
            ),
            Self::Call {
                ref dst,
                ref call,
                ref push,
            } => Box::new(
                (dst.clone(), call.clone(), push.clone())
                    .shrink()
                    .map(|(dst, call, push)| Self::Call { dst, call, push }),
            ),
        }
    }
}

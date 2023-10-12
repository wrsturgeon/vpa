/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).

use crate::{Call, IllFormed, Indices, Merge};

#[cfg(feature = "quickcheck")]
use {
    core::num::NonZeroUsize,
    quickcheck::{Arbitrary, Gen},
};

/// Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Edge<S: Copy + Ord, Ctrl: Indices> {
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

impl<S: Copy + Ord, Ctrl: Indices> Merge for Edge<S, Ctrl> {
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed> {
        match (self, other) {
            (
                Self::Call {
                    dst: ldst,
                    call: lcall,
                    push: lpush,
                },
                &Self::Call {
                    dst: ref rdst,
                    call: ref rcall,
                    push: ref rpush,
                },
            ) => Ok(Self::Call {
                dst: ldst.merge(rdst)?,
                call: lcall.merge(rcall)?,
                push: if lpush == *rpush {
                    lpush
                } else {
                    return Err(IllFormed);
                },
            }),
            (
                Self::Return {
                    dst: ldst,
                    call: lcall,
                },
                &Self::Return {
                    dst: ref rdst,
                    call: ref rcall,
                },
            ) => Ok(Self::Return {
                dst: ldst.merge(rdst)?,
                call: lcall.merge(rcall)?,
            }),
            (
                Self::Local {
                    dst: ldst,
                    call: lcall,
                },
                &Self::Local {
                    dst: ref rdst,
                    call: ref rcall,
                },
            ) => Ok(Self::Return {
                dst: ldst.merge(rdst)?,
                call: lcall.merge(rcall)?,
            }),
            (_, _) => Err(IllFormed),
        }
    }
}

impl<S: Copy + Ord, Ctrl: Indices> Edge<S, Ctrl> {
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
    /// Take this edge in an actual execution. Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop the stack and it's empty.
    #[inline]
    pub fn invoke(self, stack: &mut Vec<S>) -> Result<Ctrl, bool> {
        match self {
            Self::Call {
                dst,
                call: _call,
                push,
            } => {
                stack.push(push);
                Ok(dst)
            }
            Self::Return { dst, call: _call } => stack.pop().map_or(Err(false), |_| Ok(dst)),
            Self::Local { dst, call: _call } => Ok(dst),
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
impl<S: Arbitrary + Copy + Ord, Ctrl: Arbitrary + Indices> Arbitrary for Edge<S, Ctrl> {
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
                push,
            } => Box::new(
                (dst.clone(), call.clone(), push)
                    .shrink()
                    .map(|(dst, call, push)| Self::Call { dst, call, push }),
            ),
        }
    }
}

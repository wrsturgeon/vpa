/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).

use crate::{Call, IllFormed, Indices, Merge};
use core::{convert::Infallible, fmt, marker::PhantomData, num::NonZeroUsize};

/// Edge in a visibly pushdown automaton (everything except the source state and the token that triggers it).
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Edge<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>> {
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
    /// Bullshit uninhabited state to typecheck the `<A>` parameter.
    Phantom(Infallible, PhantomData<A>),
}

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord, Ctrl: fmt::Debug + Indices<A, S>> fmt::Debug
    for Edge<A, S, Ctrl>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Call {
                ref dst,
                ref call,
                ref push,
            } => write!(
                f,
                "Edge::Call {{ dst: {:?}.into_iter().collect(), call: {call:?}, push: {push:?} }}",
                dst.iter().collect::<Vec<_>>(),
            ),
            Self::Return { ref dst, ref call } => {
                write!(
                    f,
                    "Edge::Return {{ dst: {:?}.into_iter().collect(), call: {call:?} }}",
                    dst.iter().collect::<Vec<_>>(),
                )
            }
            Self::Local { ref dst, ref call } => {
                write!(
                    f,
                    "Edge::Local {{ dst: {:?}.into_iter().collect(), call: {call:?} }}",
                    dst.iter().collect::<Vec<_>>(),
                )
            }
            Self::Phantom(..) => never!(),
        }
    }
}

impl<A: fmt::Debug + Clone + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>> Merge<A, S, Ctrl>
    for Edge<A, S, Ctrl>
{
    #[inline]
    fn merge(self, other: &Self) -> Result<Self, IllFormed<A, S, Ctrl>> {
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
                    push: rpush,
                },
            ) => Ok(Self::Call {
                dst: ldst.merge(rdst)?,
                call: lcall.merge(rcall)?,
                push: if lpush == rpush {
                    lpush
                } else {
                    return Err(IllFormed::PushMergeConflict(lpush, rpush));
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
            (lhs, rhs) => Err(IllFormed::EdgeMergeConflict(lhs, rhs.clone())),
        }
    }
}

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord, Ctrl: Indices<A, S>> Edge<A, S, Ctrl> {
    /// Index of the machine's state after this transition.
    #[inline]
    pub const fn dst(&self) -> &Ctrl {
        match *self {
            Self::Call { ref dst, .. }
            | Self::Return { ref dst, .. }
            | Self::Local { ref dst, .. } => dst,
            Self::Phantom(..) => never!(),
        }
    }

    /// Index of the machine's state after this transition.
    #[inline]
    pub fn dst_mut(&mut self) -> &mut Ctrl {
        match *self {
            Self::Call { ref mut dst, .. }
            | Self::Return { ref mut dst, .. }
            | Self::Local { ref mut dst, .. } => dst,
            Self::Phantom(..) => never!(),
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
            Self::Phantom(..) => never!(),
        }
    }

    /// Check for structural errors.
    /// # Errors
    /// If this automaton is not well-formed.
    #[inline]
    pub fn check(&self, size: NonZeroUsize) -> Result<(), IllFormed<A, S, Ctrl>> {
        if self.dst().iter().all(|&i| i < size.into()) {
            Ok(())
        } else {
            Err(IllFormed::OutOfBounds)
        }
    }

    /// Eliminate absurd relations like transitions to non-existing states.
    #[inline]
    #[allow(clippy::arithmetic_side_effects)]
    pub fn deabsurdify(&mut self, size: NonZeroUsize) {
        self.dst_mut().map(|i| *i = *i % size);
    }
}

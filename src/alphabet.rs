/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Alphabet partitioned into three disjoint sets: calls, returns, and locals.

use core::fmt;

/// What kind of symbol something is: call, return, or local.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// Token that causes a stack push.
    Call,
    /// Token that causes a stack pop.
    Return,
    /// Token that causes neither a stack push nor a stack pop.
    Local,
}

impl fmt::Display for Kind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Call => "call",
                Self::Return => "return",
                Self::Local => "local",
            }
        )
    }
}

/// Alphabet partitioned into three disjoint sets: calls, returns, and locals.
pub trait Alphabet: Ord {
    /// What kind of symbol this is: call, return, or local.
    #[must_use]
    fn kind(&self) -> Kind;
}

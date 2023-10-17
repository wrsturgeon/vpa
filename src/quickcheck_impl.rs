/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Implementations of `quickcheck::Arbitrary`.

use crate::*;
use quickcheck::{Arbitrary, Gen};
use std::collections::BTreeMap;

impl<
        A: Arbitrary + Ord,
        S: Arbitrary + Copy + Ord,
        Ctrl: 'static + Arbitrary + PartialEq + Indices,
    > Arbitrary for Automaton<A, S, Ctrl>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        loop {
            let states = Vec::arbitrary(g);
            if states.is_empty() {
                continue;
            }
            let mut wip = Self {
                states,
                initial: Ctrl::arbitrary(g),
            };
            wip.deabsurdify();
            return wip;
        }
    }
    #[inline]
    #[allow(unsafe_code)]
    #[allow(clippy::arithmetic_side_effects)]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.states.clone(), self.initial.clone())
                .shrink()
                .filter(|&(ref states, _)| !states.is_empty())
                .map(|(states, initial)| {
                    let mut wip = Self { states, initial };
                    wip.deabsurdify();
                    wip
                }),
        )
    }
}

impl<A: Arbitrary + Ord, S: Arbitrary + Copy + Ord, Ctrl: 'static + Arbitrary + Indices> Arbitrary
    for State<A, S, Ctrl>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            transitions: Arbitrary::arbitrary(g),
            accepting: Arbitrary::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((self.transitions.clone(), self.accepting).shrink().map(
            |(transitions, accepting)| Self {
                transitions,
                accepting,
            },
        ))
    }
}

impl<Arg: Arbitrary + Ord, Etc: Arbitrary + Lookup> Arbitrary for CurryOpt<Arg, Etc> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let wildcard: Option<_> = Arbitrary::arbitrary(g);
        let none: Option<_> = if wildcard.is_none() {
            Arbitrary::arbitrary(g)
        } else {
            None
        };
        Self {
            some: if wildcard.is_none() && none.is_none() {
                Arbitrary::arbitrary(g)
            } else {
                BTreeMap::new()
            },
            wildcard,
            none,
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.wildcard.clone(), self.none.clone(), self.some.clone())
                .shrink()
                .map(|(wildcard, none, some)| Self {
                    wildcard,
                    none,
                    some,
                }),
        )
    }
}

impl<Arg: Arbitrary + Ord, Etc: Arbitrary + Lookup> Arbitrary for Curry<Arg, Etc> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let wildcard: Option<_> = Arbitrary::arbitrary(g);
        Self {
            specific: if wildcard.is_none() {
                Arbitrary::arbitrary(g)
            } else {
                vec![]
            },
            wildcard,
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.wildcard.clone(), self.specific.clone())
                .shrink()
                .map(|(wildcard, specific)| Self { wildcard, specific }),
        )
    }
}

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

impl Arbitrary for Call<(), ()> {
    #[inline]
    #[allow(clippy::absolute_paths)]
    fn arbitrary(_: &mut Gen) -> Self {
        call!(::core::convert::identity)
    }
    // No shrinking.
}

impl<T: Arbitrary + Ord> Arbitrary for Range<T> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        let (a, b) = <(T, T)>::arbitrary(g);
        if a < b {
            Self { first: a, last: b }
        } else {
            Self { first: b, last: a }
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.first.clone(), self.last.clone())
                .shrink()
                .map(|(a, b)| {
                    if a < b {
                        Self { first: a, last: b }
                    } else {
                        Self { first: b, last: a }
                    }
                }),
        )
    }
}

impl<T: Arbitrary> Arbitrary for Return<T> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self(Arbitrary::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

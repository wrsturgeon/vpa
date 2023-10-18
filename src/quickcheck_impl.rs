/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Implementations of `quickcheck::Arbitrary`.

use crate::*;
use core::{convert::identity, fmt};
use quickcheck::{Arbitrary, Gen};
use std::collections::BTreeMap;

impl<
        A: fmt::Debug + Arbitrary + Ord,
        S: fmt::Debug + Arbitrary + Copy + Ord,
        Ctrl: 'static + Arbitrary + Indices<A, S>,
    > Arbitrary for Automaton<A, S, Ctrl>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            states: Vec::arbitrary(g),
            initial: Ctrl::arbitrary(g),
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
                .map(|(states, initial)| Self { states, initial }),
        )
    }
}

impl<
        A: fmt::Debug + Arbitrary + Ord,
        S: fmt::Debug + Arbitrary + Copy + Ord,
        Ctrl: 'static + Arbitrary + Indices<A, S>,
    > Arbitrary for State<A, S, Ctrl>
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

impl<
        A: 'static + fmt::Debug + Arbitrary + Ord,
        S: 'static + fmt::Debug + Arbitrary + Copy + Ord,
        Ctrl: Arbitrary + Indices<A, S>,
    > Arbitrary for Wildcard<A, Return<Edge<A, S, Ctrl>>>
{
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Self::Any(Arbitrary::arbitrary(g))
        } else {
            Self::Specific(Arbitrary::arbitrary(g))
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match *self {
            Self::Any(ref etc) => Box::new(etc.shrink().map(Self::Any)),
            Self::Specific(ref v) => Box::new(
                v.first()
                    .map(|&(_, ref x)| Self::Any(x.clone()))
                    .into_iter()
                    .chain(v.shrink().map(Self::Specific)),
            ),
        }
    }
}

impl<
        A: 'static + fmt::Debug + Clone + Ord,
        S: fmt::Debug + Arbitrary + Copy + Ord,
        Ctrl: Arbitrary + Indices<A, S>,
    > Arbitrary for Edge<A, S, Ctrl>
{
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
            Self::Phantom(..) => never!(),
        }
    }
}

impl Arbitrary for Call<(), ()> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            ptr: identity,
            src: String::arbitrary(g),
        }
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.src.shrink().map(|src| Self { ptr: identity, src }))
    }
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

impl<T: fmt::Debug + Arbitrary> Arbitrary for Return<T> {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        Self(Arbitrary::arbitrary(g))
    }
    #[inline]
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(Self))
    }
}

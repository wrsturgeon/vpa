/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::panic, clippy::unwrap_used, clippy::use_debug)]

use crate::*;
use core::{fmt, iter::once};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "quickcheck")]
use {core::time::Duration, tokio::time::timeout};

#[cfg(not(feature = "quickcheck"))]
use tokio as _;

#[cfg(feature = "quickcheck")]
const TIMEOUT: Duration = Duration::from_secs(1);

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::{quickcheck, Arbitrary, Gen, TestResult};
    use std::{env, panic};

    const INPUTS_PER_AUTOMATON: usize = 100;

    #[inline]
    fn subset_construction<K: Copy + fmt::Debug + Ord, S: fmt::Debug + Copy + Ord>(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) -> TestResult {
        let Ok(d) = nd.determinize() else {
            return TestResult::discard();
        };
        if nd.accept(input.iter().copied()).unwrap() != d.accept(input.iter().copied()).unwrap() {
            return TestResult::failed();
        }
        TestResult::passed()
    }

    /// Has to be manual since `quickcheck!` doesn't understand `async`
    #[allow(clippy::integer_division)]
    async fn subset_construction_timed<
        A: Arbitrary + fmt::Debug + Ord + panic::RefUnwindSafe + Send + Sync,
        S: Arbitrary + Copy + fmt::Debug + Ord + panic::RefUnwindSafe + Send + Sync,
    >() {
        let mut g = Gen::new(
            env::var("QUICKCHECK_GENERATOR_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        );
        for _ in 0_usize
            ..(env::var("QUICKCHECK_TESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100)
                / INPUTS_PER_AUTOMATON)
        {
            'discard: loop {
                let nd = timeout(TIMEOUT, async {
                    Nondeterministic::<A, S>::arbitrary(&mut g)
                })
                .await
                .expect("`Nondeterministic::arbitrary` timed out");
                let d = match timeout(TIMEOUT, async { nd.determinize() }).await {
                    Err(_timed_out) => return shrink_subset_construction(nd, None).await,
                    Ok(Err(_ill_formed)) => continue 'discard,
                    Ok(Ok(ok)) => ok,
                };
                for _ in 0..INPUTS_PER_AUTOMATON {
                    let input = Vec::arbitrary(&mut g);
                    if !check_subset_construction(&nd, &d, &input) {
                        return shrink_subset_construction(nd, Some(input)).await;
                    }
                }
                break 'discard;
            }
        }
    }

    quickcheck! {
        fn range_overlap_commutativity(a: Range<u8>, b: Range<u8>) -> bool {
            a.overlap(&b) == b.overlap(&a)
        }

        fn subset_construction_bool_bool(nd: Nondeterministic<bool, bool>, input: Vec<bool>) -> TestResult {
            subset_construction(&nd, &input)
        }

        fn subset_construction_bool_u8(nd: Nondeterministic<bool, u8>, input: Vec<bool>) -> TestResult {
            subset_construction(&nd, &input)
        }

        fn subset_construction_u8_bool(nd: Nondeterministic<u8, bool>, input: Vec<u8>) -> TestResult {
            subset_construction(&nd, &input)
        }

        fn subset_construction_u8_u8(nd: Nondeterministic<u8, u8>, input: Vec<u8>) -> TestResult {
            subset_construction(&nd, &input)
        }
    }

    /// Has to be manual since `quickcheck!` doesn't understand `async`
    #[tokio::test]
    #[allow(clippy::default_numeric_fallback, clippy::std_instead_of_core)]
    async fn determinization_stopwatch() {
        let mut g = Gen::new(
            env::var("QUICKCHECK_GENERATOR_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        );
        for _ in 0_usize
            ..env::var("QUICKCHECK_TESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100)
        {
            let nd = Nondeterministic::<bool, bool>::arbitrary(&mut g);
            match timeout(TIMEOUT, async { panic::catch_unwind(|| nd.determinize()) }).await {
                Ok(Ok(_determinized)) => {}
                Ok(Err(_panicked)) => {
                    panic!("Non-shrunk input panicked: {nd:?}")
                }
                Err(_timed_out) => {
                    for shrunk in nd.shrink() {
                        #[allow(clippy::manual_assert)]
                        if timeout(TIMEOUT, async { panic::catch_unwind(|| nd.determinize()) })
                            .await
                            .is_err()
                        {
                            panic!("Reduced case: {shrunk:?}")
                        }
                    }
                    panic!("Reduced case: {nd:?}")
                }
            }
        }
    }

    /// Has to be manual since `quickcheck!` doesn't understand `async`
    #[tokio::test]
    #[allow(clippy::integer_division, clippy::std_instead_of_core)]
    async fn subset_construction_timed_bool_bool() {
        subset_construction_timed::<bool, bool>().await;
    }

    #[inline]
    async fn shrink_subset_construction<
        A: fmt::Debug + panic::RefUnwindSafe + Arbitrary + Clone + Ord + Send + Sync,
        S: fmt::Debug + panic::RefUnwindSafe + Arbitrary + Copy + Ord + Send + Sync,
    >(
        orig_nd: Nondeterministic<A, S>,
        orig_input: Option<Vec<A>>,
    ) {
        let shrunk: Vec<_> = timeout(TIMEOUT, async {
            (orig_nd.clone(), orig_input.clone()).shrink()
        })
        .await
        .expect("Shrinking timed out; original argument was ({orig_nd:?}, {orig_input:?})")
        .collect(); // ouch, but necessary for async bounds
        for (nd, maybe_input) in shrunk {
            #[allow(clippy::match_wild_err_arm)]
            let d = match timeout(TIMEOUT, async { nd.determinize() }).await {
                Err(_timed_out) => panic!("Reduced case: ({nd:?}, {maybe_input:?})"),
                Ok(Err(_ill_formed)) => continue,
                Ok(Ok(ok)) => ok,
            };
            if let Some(input) = maybe_input {
                #[allow(clippy::manual_assert)]
                if !check_subset_construction(&nd, &d, &input) {
                    panic!("Reduced case: ({nd:?}, {:?})", Some(input))
                }
            }
        }
        panic!("Reduced case: ({orig_nd:?}, {orig_input:?})")
    }

    #[inline]
    fn check_subset_construction<A: fmt::Debug + Clone + Ord, S: fmt::Debug + Copy + Ord>(
        nd: &Nondeterministic<A, S>,
        d: &Deterministic<A, S>,
        input: &[A],
    ) -> bool {
        let Ok(lhs) = nd.accept(input.iter().cloned()) else {
            return false;
        };
        let Ok(rhs) = d.accept(input.iter().cloned()) else {
            return false;
        };
        lhs == rhs
    }
}

mod reduced {
    use super::*;

    #[inline]
    fn subset_construction<K: fmt::Debug + Copy + Ord, S: fmt::Debug + Copy + Ord>(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) {
        let Ok(d) = nd.determinize() else {
            return;
        };
        assert_eq!(
            nd.accept(input.iter().copied()).unwrap(),
            d.accept(input.iter().copied()).unwrap()
        );
    }

    // TODO: enable if we get a reduced case!
    /*
    /// Has to be manual since `quickcheck!` doesn't understand `async`
    #[allow(clippy::integer_division)]
    async fn subset_construction_timed<A: Clone + Ord + Sync, S: Copy + Ord + Sync>(
        nd: &Nondeterministic<A, S>,
        input: &[A],
    ) {
        let d = match timeout(TIMEOUT, async { nd.determinize() })
            .await
            .expect("Timed out")
        {
            Err(_ill_formed) => return,
            Ok(ok) => ok,
        };
        assert_eq!(
            nd.accept(input.iter().cloned()).unwrap(),
            d.accept(input.iter().cloned()).unwrap(),
        );
    }
    */

    #[test]
    fn subset_construction_1() {
        subset_construction::<(), ()>(
            &Automaton {
                states: vec![State {
                    transitions: CurryOpt {
                        wildcard: None,
                        none: None,
                        some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn subset_construction_2() {
        subset_construction::<bool, ()>(
            &Automaton {
                states: vec![State {
                    transitions: CurryOpt {
                        wildcard: None,
                        none: Some(Curry {
                            wildcard: None,
                            specific: vec![],
                        }),
                        some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: once(0).collect(),
            },
            &[false],
        );
    }

    #[test]
    fn subset_construction_3() {
        subset_construction(
            &Automaton {
                states: vec![State {
                    transitions: CurryOpt {
                        wildcard: None,
                        none: Some(Curry {
                            wildcard: None,
                            specific: vec![(
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Call {
                                    dst: once(0).collect(),
                                    call: call!(|x| x),
                                    push: true,
                                }),
                            )],
                        }),
                        some: once((
                            true,
                            Curry {
                                wildcard: Some(Return(Edge::Return {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                })),
                                specific: vec![],
                            },
                        ))
                        .collect(),
                    },
                    accepting: false,
                }],
                initial: once(0).collect(),
            },
            &[false, false],
        );
    }

    #[test]
    #[should_panic]
    fn subset_construction_4() {
        subset_construction(
            &Automaton {
                states: vec![State {
                    transitions: CurryOpt {
                        wildcard: None,
                        none: Some(Curry {
                            wildcard: None,
                            specific: vec![(
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Call {
                                    dst: once(0).collect(),
                                    call: call!(|x| x),
                                    push: false,
                                }),
                            )],
                        }),
                        some: once((
                            false,
                            Curry {
                                wildcard: Some(Return(Edge::Call {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                    push: false,
                                })),
                                specific: vec![(
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Return {
                                        dst: BTreeSet::new(),
                                        call: call!(|x| x),
                                    }),
                                )],
                            },
                        ))
                        .collect(),
                    },
                    accepting: false,
                }],
                initial: once(0).collect(),
            },
            &[false, false],
        );
    }

    #[test]
    #[allow(clippy::absolute_paths)]
    fn subset_construction_5() {
        subset_construction(
            &Automaton {
                states: vec![
                    State {
                        transitions: CurryOpt {
                            wildcard: Some(Curry {
                                wildcard: None,
                                specific: once((
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Local {
                                        dst: BTreeSet::new(),
                                        call: call!(::core::convert::identity),
                                    }),
                                ))
                                .collect(),
                            }),
                            none: None,
                            some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                    State {
                        transitions: CurryOpt {
                            wildcard: Some(Curry {
                                wildcard: None,
                                specific: once((
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Call {
                                        dst: [0, 1].into_iter().collect(),
                                        call: call!(::core::convert::identity),
                                        push: false,
                                    }),
                                ))
                                .collect(),
                            }),
                            none: None,
                            some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                ],
                initial: once(1).collect(),
            },
            &[false, false],
        );
    }

    #[test]
    #[allow(clippy::absolute_paths)]
    fn deabsurdify_1() {
        let mut na = Nondeterministic::<(), ()> {
            states: vec![State {
                transitions: CurryOpt {
                    wildcard: Some(Curry {
                        wildcard: Some(Return(Edge::Local {
                            dst: once(1).collect(),
                            call: call!(|x| x),
                        })),
                        specific: vec![],
                    }),
                    none: None,
                    some: BTreeMap::new(),
                },
                accepting: false,
            }],
            initial: once(0).collect(),
        };
        na.deabsurdify();
        drop(na.determinize());
    }
}

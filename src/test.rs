/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::panic,
    clippy::std_instead_of_core,
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::use_debug
)]

use crate::*;
use core::{iter::once, num::NonZeroUsize, time::Duration};
use std::collections::{BTreeMap, BTreeSet};
use tokio::time::timeout;

const TIMEOUT: Duration = Duration::from_secs(1);

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    // use core::fmt;
    use quickcheck::*;
    use std::{env, panic};

    // TODO: re-enable

    // #[inline]
    // async fn timed_subset_construction<K: Copy + fmt::Debug + Ord + Sync, S: Copy + Ord + Sync>(
    //     nd: &Nondeterministic<K, S>,
    //     input: &[K],
    // ) -> TestResult {
    //     use std::time::Instant;
    //     let mut start = Instant::now();
    //     let Ok(d) = timeout(TIMEOUT, determinize(nd)).await.expect("Timed out") else {
    //         return TestResult::discard();
    //     };
    //     println!("Determinized in {:?}", start.elapsed());
    //     start = Instant::now();
    //     if nd.accept(input.iter().copied()).unwrap() != d.accept(input.iter().copied()).unwrap() {
    //         return TestResult::failed();
    //     }
    //     println!("Tested {:?} in {:?}", input, start.elapsed());
    //     TestResult::passed()
    //     // panic!("euthanasia")
    // }

    // #[inline]
    // fn subset_construction<K: Copy + fmt::Debug + Ord, S: Copy + Ord>(
    //     nd: &Nondeterministic<K, S>,
    //     input: &[K],
    // ) -> TestResult {
    //     use std::time::Instant;
    //     let mut start = Instant::now();
    //     let Ok(d) = nd.determinize() else {
    //         return TestResult::discard();
    //     };
    //     println!("Determinized in {:?}", start.elapsed());
    //     start = Instant::now();
    //     if nd.accept(input.iter().copied()).unwrap() != d.accept(input.iter().copied()).unwrap() {
    //         return TestResult::failed();
    //     }
    //     println!("Tested {:?} in {:?}", input, start.elapsed());
    //     TestResult::passed()
    //     // panic!("euthanasia")
    // }

    quickcheck! {

        fn range_overlap_commutativity(a: Range<u8>, b: Range<u8>) -> bool {
            a.overlap(&b) == b.overlap(&a)
        }

        // fn subset_construction_bool_bool(nd: Nondeterministic<bool, bool>, inputs: Vec<bool>) -> TestResult {
        //     subset_construction(&nd, &inputs).await
        // }

        // fn subset_construction_bool_u8(nd: Nondeterministic<bool, u8>, inputs: Vec<bool>) -> TestResult {
        //     subset_construction(&nd, &inputs)
        // }

        // fn subset_construction_u8_bool(nd: Nondeterministic<u8, bool>, inputs: Vec<u8>) -> TestResult {
        //     subset_construction(&nd, &inputs)
        // }

        // fn subset_construction_u8_u8(nd: Nondeterministic<u8, u8>, inputs: Vec<u8>) -> TestResult {
        //     subset_construction(&nd, &inputs)
        // }

    }

    /// Has to be manual since `quickcheck!` doesn't understand `async`
    #[tokio::test]
    async fn determinization_under_1s() {
        let mut g = quickcheck::Gen::new(
            env::var("QUICKCHECK_GENERATOR_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                // .unwrap_or(100),
                .unwrap_or(2),
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
}

mod reduced {
    use super::*;

    #[inline]
    async fn subset_construction<K: Copy + Ord + Sync, S: Copy + Ord + Sync>(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) {
        let Ok(d) = timeout(TIMEOUT, async { nd.determinize() })
            .await
            .expect("Timed out")
        else {
            return;
        };
        assert_eq!(
            nd.accept(input.iter().copied()).unwrap(),
            d.accept(input.iter().copied()).unwrap()
        );
    }

    #[tokio::test]
    async fn subset_construction_1() {
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
        )
        .await;
    }

    #[tokio::test]
    async fn subset_construction_2() {
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
        )
        .await;
    }

    #[tokio::test]
    async fn subset_construction_3() {
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
        )
        .await;
    }

    #[tokio::test]
    #[should_panic]
    async fn subset_construction_4() {
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
        )
        .await;
    }

    #[test]
    fn deabsurdify_1() {
        // Automaton {
        //     states: vec![State {
        //         transitions: CurryOpt {
        //             wildcard: None,
        //             none: Some(Curry {
        //                 wildcard: None,
        //                 specific: vec![(
        //                     Range {
        //                         first: true,
        //                         last: true,
        //                     },
        //                     Return(Call {
        //                         dst: { 7945555980123157236 },
        //                         call: Call {
        //                             ptr: 0x10b659de0,
        //                             src: "",
        //                         },
        //                         push: false,
        //                     }),
        //                 )],
        //             }),
        //             some: {},
        //         },
        //         accepting: true,
        //     }],
        //     initial: { 0 },
        // };
        let mut edge: Edge<(), usize> = Edge::Local {
            dst: 42,
            call: call!(|x| x),
        };
        edge.deabsurdify(NonZeroUsize::new(1).unwrap());
        assert_eq!(*edge.dst(), 0);
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::diverging_sub_expression,
    clippy::panic,
    clippy::print_stdout,
    clippy::todo,
    clippy::unwrap_used,
    clippy::use_debug,
    unreachable_code
)]

#[cfg(feature = "quickcheck")]
mod prop {
    use crate::*;
    use core::fmt;
    use quickcheck::{quickcheck, TestResult};
    use std::panic;

    #[inline]
    fn determinization_implies_no_runtime_errors<
        K: Copy + fmt::Debug + Ord,
        S: fmt::Debug + Copy + Ord,
    >(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) -> TestResult {
        if nd.determinize().is_err() {
            return TestResult::discard();
        };
        let _ = nd.accept(input.iter().copied()).unwrap();
        TestResult::passed()
    }

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

    quickcheck! {
        fn range_overlap_commutativity(a: Range<u8>, b: Range<u8>) -> bool {
            a.overlap(&b) == b.overlap(&a)
        }

        fn deabsurdify_implies_check_nd(nd: Nondeterministic<bool, bool>) -> bool {
            let mut nd = nd;
            if !nd.deabsurdify() {
                return true;
            }
            nd.check().is_ok()
        }

        fn deabsurdify_implies_check_d(d: Deterministic<bool, bool>) -> bool {
            let mut d = d;
            if !d.deabsurdify() {
                return true;
            }
            d.check().is_ok()
        }

        fn determinization_implies_no_runtime_errors_bool_bool(nd: Nondeterministic<bool, bool>, input: Vec<bool>) -> TestResult {
            determinization_implies_no_runtime_errors(&nd, &input)
        }

        fn generalize_determinize_succeeds(d: Deterministic<bool, bool>) -> bool {
            let mut d = d;
            if !d.deabsurdify() {
                return true;
            }
            panic::catch_unwind(|| d.clone().generalize().determinize()).is_ok()
        }

        fn subset_construction_bool_bool(nd: Nondeterministic<bool, bool>, input: Vec<bool>) -> TestResult {
            subset_construction(&nd, &input)
        }

        // fn subset_construction_bool_u8(nd: Nondeterministic<bool, u8>, input: Vec<bool>) -> TestResult {
        //     subset_construction(&nd, &input)
        // }

        // fn subset_construction_u8_bool(nd: Nondeterministic<u8, bool>, input: Vec<u8>) -> TestResult {
        //     subset_construction(&nd, &input)
        // }

        // fn subset_construction_u8_u8(nd: Nondeterministic<u8, u8>, input: Vec<u8>) -> TestResult {
        //     subset_construction(&nd, &input)
        // }
    }
}

mod reduced {
    use crate::*;
    use core::{fmt, iter};
    use std::collections::{BTreeMap, BTreeSet};

    // #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    // enum SubsetConstructionWitness {
    //     DeterminizationFailed,
    //     IdenticalBehavior,
    // }

    fn deabsurdify_implies_check<K: fmt::Debug + Clone + Ord, S: fmt::Debug + Copy + Ord>(
        mut nd: Nondeterministic<K, S>,
    ) {
        let pre = nd.clone();
        if !nd.deabsurdify() {
            return;
        }
        assert_eq!(nd.check(), Ok(()));
        assert_ne!(pre, nd, "Nothing changed--is this test off?");
    }

    #[inline]
    fn determinization_implies_no_runtime_errors<
        K: fmt::Debug + Copy + Ord,
        S: fmt::Debug + Copy + Ord,
    >(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) {
        println!("Original nondeterministic automaton: {nd:?}");
        println!();
        let Ok(d) = nd.determinize() else {
            return;
        };
        println!("Successfully determinized: {d:?}");
        println!();
        println!("Running the nondeterministic automaton...");
        let mut run_nd = input.iter().copied().run(nd);
        println!("    {run_nd:?}");
        while let Some(r) = run_nd.next() {
            if let Err(e) = r {
                panic!("Nondeterministic automaton panicked (but determinization didn't): {e:?}");
            }
            println!("    {run_nd:?}");
        }
        let _ = run_nd.ctrl.unwrap_err();
        panic!("Ran to completion: is something wrong with this test?");
    }

    // #[inline]
    // fn subset_construction<K: fmt::Debug + Copy + Ord, S: fmt::Debug + Copy + Ord>(
    //     nd: &Nondeterministic<K, S>,
    //     input: &[K],
    // ) -> SubsetConstructionWitness {
    //     println!("Original nondeterministic automaton: {nd:?}");
    //     println!();
    //     let Ok(d) = nd.determinize() else {
    //         return SubsetConstructionWitness::DeterminizationFailed;
    //     };
    //     println!("Deterministic automaton:");
    //     println!("{d:#?}");
    //     println!();
    //     println!("Running the nondeterministic automaton...");
    //     let mut run_nd = input.iter().copied().run(nd);
    //     println!("    {run_nd:?}");
    //     while let Some(r) = run_nd.next() {
    //         if let Err(e) = r {
    //             panic!("Nondeterministic automaton panicked (but determinization didn't): {e:?}");
    //         }
    //         println!("    {run_nd:?}");
    //     }
    //     let nd_accept = run_nd.ctrl.unwrap_err();
    //     println!();
    //     println!("Running the deterministic automaton...");
    //     let mut run_d = input.iter().copied().run(&d);
    //     println!("    {run_d:?}");
    //     while let Some(r) = run_d.next() {
    //         if let Err(e) = r {
    //             panic!("Deterministic automaton panicked: {e:?}");
    //         }
    //         println!("    {run_d:?}");
    //     }
    //     let d_accept = run_d.ctrl.unwrap_err();
    //     assert_eq!(nd_accept, d_accept);
    //     SubsetConstructionWitness::IdenticalBehavior
    // }

    /*
    #[test]
    fn determinization_implies_no_runtime_errors_1() {
        let nd = Nondeterministic {
            states: vec![State {
                transitions: CurryOpt {
                    wildcard: Some(Wildcard::Specific(
                        [
                            (
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Local {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                }),
                            ),
                            (
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Call {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                    push: false,
                                }),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )),
                    none: None,
                    some: BTreeMap::new(),
                },
                accepting: false,
            }],
            initial: iter::once(0).collect(),
        };
        let _falsely_determinized = Deterministic::<bool, bool> {
            states: vec![
                State {
                    transitions: CurryOpt {
                        wildcard: Some(Wildcard::Any(Return(Edge::Local {
                            dst: 0,
                            call: call!(|x| x),
                        }))),
                        none: None,
                        some: BTreeMap::new(),
                    },
                    accepting: false,
                },
                State {
                    transitions: CurryOpt {
                        wildcard: Some(Wildcard::Specific(
                            [
                                (
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Local {
                                        dst: 0,
                                        call: call!(|x| x),
                                    }),
                                ),
                                (
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Call {
                                        dst: 0,
                                        call: call!(|x| x),
                                        push: false,
                                    }),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        )),
                        none: None,
                        some: BTreeMap::new(),
                    },
                    accepting: false,
                },
            ],
            initial: 1,
        };
        determinization_implies_no_runtime_errors(&nd, &[false]);
    }
    */

    #[test]
    fn deabsurdify_1() {
        deabsurdify_implies_check(Automaton {
            states: vec![State {
                transitions: CurryOpt {
                    wildcard: None,
                    none: None,
                    some: iter::once((
                        false,
                        Wildcard::Specific(
                            [
                                (
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Return {
                                        dst: BTreeSet::new(),
                                        call: call!(|x| x),
                                    }),
                                ),
                                (
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Return {
                                        dst: BTreeSet::new(),
                                        call: call!(|x| x),
                                    }),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ))
                    .collect(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn deabsurdify_2() {
        deabsurdify_implies_check(Automaton {
            states: vec![State {
                transitions: CurryOpt {
                    wildcard: None,
                    none: Some(Wildcard::Specific(
                        [
                            (
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Local {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                }),
                            ),
                            (
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Call {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                    push: false,
                                }),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    )),
                    some: BTreeMap::new(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn deabsurdify_3() {
        deabsurdify_implies_check(Nondeterministic::<(), ()> {
            states: vec![],
            initial: iter::once(1).collect(),
        });
    }

    #[test]
    fn determinization_implies_no_runtime_errors_1() {
        determinization_implies_no_runtime_errors(
            &Nondeterministic::<(), ()> {
                states: vec![],
                initial: iter::once(0).collect(),
            },
            &[],
        );
    }
}

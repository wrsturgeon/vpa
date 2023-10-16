/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::panic,
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::use_debug
)]

use crate::*;
use core::iter::once;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "quickcheck")]
use core::fmt;

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;

    #[inline]
    fn subset_construction<K: Copy + fmt::Debug + Ord, S: Copy + Ord>(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) -> TestResult {
        use std::time::Instant;
        let mut start = Instant::now();
        let Ok(d) = nd.determinize() else {
            return TestResult::discard();
        };
        println!("Determinized in {:?}", start.elapsed());
        start = Instant::now();
        if nd.accept(input.iter().copied()).unwrap() != d.accept(input.iter().copied()).unwrap() {
            return TestResult::failed();
        }
        println!("Tested {:?} in {:?}", input, start.elapsed());
        TestResult::passed()
        // panic!("euthanasia")
    }

    quickcheck! {

        fn range_overlap_commutativity(a: Range<u8>, b: Range<u8>) -> bool {
            a.overlap(&b) == b.overlap(&a)
        }

        fn subset_construction_bool_bool(nd: Nondeterministic<bool, bool>, inputs: Vec<bool>) -> TestResult {
            subset_construction(&nd, &inputs)
        }

        // TODO: re-enable

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
}

mod reduced {
    use super::*;

    fn subset_construction<K: Copy + Ord, S: Copy + Ord>(nd: &Nondeterministic<K, S>, input: &[K]) {
        let Ok(d) = nd.determinize() else {
            return;
        };
        assert_eq!(
            nd.accept(input.iter().copied()).unwrap(),
            d.accept(input.iter().copied()).unwrap()
        );
    }

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
}

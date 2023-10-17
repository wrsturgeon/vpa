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

use crate::*;
use core::{fmt, iter::once};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

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

        fn subset_construction_bool_bool(nd: Nondeterministic<bool, bool>, input: Vec<bool>) -> TestResult {
            subset_construction(&nd, &input)
        }

        // TODO: re-enable:

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
    use super::*;

    #[inline]
    fn subset_construction<K: fmt::Debug + Copy + Ord, S: fmt::Debug + Copy + Ord>(
        nd: &Nondeterministic<K, S>,
        input: &[K],
    ) {
        let Ok(d) = nd.determinize() else {
            return;
        };
        println!("Deterministic automaton:");
        println!("{d:#?}");
        println!();
        println!("Running the nondeterministic automaton...");
        let nd_accept = nd
            .accept(input.iter().copied())
            .expect("Nondeterministic automaton panicked");
        println!();
        println!("Running the deterministic automaton...");
        let d_accept = d
            .accept(input.iter().copied())
            .expect("Deterministic automaton panicked");
        assert_eq!(nd_accept, d_accept);
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
    fn subset_construction_6() {
        subset_construction(
            &Automaton {
                states: vec![
                    State {
                        transitions: CurryOpt {
                            wildcard: Some(Curry {
                                wildcard: Some(Return(Edge::Call {
                                    dst: BTreeSet::new(),
                                    call: call!(|x| x),
                                    push: false,
                                })),
                                specific: vec![],
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
                                        dst: BTreeSet::new(),
                                        call: call!(|x| x),
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
                initial: [0, 1].into_iter().collect(),
            },
            &[false, false],
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn subset_construction_7() {
        subset_construction(
            &Nondeterministic::<bool, ()> {
                states: vec![
                    State {
                        transitions: CurryOpt {
                            wildcard: None,
                            none: Some(Curry {
                                wildcard: None,
                                specific: once((
                                    Range {
                                        first: false,
                                        last: false,
                                    },
                                    Return(Edge::Local {
                                        dst: BTreeSet::new(),
                                        call: call!(|x| x),
                                    }),
                                ))
                                .collect(),
                            }),
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
                                    Return(Edge::Local {
                                        dst: [0, 1].into_iter().collect(),
                                        call: call!(|x| x),
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
        let _determinized = Deterministic::<bool, ()> {
            states: vec![
                State {
                    transitions: CurryOpt {
                        wildcard: None,
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
                                Return(Edge::Local {
                                    dst: 1,
                                    call: call!(|x| x),
                                }),
                            ))
                            .collect(),
                        }),
                        none: Some(Curry {
                            wildcard: None,
                            specific: once((
                                Range {
                                    first: false,
                                    last: false,
                                },
                                Return(Edge::Local {
                                    dst: 0,
                                    call: call!(|x| x),
                                }),
                            ))
                            .collect(),
                        }),
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
                                Return(Edge::Local {
                                    dst: 1,
                                    call: call!(|x| x),
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
            initial: 2,
        };
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

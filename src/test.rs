/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::unwrap_used)]

use crate::*;
use core::iter::once;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;

    #[inline]
    fn subset_construction<K: Copy + Ord, S: Copy + Ord>(
        nd: &Nondeterministic<K, S>,
        inputs: &[Vec<K>],
    ) -> TestResult {
        let Ok(d) = nd.determinize() else {
            return TestResult::discard();
        };
        for input in inputs {
            if nd.accept(input.iter().copied()).unwrap() != d.accept(input.iter().copied()).unwrap()
            {
                return TestResult::failed();
            }
        }
        TestResult::passed()
    }

    quickcheck! {

        fn range_overlap_commutativity(a: Range<u8>, b: Range<u8>) -> bool {
            a.overlap(&b) == b.overlap(&a)
        }

        fn subset_construction_bool_bool(nd: Nondeterministic<bool, bool>, inputs: Vec<Vec<bool>>) -> TestResult {
            subset_construction(&nd, &inputs)
        }

        fn subset_construction_bool_u8(nd: Nondeterministic<bool, u8>, inputs: Vec<Vec<bool>>) -> TestResult {
            subset_construction(&nd, &inputs)
        }

        fn subset_construction_u8_bool(nd: Nondeterministic<u8, bool>, inputs: Vec<Vec<u8>>) -> TestResult {
            subset_construction(&nd, &inputs)
        }

        fn subset_construction_u8_u8(nd: Nondeterministic<u8, u8>, inputs: Vec<Vec<u8>>) -> TestResult {
            subset_construction(&nd, &inputs)
        }

    }
}

mod reduced {
    use super::*;

    // Automaton { states: [State { transitions: CurryOpt { wildcard: None, none: None, some: {} }, accepting: false }], initial: {} }, []

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
        subset_construction::<bool, bool>(
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
        subset_construction::<u8, u8>(
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
            &[0],
        );
    }
}

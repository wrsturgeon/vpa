/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Subset construction algorithm for determinizing nondeterministic automata.

use crate::{
    merge, CurryOpt, Deterministic, Edge, IllFormed, Nondeterministic, Return, State, Wildcard,
};
use core::{borrow::Borrow, fmt, iter::once};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord> Deterministic<A, S> {
    /// Generalize a deterministic automaton to an identical but nominally nondeterministic automaton.
    #[inline]
    #[must_use]
    pub fn generalize(self) -> Nondeterministic<A, S> {
        Nondeterministic {
            states: self
                .states
                .into_iter()
                .map(|state| State {
                    transitions: generalize_curry_opt(state.transitions),
                    accepting: state.accepting,
                })
                .collect(),
            initial: once(self.initial).collect(),
        }
    }
}

/// Generalize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
#[allow(clippy::type_complexity)]
fn generalize_curry_opt<A: 'static + fmt::Debug + Ord, S: 'static + fmt::Debug + Copy + Ord>(
    d: CurryOpt<S, Wildcard<A, Return<Edge<A, S, usize>>>>,
) -> CurryOpt<S, Wildcard<A, Return<Edge<A, S, BTreeSet<usize>>>>> {
    CurryOpt {
        wildcard: d.wildcard.map(generalize_wildcard),
        none: d.none.map(generalize_wildcard),
        some: d
            .some
            .into_iter()
            .map(|(arg, etc)| (arg, generalize_wildcard(etc)))
            .collect(),
    }
}

/// Generalize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
fn generalize_wildcard<A: 'static + fmt::Debug + Ord, S: 'static + fmt::Debug + Copy + Ord>(
    d: Wildcard<A, Return<Edge<A, S, usize>>>,
) -> Wildcard<A, Return<Edge<A, S, BTreeSet<usize>>>> {
    match d {
        Wildcard::Any(Return(edge)) => Wildcard::Any(Return(generalize_edge(edge))),
        Wildcard::Specific(v) => Wildcard::Specific(
            v.into_iter()
                .map(|(k, Return(edge))| (k, Return(generalize_edge(edge))))
                .collect(),
        ),
    }
}

/// Generalize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
fn generalize_edge<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord>(
    d: Edge<A, S, usize>,
) -> Edge<A, S, BTreeSet<usize>> {
    match d {
        Edge::Call { dst, call, push } => Edge::Call {
            dst: once(dst).collect(),
            call,
            push,
        },
        Edge::Return { dst, call } => Edge::Return {
            dst: once(dst).collect(),
            call,
        },
        Edge::Local { dst, call } => Edge::Local {
            dst: once(dst).collect(),
            call,
        },
        Edge::Phantom(..) => never!(),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_curry_opt<A: 'static + fmt::Debug + Ord, S: 'static + fmt::Debug + Copy + Ord>(
    nd: CurryOpt<S, Wildcard<A, Return<Edge<A, S, BTreeSet<usize>>>>>,
    ordering: &[BTreeSet<usize>],
) -> CurryOpt<S, Wildcard<A, Return<Edge<A, S, usize>>>> {
    CurryOpt {
        wildcard: nd.wildcard.map(|wild| fix_indices_wildcard(wild, ordering)),
        none: nd.none.map(|none| fix_indices_wildcard(none, ordering)),
        some: nd
            .some
            .into_iter()
            .map(|(arg, etc)| (arg, fix_indices_wildcard(etc, ordering)))
            .collect(),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
fn fix_indices_wildcard<A: 'static + fmt::Debug + Ord, S: 'static + fmt::Debug + Copy + Ord>(
    nd: Wildcard<A, Return<Edge<A, S, BTreeSet<usize>>>>,
    ordering: &[BTreeSet<usize>],
) -> Wildcard<A, Return<Edge<A, S, usize>>> {
    match nd {
        Wildcard::Any(Return(edge)) => Wildcard::Any(Return(fix_indices_edge(edge, ordering))),
        Wildcard::Specific(v) => Wildcard::Specific(
            v.into_iter()
                .map(|(k, Return(edge))| (k, Return(fix_indices_edge(edge, ordering))))
                .collect(),
        ),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
fn fix_indices_edge<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord>(
    nd: Edge<A, S, BTreeSet<usize>>,
    ordering: &[BTreeSet<usize>],
) -> Edge<A, S, usize> {
    match nd {
        Edge::Call { dst, call, push } => Edge::Call {
            dst: unwrap!(ordering.binary_search(&dst)),
            call,
            push,
        },
        Edge::Return { dst, call } => Edge::Return {
            dst: unwrap!(ordering.binary_search(&dst)),
            call,
        },
        Edge::Local { dst, call } => Edge::Local {
            dst: unwrap!(ordering.binary_search(&dst)),
            call,
        },
        Edge::Phantom(..) => never!(),
    }
}

impl<A: fmt::Debug + Ord, S: fmt::Debug + Copy + Ord> Nondeterministic<A, S> {
    /// Turn an iterator over indices into an iterator over references to states.
    #[inline]
    fn get_states<I: IntoIterator>(
        &self,
        i: I,
    ) -> impl Iterator<Item = &State<A, S, BTreeSet<usize>>>
    where
        I::Item: Borrow<usize>,
    {
        i.into_iter().map(|j| get!(self.states, *j.borrow()))
    }
}

impl<A: fmt::Debug + Clone + Ord, S: fmt::Debug + Copy + Ord> Nondeterministic<A, S> {
    /// Subset construction algorithm for determinizing nondeterministic automata.
    /// # Errors
    /// If there's an ambiguity (which would have crashed the nondeterministic automaton anyway).
    #[inline]
    #[allow(clippy::missing_panics_doc, clippy::unwrap_in_result)]
    pub fn determinize(&self) -> Result<Deterministic<A, S>, IllFormed<A, S, BTreeSet<usize>>> {
        // Check that the source graph is well-formed
        self.check()?;

        // Associate each subset of states with a merged state
        let mut subsets_as_states: BTreeMap<BTreeSet<usize>, State<A, S, BTreeSet<usize>>> =
            BTreeMap::new();
        self.explore(&mut subsets_as_states, self.initial.clone())?;

        // Fix an ordering on those subsets
        let mut ordering: Vec<BTreeSet<usize>> = subsets_as_states.keys().cloned().collect();
        ordering.sort_unstable();
        ordering.dedup();

        Ok(Deterministic {
            initial: unwrap!(ordering.binary_search(&self.initial)),
            states: ordering
                .iter()
                .map(|set| {
                    let State {
                        transitions,
                        accepting,
                    } = unwrap!(subsets_as_states.remove(set));
                    State {
                        transitions: fix_indices_curry_opt(transitions, &ordering),
                        accepting,
                    }
                })
                .collect(),
        })
    }

    /// Associate each subset of states with a merged state.
    fn explore(
        &self,
        subsets_as_states: &mut BTreeMap<BTreeSet<usize>, State<A, S, BTreeSet<usize>>>,
        subset: BTreeSet<usize>,
    ) -> Result<(), IllFormed<A, S, BTreeSet<usize>>> {
        // Check if we've seen this subset already
        let Entry::Vacant(entry) = subsets_as_states.entry(subset.clone()) else {
            return Ok(());
        };

        #[allow(clippy::print_stdout, clippy::use_debug)]
        {
            println!("Merging {subset:?}");
        }

        // Merge this subset of states into one (most of the heavy lifting)
        let mega_state: State<A, S, BTreeSet<usize>> = match merge(self.get_states(subset)) {
            // If there were no states in the subset, reject immediately without a transition
            None => State {
                transitions: CurryOpt {
                    wildcard: Some(Wildcard::Any(Return(Edge::Local {
                        dst: BTreeSet::new(),
                        call: call!(|x| x),
                    }))),
                    none: None,
                    some: BTreeMap::new(),
                },
                accepting: false,
            },
            // If they successfully merged, return the merged state
            Some(Ok(ok)) => ok,
            // If they didn't successfully merge, something's wrong with the original automaton
            Some(Err(e)) => return Err(e),
        };

        // Cache all possible next states
        #[allow(clippy::needless_collect)] // <-- false positive: can't move `mega_state` below
        let dsts: BTreeSet<BTreeSet<usize>> = mega_state
            .transitions
            .values()
            .flat_map(Wildcard::values)
            .map(|edge| edge.dst().clone())
            .collect();

        // Associate this subset of states with the merged state
        let _ = entry.insert(mega_state);

        // Recurse on all destinations
        dsts.into_iter()
            .try_fold((), |(), dst| self.explore(subsets_as_states, dst))
    }
}

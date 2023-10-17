/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Subset construction algorithm for determinizing nondeterministic automata.

use crate::{
    merge, Curry, CurryOpt, Deterministic, Edge, IllFormed, Nondeterministic, Return, State,
};
use core::{borrow::Borrow, iter::once};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

impl<A: Ord, S: Copy + Ord> Deterministic<A, S> {
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
fn generalize_curry_opt<A: 'static + Ord, S: 'static + Copy + Ord>(
    d: CurryOpt<S, Curry<A, Return<Edge<S, usize>>>>,
) -> CurryOpt<S, Curry<A, Return<Edge<S, BTreeSet<usize>>>>> {
    CurryOpt {
        wildcard: d.wildcard.map(generalize_curry),
        none: d.none.map(generalize_curry),
        some: d
            .some
            .into_iter()
            .map(|(arg, etc)| (arg, generalize_curry(etc)))
            .collect(),
    }
}

/// Generalize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
fn generalize_curry<A: Ord, S: 'static + Copy + Ord>(
    d: Curry<A, Return<Edge<S, usize>>>,
) -> Curry<A, Return<Edge<S, BTreeSet<usize>>>> {
    Curry {
        wildcard: d.wildcard.map(|Return(x)| Return(generalize_edge(x))),
        specific: d
            .specific
            .into_iter()
            .map(|(token, Return(x))| (token, Return(generalize_edge(x))))
            .collect(),
    }
}

/// Generalize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
fn generalize_edge<S: Copy + Ord>(d: Edge<S, usize>) -> Edge<S, BTreeSet<usize>> {
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
    }
}

/// Determinize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
#[allow(clippy::type_complexity)]
fn determinize_curry_opt<A: 'static + Ord, S: 'static + Copy + Ord>(
    nd: CurryOpt<S, Curry<A, Return<Edge<S, BTreeSet<usize>>>>>,
    ordering: &[BTreeSet<usize>],
) -> CurryOpt<S, Curry<A, Return<Edge<S, usize>>>> {
    CurryOpt {
        wildcard: nd.wildcard.map(|wild| determinize_curry(wild, ordering)),
        none: nd.none.map(|none| determinize_curry(none, ordering)),
        some: nd
            .some
            .into_iter()
            .map(|(arg, etc)| (arg, determinize_curry(etc, ordering)))
            .collect(),
    }
}

/// Determinize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
fn determinize_curry<A: Ord, S: 'static + Copy + Ord>(
    nd: Curry<A, Return<Edge<S, BTreeSet<usize>>>>,
    ordering: &[BTreeSet<usize>],
) -> Curry<A, Return<Edge<S, usize>>> {
    Curry {
        wildcard: nd
            .wildcard
            .map(|Return(x)| Return(determinize_edge(x, ordering))),
        specific: nd
            .specific
            .into_iter()
            .map(|(token, Return(x))| (token, Return(determinize_edge(x, ordering))))
            .collect(),
    }
}

/// Determinize a deterministic automaton to an identical but nominally nondeterministic automaton.
#[inline]
fn determinize_edge<S: Copy + Ord>(
    nd: Edge<S, BTreeSet<usize>>,
    ordering: &[BTreeSet<usize>],
) -> Edge<S, usize> {
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
    }
}

impl<A: Ord, S: Copy + Ord> Nondeterministic<A, S> {
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

    /// Subset construction algorithm for determinizing nondeterministic automata.
    /// # Errors
    /// If there's an ambiguity (which would have crashed the nondeterministic automaton anyway).
    #[inline]
    #[allow(clippy::missing_panics_doc, clippy::unwrap_in_result)]
    pub fn determinize(&self) -> Result<Deterministic<A, S>, IllFormed>
    where
        A: Clone,
    {
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
                        transitions: determinize_curry_opt(transitions, &ordering),
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
    ) -> Result<(), IllFormed>
    where
        A: Clone,
    {
        // Merge all states into one (here's most of the heavy lifting)
        let mega_state: State<_, _, _> = match merge(self.get_states(subset.clone())) {
            None => {
                drop(subsets_as_states.insert(BTreeSet::new(), State::default()));
                return Ok(());
            }
            Some(r) => match r {
                Ok(state) => state,
                Err(e) => return Err(e),
            },
        };

        // Check if we've seen this subset already
        let Entry::Vacant(entry) = subsets_as_states.entry(subset) else {
            // TODO: check for inconsistencies
            return Ok(());
        };

        // Cache all possible next states
        #[allow(clippy::needless_collect)] // <-- false positive: can't move `mega_state` below
        let dsts: Vec<BTreeSet<usize>> = mega_state
            .transitions
            .values()
            .flat_map(Curry::values)
            .map(|&Return(ref edge)| edge.dst().clone())
            .collect();

        // Associate this subset of states with the merged state
        let _ = entry.insert(mega_state);

        // Recurse on all destinations
        dsts.into_iter()
            .try_fold((), |(), dst| self.explore(subsets_as_states, dst))
    }
}

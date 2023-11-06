/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on nondeterministic finite automata returning nondeterministic finite automata.

#![allow(clippy::manual_assert, clippy::match_wild_err_arm, clippy::panic)]

use crate::{Ctrl, Curry, Deterministic, Graph, Input, Merge, RangeMap, State, Transition};
use core::{iter, mem, ops};
use std::collections::BTreeSet;

impl<I: Input> ops::BitOr for Deterministic<I> {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        let mut s = self.generalize();
        let other = rhs.generalize();
        // Note that union on pushdown automata is undecidable;
        // we just reject a subset of automata that wouldn't work.
        if s.check().is_err() {
            panic!("Internal error")
        }
        let size = s.states.len();
        let Graph {
            states: other_states,
            initial: other_initial,
            tags: other_tags,
        } = other.map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));
        s.states.extend(other_states);
        s.initial.extend(other_initial);
        s.tags = unwrap!(s.tags.merge(other_tags));
        s.sort();
        s.determinize().unwrap_or_else(|e| panic!("{e}"))
    }
}

impl<I: Input> ops::Shr<Self> for Deterministic<I> {
    type Output = Self;
    #[inline]
    fn shr(mut self, other: Self) -> Self::Output {
        let rhs_init = get!(other.states, other.initial)
            .transitions
            .clone()
            .generalize();

        let accepting_indices =
            self.states
                .iter_mut()
                .enumerate()
                .fold(BTreeSet::new(), |mut acc_i, (i, st)| {
                    if st.non_accepting.is_empty() {
                        st.non_accepting = iter::once(
                            "Ran the first part of a two-parser concatenation \
                            (with `>>`) but not the second one."
                                .to_owned(),
                        )
                        .collect(); // <-- No longer accepting since we need to run the second parser
                        let _ = acc_i.insert(i);
                    }
                    acc_i
                });
        let accepting_tags: BTreeSet<String> = self
            .tags
            .iter()
            .filter(|&(_, v)| accepting_indices.contains(v))
            .map(|(k, _)| k.clone())
            .collect();

        let mut s = self.generalize();
        if s.check().is_err() {
            panic!("Internal error")
        }
        let size = s.states.len();

        let Graph {
            states: other_states,
            initial: other_initial,
            tags: other_tags,
        } = other
            .generalize()
            .map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));

        s.states.extend(other_states);
        s.tags.extend(other_tags);

        // For every transition that an empty stack can take from the initial state of the right-hand parser,
        // add that transition (only on the empty stack) to each accepting state of the left-hand parser.
        for state in &mut s.states {
            state.transitions =
                mem::replace(&mut state.transitions, Curry::Wildcard(Transition::Return))
                    .merge(rhs_init.clone())
                    .unwrap_or_else(|e| panic!("{e}"));
        }

        // If any initial states are immediately accepting, we need to start in the second parser, too.
        if s.initial.iter().any(|r| {
            will_accept(
                r.as_ref().map_or_else(|st| Err(st.as_str()), |&i| Ok(i)),
                &accepting_indices,
                &accepting_tags,
            )
        }) {
            s.initial.extend(other_initial.iter().cloned());
        }

        let mut out = Graph {
            states: s
                .states
                .into_iter()
                .map(|st| {
                    add_tail_call_state(st, &other_initial, &accepting_indices, &accepting_tags)
                })
                .collect(),
            ..s
        };
        out.sort();
        out.determinize().unwrap_or_else(|e| panic!("{e}"))
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_state<I: Input, C: Ctrl<I>>(
    s: State<I, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> State<I, BTreeSet<Result<usize, String>>> {
    State {
        transitions: add_tail_call_curry(
            s.transitions,
            other_init,
            accepting_indices,
            accepting_tags,
        ),
        non_accepting: s.non_accepting,
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_curry<I: Input, C: Ctrl<I>>(
    s: Curry<I, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> Curry<I, BTreeSet<Result<usize, String>>> {
    match s {
        Curry::Wildcard(t) => Curry::Wildcard(add_tail_call_transition(
            t,
            other_init,
            accepting_indices,
            accepting_tags,
        )),
        Curry::Scrutinize(rm) => Curry::Scrutinize(add_tail_call_range_map(
            rm,
            other_init,
            accepting_indices,
            accepting_tags,
        )),
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_range_map<I: Input, C: Ctrl<I>>(
    s: RangeMap<I, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> RangeMap<I, BTreeSet<Result<usize, String>>> {
    RangeMap(
        s.0.into_iter()
            .map(|(k, v)| {
                (
                    k,
                    add_tail_call_transition(v, other_init, accepting_indices, accepting_tags),
                )
            })
            .collect(),
    )
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
#[allow(clippy::needless_pass_by_value, clippy::todo)] // <-- FIXME
fn add_tail_call_transition<I: Input, C: Ctrl<I>>(
    _s: Transition<I, C>,
    _other_init: &BTreeSet<Result<usize, String>>,
    _accepting_indices: &BTreeSet<usize>,
    _accepting_tags: &BTreeSet<String>,
) -> Transition<I, BTreeSet<Result<usize, String>>> {
    // let good = s
    //     .dst
    //     .view()
    //     .any(|result| will_accept(result, accepting_indices, accepting_tags));
    // let iter = s.dst.view().map(|result| result.map_err(str::to_owned));
    // let dst = if good {
    //     iter.chain(other_init.iter().cloned()).collect()
    // } else {
    //     iter.collect()
    // };
    // Transition {
    //     dst,
    //     act: s.act,
    //     update: s.update,
    // }
    todo!()
}

/// Check if this state corresponds to an accepting state.
#[inline]
fn will_accept(
    r: Result<usize, &str>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> bool {
    match r {
        Ok(i) => accepting_indices.contains(&i),
        Err(tag) => accepting_tags.contains(tag),
    }
}

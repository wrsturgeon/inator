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

impl<I: Input> ops::BitOr<Self> for Deterministic<I> {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        let mut s = self.generalize();
        let other = rhs.generalize();
        // Note that union on pushdown automata is undecidable;
        // we presumably reject a subset of automata that might possibly work.
        if s.check().is_err() {
            panic!("Internal error")
        }
        let size = s.states.len();
        let Graph {
            states: other_states,
            initial: other_initial,
        } = other.map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));
        s.states.extend(other_states);
        s.initial.extend(other_initial);
        s.sort();
        s.determinize().unwrap_or_else(|e| panic!("{e}"))
    }
}

impl<I: Input> ops::Rem<Transition<I, usize>> for Deterministic<I> {
    type Output = Self;
    #[inline]
    fn rem(mut self, rhs: Transition<I, usize>) -> Self::Output {
        for state in &mut self.states {
            if state.non_accepting.is_empty() {
                if let Curry::Scrutinize {
                    ref mut fallback, ..
                } = state.transitions
                {
                    assert!(
                        fallback.is_none(),
                        "Tried to add a fallback transition, \
                        but a fallback already existed.",
                    );
                    *fallback = Some(rhs.clone());
                }
            }
        }
        self
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

        let mut s = self.generalize();
        if s.check().is_err() {
            panic!("Internal error")
        }
        let size = s.states.len();

        let Graph {
            states: other_states,
            initial: other_initial,
        } = other
            .generalize()
            .map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));

        s.states.extend(other_states);

        // For every transition that an empty stack can take from the initial state of the right-hand parser,
        // add that transition (only on the empty stack) to each accepting state of the left-hand parser.
        for state in &mut s.states {
            state.transitions = mem::replace(
                &mut state.transitions,
                Curry::Wildcard(Transition::Return { region: "" }),
            )
            .merge(rhs_init.clone())
            .unwrap_or_else(|e| panic!("{e}"));
        }

        // If any initial states are immediately accepting, we need to start in the second parser, too.
        if s.initial.iter().any(|i| accepting_indices.contains(i)) {
            s.initial.extend(other_initial.iter().copied());
        }

        let mut out = Graph {
            states: s
                .states
                .into_iter()
                .map(|st| add_tail_call_state(st, &other_initial, &accepting_indices))
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
    other_init: &BTreeSet<usize>,
    accepting_indices: &BTreeSet<usize>,
) -> State<I, BTreeSet<usize>> {
    State {
        transitions: add_tail_call_curry(s.transitions, other_init, accepting_indices),
        non_accepting: s.non_accepting,
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_curry<I: Input, C: Ctrl<I>>(
    s: Curry<I, C>,
    other_init: &BTreeSet<usize>,
    accepting_indices: &BTreeSet<usize>,
) -> Curry<I, BTreeSet<usize>> {
    match s {
        Curry::Wildcard(t) => {
            Curry::Wildcard(add_tail_call_transition(t, other_init, accepting_indices))
        }
        Curry::Scrutinize { filter, fallback } => Curry::Scrutinize {
            filter: add_tail_call_range_map(filter, other_init, accepting_indices),
            fallback: fallback.map(|f| add_tail_call_transition(f, other_init, accepting_indices)),
        },
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_range_map<I: Input, C: Ctrl<I>>(
    s: RangeMap<I, C>,
    other_init: &BTreeSet<usize>,
    accepting_indices: &BTreeSet<usize>,
) -> RangeMap<I, BTreeSet<usize>> {
    RangeMap(
        s.0.into_iter()
            .map(|(k, v)| {
                (
                    k,
                    add_tail_call_transition(v, other_init, accepting_indices),
                )
            })
            .collect(),
    )
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_transition<I: Input, C: Ctrl<I>>(
    s: Transition<I, C>,
    other_init: &BTreeSet<usize>,
    accepting_indices: &BTreeSet<usize>,
) -> Transition<I, BTreeSet<usize>> {
    match s {
        Transition::Lateral { ref dst, update } => Transition::Lateral {
            dst: add_tail_call_c(dst, other_init, accepting_indices),
            update,
        },
        Transition::Call {
            region,
            ref detour,
            ref dst,
            combine,
        } => Transition::Call {
            region,
            detour: add_tail_call_c(detour, other_init, accepting_indices),
            dst: add_tail_call_c(dst, other_init, accepting_indices),
            combine,
        },
        Transition::Return { region } => Transition::Return { region },
    }
}

/// Add a tail call only to accepting states.
#[inline]
#[must_use]
fn add_tail_call_c<I: Input, C: Ctrl<I>>(
    c: &C,
    other_init: &BTreeSet<usize>,
    accepting_indices: &BTreeSet<usize>,
) -> BTreeSet<usize> {
    let accepts = c.view().any(|ref i| accepting_indices.contains(i));
    let iter = c.view();
    if accepts {
        iter.chain(other_init.iter().copied()).collect()
    } else {
        iter.collect()
    }
}

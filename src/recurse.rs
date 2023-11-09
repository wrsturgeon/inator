/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Call a tagged state by name.

use core::{iter, ops};
use inator_automata::*;
use std::collections::BTreeSet;

/// Call a tagged state by name.
#[must_use = "Recursion does nothing unless applied to an automaton with the `>>` operator."]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Recurse(String);

/// Check if this state corresponds to an accepting state.
#[inline]
fn accepting(
    r: Result<usize, &str>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> bool {
    match r {
        Ok(i) => accepting_indices.contains(&i),
        Err(tag) => accepting_tags.contains(tag),
    }
}

impl<I: Input, C: Ctrl<I>> ops::Shr<Recurse> for Graph<I, C> {
    type Output = Deterministic<I>;
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    fn shr(self, rhs: Recurse) -> Self::Output {
        let accepting_indices =
            self.states
                .iter()
                .enumerate()
                .fold(BTreeSet::new(), |mut acc_i, (i, s)| {
                    if s.non_accepting.is_empty() {
                        let _ = acc_i.insert(i);
                    }
                    acc_i
                });
        let accepting_tags = self
            .tags
            .iter()
            .filter(|&(_, i)| accepting_indices.contains(i))
            .map(|(k, _)| k.clone())
            .collect();
        let mut out = Graph {
            states: self
                .states
                .into_iter()
                .map(|s| add_tail_call_state(s, &rhs, &accepting_indices, &accepting_tags))
                .collect(),
            initial: self
                .initial
                .view()
                .map(|r| r.map_err(str::to_owned))
                .collect(),
            tags: self.tags,
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
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> State<I, BTreeSet<Result<usize, String>>> {
    State {
        transitions: add_tail_call_curry(s.transitions, r, accepting_indices, accepting_tags),
        non_accepting: s.non_accepting,
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_curry<I: Input, C: Ctrl<I>>(
    s: Curry<I, C>,
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> Curry<I, BTreeSet<Result<usize, String>>> {
    match s {
        Curry::Wildcard(t) => Curry::Wildcard(add_tail_call_transition(
            t,
            r,
            accepting_indices,
            accepting_tags,
        )),
        Curry::Scrutinize(rm) => Curry::Scrutinize(add_tail_call_range_map(
            rm,
            r,
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
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> RangeMap<I, BTreeSet<Result<usize, String>>> {
    RangeMap(
        s.0.into_iter()
            .map(|(k, v)| {
                (
                    k,
                    add_tail_call_transition(v, r, accepting_indices, accepting_tags),
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
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> Transition<I, BTreeSet<Result<usize, String>>> {
    match s {
        Transition::Lateral { ref dst, update } => Transition::Lateral {
            dst: add_tail_call_c(dst, r, accepting_indices, accepting_tags),
            update,
        },
        Transition::Call {
            region,
            ref detour,
            ref dst,
            combine,
        } => Transition::Call {
            region,
            detour: add_tail_call_c(detour, r, accepting_indices, accepting_tags),
            dst: add_tail_call_c(dst, r, accepting_indices, accepting_tags),
            combine,
        },
        Transition::Return { region } => Transition::Return { region },
    }
}

/// Call a tagged state by name.
#[inline]
pub fn recurse(call_by_name: &str) -> Recurse {
    Recurse(call_by_name.to_owned())
}

/// Add a tail call only to accepting states.
#[inline]
#[must_use]
fn add_tail_call_c<I: Input, C: Ctrl<I>>(
    c: &C,
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> BTreeSet<Result<usize, String>> {
    let accepts = c
        .view()
        .any(|rs| accepting(rs, accepting_indices, accepting_tags));
    let iter = c.view().map(|rs| rs.map_err(str::to_owned));
    if accepts {
        iter.chain(iter::once(Err(r.0.clone()))).collect()
    } else {
        iter.collect()
    }
}

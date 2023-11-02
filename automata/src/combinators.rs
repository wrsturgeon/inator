/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on nondeterministic finite automata returning nondeterministic finite automata.

use crate::{
    Ctrl, CurryInput, CurryStack, Graph, Input, Merge, Nondeterministic, RangeMap, Stack, State,
    Transition,
};
use core::ops;
use std::collections::BTreeSet;

impl<I: Input, S: Stack> ops::BitOr for Nondeterministic<I, S> {
    type Output = Self;
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    fn bitor(mut self, other: Self) -> Self {
        // Note that union on pushdown automata is undecidable;
        // we just reject a subset of automata that wouldn't work.
        if self.check().is_err() {
            return self;
        }
        let size = self.states.len();
        let Graph {
            states: other_states,
            initial: other_initial,
            tags: other_tags,
        } = other.map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));
        self.states.extend(other_states);
        self.initial.extend(other_initial);
        self.tags = unwrap!(self.tags.merge(other_tags));
        self.sort() // <-- Not guarantted to sort (almost always) but certainly does remove duplicate states
    }
}

impl<I: Input, S: Stack> ops::Shr for Nondeterministic<I, S> {
    type Output = Self;
    #[inline]
    fn shr(mut self, other: Self) -> Self::Output {
        if self.check().is_err() {
            return self;
        }

        let size = self.states.len();
        let Graph {
            states: other_states,
            initial: other_initial,
            tags: other_tags,
        } = other.map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));

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
        let accepting_tags: BTreeSet<String> = self
            .tags
            .iter()
            .filter(|&(_, v)| v.iter().any(|i| accepting_indices.contains(i)))
            .map(|(k, _)| k.clone())
            .collect();

        self.states.extend(other_states);
        self.tags.extend(other_tags);
        if self.initial.iter().any(|r| {
            will_accept(
                r.as_ref().map_or_else(|s| Err(s.as_str()), |&i| Ok(i)),
                &accepting_indices,
                &accepting_tags,
            )
        }) {
            self.initial.extend(other_initial.iter().cloned());
        }

        Graph {
            states: self
                .states
                .into_iter()
                .map(|s| {
                    add_tail_call_state(s, &other_initial, &accepting_indices, &accepting_tags)
                })
                .collect(),
            ..self
        }
        .sort()
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_state<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: State<I, S, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> State<I, S, BTreeSet<Result<usize, String>>> {
    State {
        transitions: add_tail_call_curry_stack(
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
fn add_tail_call_curry_stack<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: CurryStack<I, S, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> CurryStack<I, S, BTreeSet<Result<usize, String>>> {
    CurryStack {
        wildcard: s
            .wildcard
            .map(|w| add_tail_call_curry_input(w, other_init, accepting_indices, accepting_tags)),
        map_none: s
            .map_none
            .map(|m| add_tail_call_curry_input(m, other_init, accepting_indices, accepting_tags)),
        map_some: s
            .map_some
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    add_tail_call_curry_input(v, other_init, accepting_indices, accepting_tags),
                )
            })
            .collect(),
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_curry_input<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: CurryInput<I, S, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> CurryInput<I, S, BTreeSet<Result<usize, String>>> {
    match s {
        CurryInput::Wildcard(t) => CurryInput::Wildcard(add_tail_call_transition(
            t,
            other_init,
            accepting_indices,
            accepting_tags,
        )),
        CurryInput::Scrutinize(rm) => CurryInput::Scrutinize(add_tail_call_range_map(
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
fn add_tail_call_range_map<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: RangeMap<I, S, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> RangeMap<I, S, BTreeSet<Result<usize, String>>> {
    RangeMap {
        entries: s
            .entries
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    add_tail_call_transition(v, other_init, accepting_indices, accepting_tags),
                )
            })
            .collect(),
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_transition<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: Transition<I, S, C>,
    other_init: &BTreeSet<Result<usize, String>>,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> Transition<I, S, BTreeSet<Result<usize, String>>> {
    let good = s
        .dst
        .view()
        .any(|result| will_accept(result, accepting_indices, accepting_tags));
    let iter = s.dst.view().map(|result| result.map_err(str::to_owned));
    let dst = if good {
        iter.chain(other_init.iter().cloned()).collect()
    } else {
        iter.collect()
    };
    Transition {
        dst,
        act: s.act,
        update: s.update,
    }
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

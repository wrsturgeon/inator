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

impl<I: Input, S: Stack, C: Ctrl<I, S>> ops::Shr<Recurse> for Graph<I, S, C> {
    type Output = Nondeterministic<I, S>;
    #[inline]
    #[allow(clippy::panic)]
    fn shr(self, rhs: Recurse) -> Self::Output {
        println!("Evaluting `... >> Recurse(...`");
        let (accepting_indices, accepting_tags) = self.states.iter().enumerate().fold(
            (BTreeSet::new(), BTreeSet::new()),
            |(mut acc_i, mut acc_t), (i, s)| {
                if s.non_accepting.is_empty() {
                    let _ = acc_i.insert(i);
                    acc_t.extend(s.tags.iter().cloned());
                }
                (acc_i, acc_t)
            },
        );
        println!("Found all accepting indices and tags");
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
        };
        println!("Computed the initial output");
        while out.check_sorted().is_err() {
            println!("Sorted it");
            out = out.sort();
        }
        out
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_state<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: State<I, S, C>,
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> State<I, S, BTreeSet<Result<usize, String>>> {
    State {
        transitions: add_tail_call_curry_stack(s.transitions, r, accepting_indices, accepting_tags),
        non_accepting: s.non_accepting,
        tags: s.tags,
    }
}

/// Add a tail call to any accepting state.
#[inline]
#[must_use]
fn add_tail_call_curry_stack<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: CurryStack<I, S, C>,
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> CurryStack<I, S, BTreeSet<Result<usize, String>>> {
    CurryStack {
        wildcard: s
            .wildcard
            .map(|w| add_tail_call_curry_input(w, r, accepting_indices, accepting_tags)),
        map_none: s
            .map_none
            .map(|m| add_tail_call_curry_input(m, r, accepting_indices, accepting_tags)),
        map_some: s
            .map_some
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    add_tail_call_curry_input(v, r, accepting_indices, accepting_tags),
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
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> CurryInput<I, S, BTreeSet<Result<usize, String>>> {
    match s {
        CurryInput::Wildcard(t) => CurryInput::Wildcard(add_tail_call_transition(
            t,
            r,
            accepting_indices,
            accepting_tags,
        )),
        CurryInput::Scrutinize(rm) => CurryInput::Scrutinize(add_tail_call_range_map(
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
fn add_tail_call_range_map<I: Input, S: Stack, C: Ctrl<I, S>>(
    s: RangeMap<I, S, C>,
    r: &Recurse,
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
                    add_tail_call_transition(v, r, accepting_indices, accepting_tags),
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
    r: &Recurse,
    accepting_indices: &BTreeSet<usize>,
    accepting_tags: &BTreeSet<String>,
) -> Transition<I, S, BTreeSet<Result<usize, String>>> {
    let good = s
        .dst
        .view()
        .any(|result| will_accept(result, accepting_indices, accepting_tags));
    let iter = s.dst.view().map(|result| result.map_err(str::to_owned));
    let dst = if good {
        iter.chain(iter::once(Err(r.0.clone()))).collect()
    } else {
        iter.collect()
    };
    Transition {
        dst,
        act: s.act,
        update: s.update,
    }
}

/*

        match self.transitions.wildcard {
            None => {}
            Some(CurryInput::Wildcard(ref mut w)) => if w.dst.view().any(will_accept) {},
            Some(CurryInput::Scrutinize(ref mut s)) => todo!(),
        }
        let path_to_accepting = state
            .transitions
            .values()
            .flat_map(|curry| curry.values())
            .map(|transition| transition);
        for curry in state.transitions.values() {
            for shit in curry.values() {}
        }


*/

/// Call a tagged state by name.
#[inline]
pub fn recurse(call_by_name: &str) -> Recurse {
    Recurse(call_by_name.to_owned())
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{
    merge, Check, Ctrl, CurryInput, CurryStack, IllFormed, Input, Output, ParseError, RangeMap,
    Stack, State, Transition,
};
use core::num::NonZeroUsize;
use std::{
    collections::{btree_map, BTreeMap, BTreeSet},
    ffi::OsStr,
    fs, io,
    path::Path,
    process::Command,
};

/// One token corresponds to at most one transition.
pub type Deterministic<I, S, O> = Graph<I, S, O, usize>;

/// One token corresponds to as many transitions as it would like;
/// if any of these transitions eventually accept, the whole thing accepts.
pub type Nondeterministic<I, S, O> = Graph<I, S, O, BTreeSet<usize>>;

/// Automaton loosely based on visibly pushdown automata.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Eq, PartialEq)]
pub struct Graph<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Every state, indexed.
    pub states: Vec<State<I, S, O, C>>,
    /// Initial state of the machine (before reading input).
    pub initial: C,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Clone for Graph<I, S, O, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            initial: self.initial.clone(),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Graph<I, S, O, C> {
    /// Check well-formedness.
    /// # Errors
    /// When not well-formed (with a witness).
    #[inline]
    pub fn check(&self) -> Result<(), IllFormed<I, S, O, C>> {
        let n_states = self.states.len();
        if let Some(i) = self
            .initial
            .view()
            .fold(None, |acc, i| acc.or_else(|| (i >= n_states).then_some(i)))
        {
            return Err(IllFormed::OutOfBounds(i));
        }
        NonZeroUsize::new(n_states).map_or(Ok(()), |nz| {
            self.states.iter().try_fold((), |(), state| state.check(nz))
        })
    }

    /// Run this parser to completion and check the result.
    /// # Errors
    /// If the parser determines there should be an error.
    #[inline]
    #[allow(unsafe_code)]
    pub fn accept<In: IntoIterator<Item = I>>(
        &self,
        input: In,
    ) -> Result<Option<O>, ParseError<I, S, O, C>> {
        use crate::Run;
        let mut run = input.run(self);
        for r in &mut run {
            drop(r?);
        }
        let output = run
            .ctrl
            .view()
            .any(|i| get!(self.states, i).accepting)
            .then(|| {
                // SAFETY: Never uninitialized except inside one function (and initialized before it exits).
                unsafe { run.output.assume_init() }
            });
        Ok(output)
    }

    /// Subset construction algorithm for determinizing nondeterministic automata.
    /// # Errors
    /// If there's an ambiguity (which would have crashed the nondeterministic automaton anyway).
    #[inline]
    #[allow(
        clippy::missing_panics_doc,
        clippy::type_complexity,
        clippy::unwrap_in_result
    )]
    pub fn determinize(&self) -> Result<Deterministic<I, S, O>, IllFormed<I, S, O, C>> {
        // Check that the source graph is well-formed
        self.check()?;

        // Associate each subset of states with a merged state
        let mut subsets_as_states = BTreeMap::new();
        self.explore(&mut subsets_as_states, &self.initial)?;

        // Fix an ordering on those subsets
        let mut ordering: Vec<C> = subsets_as_states.keys().cloned().collect();
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
                        transitions: fix_indices_curry_stack(transitions, &ordering),
                        accepting,
                    }
                })
                .collect(),
        })
    }

    /// Associate each subset of states with a merged state.
    fn explore(
        &self,
        subsets_as_states: &mut BTreeMap<C, State<I, S, O, C>>,
        subset: &C,
    ) -> Result<(), IllFormed<I, S, O, C>> {
        // Check if we've seen this subset already
        let btree_map::Entry::Vacant(entry) = subsets_as_states.entry(subset.clone()) else {
            return Ok(());
        };

        #[allow(clippy::print_stdout)]
        {
            println!("Merging {}", subset.to_src());
        }

        // Merge this subset of states into one (most of the heavy lifting)
        let mega_state: State<I, S, O, C> =
            match merge(subset.view().map(|i| get!(self.states, i)).cloned()) {
                // If no state follows, reject immediately.
                None => State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
                // If they successfully merged, return the merged state
                Some(Ok(ok)) => ok,
                // If they didn't successfully merge, something's wrong with the original automaton
                Some(Err(e)) => return Err(e),
            };

        // Cache all possible next states
        #[allow(clippy::needless_collect)] // <-- false positive: couldn't move `mega_state` below
        let dsts: BTreeSet<C> = mega_state
            .transitions
            .values()
            .flat_map(CurryInput::values)
            .map(|transition| transition.dst.clone())
            .collect();

        // Associate this subset of states with the merged state
        let _ = entry.insert(mega_state);

        // Recurse on all destinations
        dsts.into_iter()
            .try_fold((), |(), dst| self.explore(subsets_as_states, &dst))
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_curry_stack<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    value: CurryStack<I, S, O, C>,
    ordering: &[C],
) -> CurryStack<I, S, O, usize> {
    CurryStack {
        wildcard: value
            .wildcard
            .map(|wild| fix_indices_curry_input(wild, ordering)),
        map_none: value
            .map_none
            .map(|none| fix_indices_curry_input(none, ordering)),
        map_some: value
            .map_some
            .into_iter()
            .map(|(arg, etc)| (arg, fix_indices_curry_input(etc, ordering)))
            .collect(),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_curry_input<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    value: CurryInput<I, S, O, C>,
    ordering: &[C],
) -> CurryInput<I, S, O, usize> {
    match value {
        CurryInput::Wildcard(etc) => CurryInput::Wildcard(fix_indices_transition(etc, ordering)),
        CurryInput::Scrutinize(etc) => CurryInput::Scrutinize(fix_indices_range_map(etc, ordering)),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_range_map<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    value: RangeMap<I, S, O, C>,
    ordering: &[C],
) -> RangeMap<I, S, O, usize> {
    RangeMap {
        entries: value
            .entries
            .into_iter()
            .map(|(k, v)| (k, fix_indices_transition(v, ordering)))
            .collect(),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_transition<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    value: Transition<I, S, O, C>,
    ordering: &[C],
) -> Transition<I, S, O, usize> {
    Transition {
        dst: unwrap!(ordering.binary_search(&value.dst)),
        act: value.act,
        update: value.update,
    }
}

impl<I: Input, S: Stack, O: Output> Graph<I, S, O, usize> {
    /// Write this parser as a Rust source file.
    /// # Errors
    /// If file creation or formatting fails.
    #[inline]
    pub fn to_file<P: AsRef<OsStr> + AsRef<Path>>(&self, path: P) -> io::Result<()> {
        fs::write(&path, self.to_src())?;
        Command::new("rustfmt").arg(path).output().map(|_| {})
    }
}

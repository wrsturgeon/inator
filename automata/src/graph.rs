/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{
    merge, Check, CmpFirst, Ctrl, CurryInput, CurryStack, IllFormed, Input, Output, ParseError,
    RangeMap, Stack, State, Transition,
};
use core::{
    cmp::Ordering,
    mem::{transmute, MaybeUninit},
    num::NonZeroUsize,
};
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
#[derive(Debug)]
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

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Eq for Graph<I, S, O, C> {}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialEq for Graph<I, S, O, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.initial == other.initial && self.states == other.states
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
            // Check sorted without duplicates
            let _ = self.states.iter().try_fold(None, |mlast, curr| {
                mlast.map_or(Ok(Some(curr)), |last: &State<I, S, O, C>| {
                    match last.cmp(curr) {
                        Ordering::Less => Ok(Some(curr)),
                        Ordering::Equal => Err(IllFormed::DuplicateState),
                        Ordering::Greater => Err(IllFormed::UnsortedStates),
                    }
                })
            })?;
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
    #[inline]
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

    /// Chop off parts of the automaton until it's valid.
    #[inline]
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        clippy::missing_panics_doc,
        clippy::transmute_undefined_repr,
        unsafe_code
    )]
    #[allow(clippy::print_stdout, clippy::use_debug)] // <-- TODO
    pub fn procrustes(mut self) -> Option<Self> {
        'restart: loop {
            let Some(size) = NonZeroUsize::new(self.states.len()) else {
                return self.check().is_ok().then_some(self);
            };
            let mut reindexed: Vec<_> = self
                .states
                .into_iter()
                .enumerate()
                .map(|(i, s)| CmpFirst(s, i))
                .collect();
            reindexed.sort_unstable();
            let (indices, orig_states): (Vec<_>, Vec<_>) =
                reindexed.into_iter().map(|CmpFirst(s, i)| (i, s)).unzip();
            let mut states = orig_states
                .into_iter()
                .map(|state| {
                    MaybeUninit::new(
                        state
                            .map_indices(|i| {
                                let r = i % size;
                                unwrap!(indices.iter().position(|&j| r == j))
                            })
                            .procrustes(),
                    )
                })
                .collect::<Vec<_>>();
            let mut initial = MaybeUninit::new(self.initial.map_indices(|i| {
                let r = i % size;
                unwrap!(indices.iter().position(|&j| r == j))
            }));
            if !indices.iter().copied().eq(0..size.into()) {
                // SAFETY: ABI guaranteed to be identical.
                self.states = unsafe { transmute(states) };
                // SAFETY: Never uninitialized except inside `dedup` above.
                self.initial = unsafe { initial.assume_init() };
                continue 'restart;
            }
            dedup(&mut states, &mut initial);
            return Some(Self {
                // SAFETY: ABI guaranteed to be identical.
                states: unsafe { transmute(states) },
                // SAFETY: Never uninitialized except inside `dedup` above.
                initial: unsafe { initial.assume_init() },
            });
        }
    }
}

/// Remove the first duplicate state, if any, AND adjust indices accordingly.
#[inline]
#[allow(unsafe_code, unused_unsafe)]
fn dedup<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    states: &mut Vec<MaybeUninit<State<I, S, O, C>>>,
    initial: &mut MaybeUninit<C>,
) {
    let mut i = 0;
    while i < states.len().saturating_sub(1) {
        match {
            // SAFETY: Never uninitialized except below.
            let curr = unsafe { get!(states, i).assume_init_ref() };
            i = i.checked_add(1).expect("Absurdly huge number of states");
            // SAFETY: Never uninitialized except below.
            let next = unsafe { get!(states, i).assume_init_ref() };
            curr.cmp(next)
        } {
            Ordering::Less => continue,
            Ordering::Equal => {
                let _ = states.remove(i);
                for state in &mut *states {
                    // SAFETY: Never uninitialized except right here.
                    let _ = state.write(unsafe { state.assume_init_read() }.map_indices(|j| {
                        if j < i {
                            j
                        } else {
                            j.overflowing_sub(1).0
                        }
                    }));
                }
                // SAFETY: Never uninitialized except right here.
                let _ = initial.write(unsafe { initial.assume_init_read() }.map_indices(|j| {
                    if j < i {
                        j
                    } else {
                        j.overflowing_sub(1).0
                    }
                }));
            }
            Ordering::Greater => never!(),
        }
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
            .map(|CmpFirst(k, v)| CmpFirst(k, fix_indices_transition(v, ordering)))
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

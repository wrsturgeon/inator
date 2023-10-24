/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{
    try_merge, Check, Ctrl, CurryInput, CurryStack, IllFormed, Input, InputError, Output,
    ParseError, RangeMap, Stack, State, Transition,
};
use core::{cmp::Ordering, num::NonZeroUsize};
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
pub type Nondeterministic<I, S, O> = Graph<I, S, O, BTreeSet<Result<usize, String>>>;

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
    /// Note that this can't check if determinization will succeed in less time than actually trying;
    /// if you want to see if there will be any runtime errors, just determinize it.
    /// # Errors
    /// When ill-formed (with a witness).
    #[inline]
    pub fn check(&self) -> Result<(), IllFormed<I, S, O, C>> {
        let n_states = self.states.len();
        if let Some(i) = self.initial.view().fold(None, |acc, r| {
            acc.or_else(|| r.map_or(None, |i| (i >= n_states).then_some(i)))
        }) {
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
    ) -> Result<O, ParseError<I, S, O, C>> {
        use crate::Run;
        let mut run = input.run(self);
        for r in &mut run {
            drop(r?);
        }
        for r in run.ctrl.view() {
            if (match r {
                Ok(i) => get!(self.states, i),
                Err(s) => find_tag(&self.states, s).map_err(ParseError::BadParser)?,
            })
            .accepting
            {
                // SAFETY: Never uninitialized except inside one function (and initialized before it exits).
                return Ok(unsafe { run.output.assume_init() });
            }
        }
        Err(ParseError::BadInput(InputError::NotAccepting))
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

        let output = Deterministic {
            initial: unwrap!(ordering.binary_search(&self.initial)),
            states: ordering
                .iter()
                .map(|set| {
                    let State {
                        transitions,
                        accepting,
                        tag,
                    } = unwrap!(subsets_as_states.remove(set));
                    State {
                        transitions: fix_indices_curry_stack(transitions, &ordering),
                        accepting,
                        tag,
                    }
                })
                .collect(),
        };
        output
            .check()
            .map_err(IllFormed::convert_ctrl)
            .map(|()| output)
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

        // Merge this subset of states into one (most of the heavy lifting)
        let mega_state: State<I, S, O, C> = match try_merge(subset.view().map(|r| match r {
            Ok(i) => Ok(get!(self.states, i).clone()),
            Err(s) => find_tag(&self.states, s).map(Clone::clone),
        })) {
            // If no state follows, reject immediately.
            None => State {
                transitions: CurryStack {
                    wildcard: None,
                    map_none: None,
                    map_some: BTreeMap::new(),
                },
                accepting: false,
                tag: vec![],
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

/// Look up a tag and return the specific state tagged with it.
/// # Errors
/// If no state has this tag, or if multiple have this tag.
#[inline]
#[allow(clippy::type_complexity)]
pub fn find_tag<'s, I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    states: &'s [State<I, S, O, C>],
    tag: &str,
) -> Result<&'s State<I, S, O, C>, IllFormed<I, S, O, C>> {
    let mut acc = None;
    for state in states {
        if state.tag.iter().any(|s| s == tag) {
            match acc {
                None => acc = Some(state),
                Some(..) => return Err(IllFormed::DuplicateTag(tag.to_owned())),
            }
        }
    }
    acc.ok_or(IllFormed::TagDNE(tag.to_owned()))
}

/// Look up a tag and return the specific state tagged with it.
/// # Errors
/// If no state has this tag, or if multiple have this tag.
#[inline]
#[allow(clippy::type_complexity)]
pub fn find_tag_mut<'s, I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>>(
    states: &'s mut [State<I, S, O, C>],
    tag: &str,
) -> Result<&'s mut State<I, S, O, C>, IllFormed<I, S, O, C>> {
    let mut acc = None;
    for state in states {
        if state.tag.iter().any(|s| s == tag) {
            match acc {
                None => acc = Some(state),
                Some(..) => return Err(IllFormed::DuplicateTag(tag.to_owned())),
            }
        }
    }
    acc.ok_or(IllFormed::TagDNE(tag.to_owned()))
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

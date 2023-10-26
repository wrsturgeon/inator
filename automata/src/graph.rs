/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{
    try_merge, Check, Ctrl, CurryInput, CurryStack, IllFormed, Input, InputError, ParseError,
    RangeMap, Stack, State, Transition,
};
use core::{cmp::Ordering, num::NonZeroUsize};
use std::{
    any::Any,
    collections::{btree_map, BTreeMap, BTreeSet},
    ffi::OsStr,
    fs, io,
    path::Path,
    process::Command,
};

/// One token corresponds to at most one transition.
pub type Deterministic<I, S> = Graph<I, S, usize>;

/// One token corresponds to as many transitions as it would like;
/// if any of these transitions eventually accept, the whole thing accepts.
pub type Nondeterministic<I, S> = Graph<I, S, BTreeSet<Result<usize, String>>>;

/// Automaton loosely based on visibly pushdown automata.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct Graph<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Every state, indexed.
    pub states: Vec<State<I, S, C>>,
    /// Initial state of the machine (before reading input).
    pub initial: C,
    /// Type of output after any successful run.
    pub output_t: String,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for Graph<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            initial: self.initial.clone(),
            output_t: self.output_t.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Eq for Graph<I, S, C> {}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialEq for Graph<I, S, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.initial == other.initial && self.states == other.states
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Graph<I, S, C> {
    /// Check well-formedness.
    /// Note that this can't check if determinization will succeed in less time than actually trying;
    /// if you want to see if there will be any runtime errors, just determinize it.
    /// # Errors
    /// When ill-formed (with a witness).
    #[inline]
    pub fn check(&self) -> Result<(), IllFormed<I, S, C>> {
        let n_states = self.states.len();
        for r in self.initial.view() {
            let state = match r {
                Ok(i) => {
                    if let Some(state) = self.states.get(i) {
                        state
                    } else {
                        return Err(IllFormed::OutOfBounds(i));
                    }
                }
                Err(tag) => find_tag(&self.states, tag)?,
            };
            if state.input_t != "()" {
                return Err(IllFormed::InitialNotUnit(state.input_t.clone()));
            }
            for curry in state.transitions.values() {
                for transition in curry.values() {
                    if transition.update.input_t != "()" {
                        return Err(IllFormed::InitialNotUnit(transition.update.input_t.clone()));
                    }
                }
            }
        }
        NonZeroUsize::new(n_states).map_or(Ok(()), |nz| {
            // Check sorted without duplicates
            let _ = self.states.iter().try_fold(None, |mlast, curr| {
                mlast.map_or(Ok(Some(curr)), |last: &State<I, S, C>| {
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
    ) -> Result<Box<dyn Any>, ParseError<I, S, C>> {
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
                return Ok(run.output.unwrap());
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
    pub fn determinize(&self) -> Result<Deterministic<I, S>, IllFormed<I, S, C>> {
        // Check that the source graph is well-formed
        self.check()?;

        // Associate each subset of states with a merged state
        let mut subsets_as_states = BTreeMap::new();
        self.explore(&mut subsets_as_states, &self.initial, "()")?;

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
                        input_t,
                    } = unwrap!(subsets_as_states.remove(set));
                    State {
                        transitions: fix_indices_curry_stack(transitions, &ordering),
                        accepting,
                        tag,
                        input_t,
                    }
                })
                .collect(),
            output_t: self.output_t.clone(),
        };
        output
            .check()
            .map(|()| output)
            .map_err(IllFormed::convert_ctrl)
    }

    /// Associate each subset of states with a merged state.
    #[inline]
    fn explore(
        &self,
        subsets_as_states: &mut BTreeMap<C, State<I, S, C>>,
        subset: &C,
        input_t: &str,
    ) -> Result<(), IllFormed<I, S, C>> {
        // Check if we've seen this subset already
        let btree_map::Entry::Vacant(entry) = subsets_as_states.entry(subset.clone()) else {
            return Ok(());
        };

        // Merge this subset of states into one (most of the heavy lifting)
        let mega_state: State<I, S, C> = match try_merge(subset.view().map(|r| match r {
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
                tag: BTreeSet::new(),
                input_t: input_t.to_owned(),
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
        let next_input_t = mega_state.input_t.clone();

        // Associate this subset of states with the merged state
        let _ = entry.insert(mega_state);

        // Recurse on all destinations
        dsts.into_iter().try_fold((), |(), dst| {
            self.explore(subsets_as_states, &dst, &next_input_t)
        })
    }
}

/// Look up a tag and return the specific state tagged with it.
/// # Errors
/// If no state has this tag, or if multiple have this tag.
#[inline]
#[allow(clippy::type_complexity)]
pub fn find_tag<'s, I: Input, S: Stack, C: Ctrl<I, S>>(
    states: &'s [State<I, S, C>],
    tag: &str,
) -> Result<&'s State<I, S, C>, IllFormed<I, S, C>> {
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
pub fn find_tag_mut<'s, I: Input, S: Stack, C: Ctrl<I, S>>(
    states: &'s mut [State<I, S, C>],
    tag: &str,
) -> Result<&'s mut State<I, S, C>, IllFormed<I, S, C>> {
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
fn fix_indices_curry_stack<I: Input, S: Stack, C: Ctrl<I, S>>(
    value: CurryStack<I, S, C>,
    ordering: &[C],
) -> CurryStack<I, S, usize> {
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
fn fix_indices_curry_input<I: Input, S: Stack, C: Ctrl<I, S>>(
    value: CurryInput<I, S, C>,
    ordering: &[C],
) -> CurryInput<I, S, usize> {
    match value {
        CurryInput::Wildcard(etc) => CurryInput::Wildcard(fix_indices_transition(etc, ordering)),
        CurryInput::Scrutinize(etc) => CurryInput::Scrutinize(fix_indices_range_map(etc, ordering)),
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_range_map<I: Input, S: Stack, C: Ctrl<I, S>>(
    value: RangeMap<I, S, C>,
    ordering: &[C],
) -> RangeMap<I, S, usize> {
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
fn fix_indices_transition<I: Input, S: Stack, C: Ctrl<I, S>>(
    value: Transition<I, S, C>,
    ordering: &[C],
) -> Transition<I, S, usize> {
    Transition {
        dst: unwrap!(ordering.binary_search(&value.dst)),
        act: value.act,
        update: value.update,
    }
}

impl<I: Input, S: Stack> Graph<I, S, usize> {
    /// Write this parser as a Rust source file.
    /// # Errors
    /// If file creation or formatting fails.
    #[inline]
    pub fn to_file<P: AsRef<OsStr> + AsRef<Path>>(&self, path: P) -> io::Result<()> {
        fs::write(&path, self.to_src())?;
        Command::new("rustfmt").arg(path).output().map(|_| {})
    }
}

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
use core::{cmp::Ordering, iter, mem, num::NonZeroUsize};
use std::{
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

// TODO: make `states` a `BTreeSet`.

/// Automaton loosely based on visibly pushdown automata.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct Graph<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Every state, indexed.
    pub states: Vec<State<I, S, C>>,
    /// Initial state of the machine (before reading input).
    pub initial: C,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for Graph<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            initial: self.initial.clone(),
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
    /// Check a subset of well-formedness.
    /// Note that this can't check if determinization will succeed in less time than actually trying;
    /// if you want to see if there can be any runtime errors, just try to determinize it.
    /// # Errors
    /// When ill-formed (with a witness).
    #[inline]
    pub fn check(&self) -> Result<(), IllFormed<I, S, C>> {
        let n_states = self.states.len();
        for r in self.initial.view() {
            let states = match r {
                Ok(i) => {
                    if let Some(state) = self.states.get(i) {
                        iter::once(state).collect()
                    } else {
                        return Err(IllFormed::OutOfBounds(i));
                    }
                }
                Err(tags) => find_tag(&self.states, tags)?,
            };
            let typed = states.into_iter().try_fold(vec![], |mut acc, s| {
                acc.push((s, s.input_type()?));
                Ok(acc)
            })?;
            for (state, input_t) in typed {
                #[allow(clippy::match_same_arms)] // TBD
                match input_t {
                    None => {}
                    Some(t) if t == "()" => {}
                    Some(other) => return Err(IllFormed::InitialNotUnit(other)),
                }
                for curry in state.transitions.values() {
                    for transition in curry.values() {
                        if transition.update.input_t != "()" {
                            return Err(IllFormed::InitialNotUnit(
                                transition.update.input_t.clone(),
                            ));
                        }
                    }
                }
            }
        }
        drop(self.output_type()?);
        NonZeroUsize::new(n_states).map_or(Ok(()), |nz| {
            // Check sorted without duplicates
            self.check_sorted()?;
            self.states.iter().try_fold((), |(), state| state.check(nz))
        })
    }

    /// Run this parser to completion and check types along the way.
    /// # Errors
    /// If the parser determines there should be an error.
    #[inline]
    #[allow(unsafe_code)]
    pub fn accept<In: IntoIterator<Item = I>>(
        &self,
        input: In,
    ) -> Result<String, ParseError<I, S, C>> {
        use crate::Run;
        let mut run = input.run(self);
        for r in &mut run {
            drop(r?);
        }
        for r in run.ctrl.view() {
            if match r {
                Ok(i) => iter::once(get!(self.states, i)).collect(),
                Err(s) => find_tag(&self.states, s).map_err(ParseError::BadParser)?,
            }
            .into_iter()
            .any(|s| s.non_accepting.is_empty())
            {
                return Ok(run.output_t);
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
                        non_accepting,
                        tags,
                    } = unwrap!(subsets_as_states.remove(set));
                    State {
                        transitions: fix_indices_curry_stack(transitions, &ordering),
                        non_accepting,
                        tags,
                    }
                })
                .collect(),
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
    ) -> Result<(), IllFormed<I, S, C>> {
        // Check if we've seen this subset already
        let btree_map::Entry::Vacant(entry) = subsets_as_states.entry(subset.clone()) else {
            return Ok(());
        };

        // Merge this subset of states into one (most of the heavy lifting)
        let result_iterator = subset.view().flat_map(|r| match r {
            Ok(i) => iter::once(Ok(get!(self.states, i).clone())).collect(),
            Err(s) => find_tag(&self.states, s).map_or_else(
                |e| vec![Err(e)],
                |set| set.into_iter().map(|st| Ok(st.clone())).collect(),
            ),
        });
        let mega_state: State<I, S, C> = match try_merge(result_iterator) {
            // If no state follows, reject immediately.
            None => State {
                transitions: CurryStack {
                    wildcard: None,
                    map_none: None,
                    map_some: BTreeMap::new(),
                },
                non_accepting: iter::once("Unexpected token".to_owned()).collect(),
                tags: BTreeSet::new(),
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

    /// Compute the output type of any successful run.
    /// # Errors
    /// If multiple accepting states attempt to return different types.
    #[inline]
    pub fn output_type(&self) -> Result<Option<String>, IllFormed<I, S, C>> {
        self.states
            .iter()
            .try_fold(None, |acc: Option<String>, state| {
                if state.non_accepting.is_empty() {
                    acc.map_or_else(
                        || state.input_type(),
                        |t| {
                            if let Some(input_t) = state.input_type()? {
                                if input_t != t {
                                    return Err(IllFormed::WrongReturnType(t, input_t));
                                }
                            }
                            Ok(Some(t))
                        },
                    )
                } else {
                    Ok(acc)
                }
            })
    }

    /// Compute the output type of any successful run.
    /// # Errors
    /// If multiple accepting states attempt to return different types.
    #[inline]
    pub fn input_type(&self) -> Result<Option<String>, IllFormed<I, S, C>> {
        self.states
            .iter()
            .try_fold(None, |acc: Option<String>, state| {
                Ok(state.input_type()?.or(acc))
            })
    }

    /// Check if states are sorted.
    /// # Errors
    /// If there are duplicate states or any out of order.
    #[inline]
    pub fn check_sorted(&self) -> Result<(), IllFormed<I, S, C>> {
        self.states
            .iter()
            .try_fold(None, |mlast, curr| {
                mlast.map_or(Ok(Some(curr)), |last: &State<I, S, C>| {
                    match last.cmp(curr) {
                        Ordering::Less => Ok(Some(curr)),
                        Ordering::Equal => Err(IllFormed::DuplicateState),
                        Ordering::Greater => Err(IllFormed::UnsortedStates),
                    }
                })
            })
            .map(|_| {})
    }

    /// Change nothing about the semantics but sort the internal vector of states.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn sort(mut self) -> Nondeterministic<I, S> {
        // Remove all tags but retain their associations
        // (in case two otherwise identical states have different tags)
        let mut tag_map = BTreeMap::new();
        for state in &mut self.states {
            let untagged = State {
                transitions: state.transitions.clone(),
                non_accepting: state.non_accepting.clone(),
                tags: BTreeSet::new(),
            };
            let tags = mem::take(&mut state.tags);
            tag_map
                .entry(untagged)
                .or_insert(BTreeSet::new())
                .extend(tags);
        }
        #[cfg(any(test, debug))]
        for state in &self.states {
            assert_eq!(state.tags, BTreeSet::new(), "Should have emptied tags");
        }

        // Associate each original index with a concrete state instead of just an index,
        // since we're going to be swapping the indices around.
        let index_map: BTreeMap<usize, State<_, _, _>> =
            self.states.iter().cloned().enumerate().collect();
        self.states.sort_unstable();
        self.states.dedup(); // <-- Cool that we can do this!
        #[allow(unused_unsafe)] // Nested macros
        let initial = self
            .initial
            .view()
            .map(|r| {
                r.map_err(str::to_owned)
                    .map(|i| unwrap!(self.states.binary_search(unwrap!(index_map.get(&i)))))
            })
            .collect();
        let states = self
            .states
            .iter()
            .map(|s| State {
                tags: unwrap!(tag_map.remove(s)),
                ..s.reindex(&self.states, &index_map)
            })
            .collect();
        Graph { states, initial }
    }
}

/// Look up a tag and return the specific state tagged with it.
/// # Errors
/// If no state has this tags, or if multiple have this tags.
#[inline]
#[allow(clippy::type_complexity)]
pub fn find_tag<'s, I: Input, S: Stack, C: Ctrl<I, S>>(
    states: &'s [State<I, S, C>],
    tag: &str,
) -> Result<BTreeSet<&'s State<I, S, C>>, IllFormed<I, S, C>> {
    let acc = states.iter().fold(BTreeSet::new(), |mut acc, s| {
        if s.tags.iter().any(|st| st == tag) {
            let _ = acc.insert(s);
        }
        acc
    });
    if acc.is_empty() {
        Err(IllFormed::TagDNE(tag.to_owned()))
    } else {
        Ok(acc)
    }
}

/// Look up a tag and return the specific state tagged with it.
/// # Errors
/// If no state has this tags, or if multiple have this tags.
#[inline]
#[allow(clippy::type_complexity)]
pub fn find_tag_mut<'s, I: Input, S: Stack, C: Ctrl<I, S>>(
    states: &'s mut [State<I, S, C>],
    tag: &str,
) -> Result<BTreeSet<&'s mut State<I, S, C>>, IllFormed<I, S, C>> {
    #[allow(clippy::mutable_key_type)]
    let acc = states.iter_mut().fold(BTreeSet::new(), |mut acc, s| {
        if s.tags.iter().any(|st| st == tag) {
            let _ = acc.insert(s);
        }
        acc
    });
    if acc.is_empty() {
        Err(IllFormed::TagDNE(tag.to_owned()))
    } else {
        Ok(acc)
    }
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
    pub fn to_file<P: AsRef<OsStr> + AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<io::Result<()>, IllFormed<I, S, usize>> {
        self.to_src().map(|src| {
            fs::write(&path, src)?;
            Command::new("rustfmt").arg(path).output().map(|_| {})
        })
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{
    try_merge, Check, Ctrl, Curry, IllFormed, Input, InputError, Merge, ParseError, RangeMap,
    State, Transition,
};
use core::{iter, num::NonZeroUsize};
use std::{
    collections::{btree_map, BTreeMap, BTreeSet},
    ffi::OsStr,
    fs, io,
    path::Path,
    process::Command,
};

/// One token corresponds to at most one transition.
pub type Deterministic<I> = Graph<I, usize>;

/// One token corresponds to as many transitions as it would like;
/// if any of these transitions eventually accept, the whole thing accepts.
pub type Nondeterministic<I> = Graph<I, BTreeSet<usize>>;

// TODO: make `states` a `BTreeSet`.

/// Automaton loosely based on visibly pushdown automata.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct Graph<I: Input, C: Ctrl<I>> {
    /// Every state, indexed.
    pub states: Vec<State<I, C>>,
    /// Initial state of the machine (before reading input).
    pub initial: C,
}

impl<I: Input, C: Ctrl<I>> Clone for Graph<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            initial: self.initial.clone(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Eq for Graph<I, C> {}

impl<I: Input, C: Ctrl<I>> PartialEq for Graph<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.initial == other.initial && self.states == other.states
    }
}

impl<I: Input, C: Ctrl<I>> Graph<I, C> {
    /// Check a subset of well-formedness.
    /// Note that this can't check if determinization will succeed in less time than actually trying;
    /// if you want to see if there can be any runtime errors, just try to determinize it.
    /// # Errors
    /// When ill-formed (with a witness).
    #[inline]
    pub fn check(&self) -> Result<(), IllFormed<I, C>> {
        let n_states = self.states.len();
        for i in self.initial.view() {
            let Some(state) = self.states.get(i) else {
                return Err(IllFormed::OutOfBounds(i));
            };
            if let Some(t) = state.input_type()? {
                if t != "()" {
                    return Err(IllFormed::InitialNotUnit(t.to_owned()));
                }
            }
            for transition in state.transitions.values() {
                let in_t = transition.input_type();
                if let Some(t) = in_t {
                    if t != "()" {
                        return Err(IllFormed::InitialNotUnit(t.to_owned()));
                    }
                }
            }
        }
        let _ = self.output_type()?;
        for (i, state) in self.states.iter().enumerate() {
            if get!(self.states, ..i).contains(state) {
                return Err(IllFormed::DuplicateState(Box::new(state.clone())));
            }
        }
        NonZeroUsize::new(n_states).map_or(Ok(()), |nz| {
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
    ) -> Result<String, ParseError<I, C>> {
        use crate::Run;
        let mut run = input.run(self);
        for r in &mut run {
            drop(r?);
        }
        for i in run.ctrl.view() {
            if get!(self.states, i).non_accepting.is_empty() {
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
    pub fn determinize(&self) -> Result<Deterministic<I>, IllFormed<I, C>> {
        // Check that the source graph is well-formed
        self.check()?;

        // Associate each subset of states with a merged state
        let mut subsets_as_states = BTreeMap::new();
        self.explore(&mut subsets_as_states, &self.initial)?;

        // Fix an ordering on those subsets
        let ordering: Vec<C> = subsets_as_states.keys().cloned().collect();
        // Don't need to sort--that's guaranteed in `BTreeMap::keys`

        let mut output = Deterministic {
            initial: unwrap!(ordering.binary_search(&self.initial)),
            states: ordering
                .iter()
                .map(|set| {
                    let State {
                        transitions,
                        non_accepting,
                    } = unwrap!(subsets_as_states.remove(set));
                    State {
                        transitions: fix_indices_curry(transitions, &ordering),
                        non_accepting,
                    }
                })
                .collect(),
        };
        output.sort();
        output
            .check()
            .map(|()| output)
            .map_err(IllFormed::convert_ctrl)
    }

    /// Associate each subset of states with a merged state.
    #[inline]
    fn explore(
        &self,
        subsets_as_states: &mut BTreeMap<C, State<I, C>>,
        subset: &C,
    ) -> Result<(), IllFormed<I, C>> {
        // Check if we've seen this subset already
        let btree_map::Entry::Vacant(entry) = subsets_as_states.entry(subset.clone()) else {
            return Ok(());
        };

        // Merge this subset of states into one (most of the heavy lifting)
        let result_iterator = subset.view().map(|i| Ok(get!(self.states, i).clone()));

        let mega_state = match try_merge(result_iterator) {
            // If no state follows, reject immediately.
            None => State {
                transitions: Curry::Scrutinize {
                    filter: RangeMap(BTreeMap::new()),
                    fallback: None,
                },
                non_accepting: iter::once("Unexpected token".to_owned()).collect(),
            },
            // If they successfully merged, return the merged state
            Some(Ok(ok)) => ok,
            // If they didn't successfully merge, something's wrong with the original automaton
            Some(Err(e)) => return Err(e),
        };

        // Necessary before we move `mega_state`
        let all_dsts: BTreeSet<C> = mega_state
            .transitions
            .values()
            .flat_map(|t| t.dsts().into_iter().cloned())
            .collect();

        // Insert the finished value (also to tell all below iterations that we've covered this case)
        let _ = entry.insert(mega_state);

        // Recurse on all possible next states
        all_dsts
            .into_iter()
            .try_fold((), |(), dst| self.explore(subsets_as_states, &dst))
    }

    /// Compute the output type of any successful run.
    /// # Errors
    /// If multiple accepting states attempt to return different types.
    #[inline]
    pub fn output_type(&self) -> Result<Option<&str>, IllFormed<I, C>> {
        self.states
            .iter()
            .try_fold(None, |acc: Option<&str>, state| {
                if state.non_accepting.is_empty() {
                    acc.map_or_else(
                        || state.input_type(),
                        |t| {
                            if let Some(input_t) = state.input_type()? {
                                if input_t != t {
                                    return Err(IllFormed::WrongReturnType(
                                        t.to_owned(),
                                        input_t.to_owned(),
                                    ));
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

    /// Compute the input type of any successful run.
    /// # Errors
    /// If multiple accepting states attempt to return different types.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn input_type(&self) -> Result<Option<&str>, IllFormed<I, C>> {
        self.initial
            .view()
            .map(|i| get!(self.states, i))
            .try_fold(None, |acc, state| {
                let shit = acc.merge(state.transitions.values().try_fold(None, |accc, t| {
                    accc.merge(t.input_type()).map_or_else(
                        |(a, b)| {
                            if a == b {
                                Ok(Some(a))
                            } else {
                                Err(IllFormed::TypeMismatch(a.to_owned(), b.to_owned()))
                            }
                        },
                        Ok,
                    )
                })?);
                shit.map_or_else(
                    |(a, b)| {
                        if a == b {
                            Ok(Some(a))
                        } else {
                            Err(IllFormed::TypeMismatch(a.to_owned(), b.to_owned()))
                        }
                    },
                    Ok,
                )
            })
    }

    /// Change nothing about the semantics but sort the internal vector of states.
    #[inline]
    #[allow(clippy::panic)] // <-- TODO
    #[allow(clippy::missing_panics_doc)]
    pub fn sort(&mut self) {
        // Associate each original index with a concrete state instead of just an index,
        // since we're going to be swapping the indices around.
        let index_map: BTreeMap<usize, State<_, _>> =
            self.states.iter().cloned().enumerate().collect();
        self.states.sort_unstable();
        self.states.dedup(); // <-- Cool that we can do this!
        self.initial = self
            .initial
            .clone()
            .map_indices(|i| unwrap!(self.states.binary_search(unwrap!(index_map.get(&i)))));
        // Can't do this in-place since the entire state array is required as an argument.
        self.states = self
            .states
            .iter()
            .map(|s| s.reindex(&self.states, &index_map))
            .collect();
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_curry<I: Input, C: Ctrl<I>>(value: Curry<I, C>, ordering: &[C]) -> Curry<I, usize> {
    match value {
        Curry::Wildcard(etc) => Curry::Wildcard(fix_indices_transition(etc, ordering)),
        Curry::Scrutinize { filter, fallback } => Curry::Scrutinize {
            filter: fix_indices_range_map(filter, ordering),
            fallback: fallback.map(|f| fix_indices_transition(f, ordering)),
        },
    }
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_range_map<I: Input, C: Ctrl<I>>(
    value: RangeMap<I, C>,
    ordering: &[C],
) -> RangeMap<I, usize> {
    RangeMap(
        value
            .0
            .into_iter()
            .map(|(k, v)| (k, fix_indices_transition(v, ordering)))
            .collect(),
    )
}

/// Use an ordering on subsets to translate each subset into a specific state.
#[inline]
#[allow(clippy::type_complexity)]
fn fix_indices_transition<I: Input, C: Ctrl<I>>(
    value: Transition<I, C>,
    ordering: &[C],
) -> Transition<I, usize> {
    match value {
        Transition::Lateral { dst, update } => Transition::Lateral {
            dst: unwrap!(ordering.binary_search(&dst)),
            update,
        },
        Transition::Call {
            region,
            detour,
            dst,
            combine,
        } => Transition::Call {
            region,
            detour: unwrap!(ordering.binary_search(&detour)),
            dst: unwrap!(ordering.binary_search(&dst)),
            combine,
        },
        Transition::Return { region } => Transition::Return { region },
    }
}

impl<I: Input> Graph<I, usize> {
    /// Write this parser as a Rust source file.
    /// # Errors
    /// If file creation or formatting fails.
    #[inline]
    pub fn to_file<P: AsRef<OsStr> + AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<io::Result<()>, IllFormed<I, usize>> {
        self.to_src().map(|src| {
            fs::write(&path, src)?;
            Command::new("rustfmt").arg(path).output().map(|_| {})
        })
    }
}

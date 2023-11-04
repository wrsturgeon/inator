/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{
    try_merge, Check, Ctrl, CurryInput, CurryStack, IllFormed, Input, InputError, Merge,
    ParseError, RangeMap, Stack, State, ToSrc, Transition,
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
    /// Map string tags to the state or states they represent.
    pub tags: BTreeMap<String, usize>,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for Graph<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            initial: self.initial.clone(),
            tags: self.tags.clone(),
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
            let state = match r {
                Ok(i) => {
                    if let Some(state) = self.states.get(i) {
                        state
                    } else {
                        return Err(IllFormed::OutOfBounds(i));
                    }
                }
                Err(tag) => self
                    .tags
                    .get(tag)
                    .map(|&i| get!(self.states, i))
                    .ok_or(IllFormed::TagDNE(tag.to_owned()))?,
            };
            if let Some(t) = state.input_type()? {
                if t != "()" {
                    return Err(IllFormed::InitialNotUnit(t));
                }
            }
            for curry in state.transitions.values() {
                for transition in curry.values() {
                    if transition.update.input_t != "()" {
                        return Err(IllFormed::InitialNotUnit(transition.update.input_t.clone()));
                    }
                }
            }
        }
        drop(self.output_type()?);
        for (i, state) in self.states.iter().enumerate() {
            if get!(self.states, ..i).contains(state) {
                return Err(IllFormed::DuplicateState(Box::new(state.clone())));
            }
        }
        for &index in self.tags.values() {
            if index >= n_states {
                return Err(IllFormed::OutOfBounds(index));
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
    ) -> Result<String, ParseError<I, S, C>> {
        use crate::Run;
        let mut run = input.run(self);
        for r in &mut run {
            drop(r?);
        }
        for r in run.ctrl.view() {
            if match r {
                Ok(i) => get!(self.states, i),
                Err(tag) => get!(
                    self.states,
                    *self
                        .tags
                        .get(tag)
                        .ok_or(ParseError::BadParser(IllFormed::TagDNE(tag.to_owned())))?
                ),
            }
            .non_accepting
            .is_empty()
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
        for &i in self.tags.values() {
            self.explore(&mut subsets_as_states, &C::from_usize(i))?;
        }

        // Fix an ordering on those subsets
        let mut ordering: Vec<C> = subsets_as_states.keys().cloned().collect();
        ordering.sort_unstable();
        ordering.dedup();

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
                        transitions: fix_indices_curry_stack(transitions, &ordering),
                        non_accepting,
                    }
                })
                .collect(),
            tags: self
                .tags
                .iter()
                .map(|(k, &v)| {
                    (
                        k.clone(),
                        unwrap!(ordering.binary_search(&C::from_usize(v))),
                        // unwrap!(ordering.binary_search_by(|c| {
                        //     // Make sure there's only one element and name it `r`
                        //     let r = {
                        //         let mut view = c.view();
                        //         let Some(r) = view.next() else {
                        //             return Ordering::Less;
                        //         };
                        //         if view.next().is_some() {
                        //             return Ordering::Greater;
                        //         }
                        //         r
                        //     };
                        //     match r {
                        //         Ok(ref i) => i,
                        //         Err(tag) => unwrap!(self.tags.get(tag)),
                        //     }
                        //     .cmp(&v)
                        // })),
                        // ordering
                        //     .iter()
                        //     .position(|c| {
                        //         // Make sure there's only one element and name it `r`
                        //         let r = {
                        //             let mut view = c.view();
                        //             let Some(r) = view.next() else {
                        //                 return false;
                        //             };
                        //             if view.next().is_some() {
                        //                 return false;
                        //             }
                        //             r
                        //         };
                        //         match r {
                        //             Ok(i) => i == v,
                        //             Err(tag) => self.tags.get(tag) == Some(&v),
                        //         }
                        //     })
                        //     .unwrap(),
                    )
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
        subsets_as_states: &mut BTreeMap<C, State<I, S, C>>,
        subset: &C,
    ) -> Result<(), IllFormed<I, S, C>> {
        // Check if we've seen this subset already
        let btree_map::Entry::Vacant(entry) = subsets_as_states.entry(subset.clone()) else {
            return Ok(());
        };

        // Merge this subset of states into one (most of the heavy lifting)
        let result_iterator = subset.view().map(|r| match r {
            Ok(i) => Ok(get!(self.states, i).clone()),
            Err(s) => self.tags.get(s).map_or_else(
                || Err(IllFormed::TagDNE(s.to_owned())),
                |&i| Ok(get!(self.states, i).clone()),
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

    /// Compute the input type of any successful run.
    /// # Errors
    /// If multiple accepting states attempt to return different types.
    #[inline]
    pub fn input_type(&self) -> Result<Option<String>, IllFormed<I, S, C>> {
        self.initial
            .view()
            .map(|r| {
                r.map_or_else(
                    |tag| get!(self.states, *unwrap!(self.tags.get(tag))),
                    |i| get!(self.states, i),
                )
            })
            .try_fold(None, |acc, state| {
                let shit = acc.merge(state.transitions.values().try_fold(None, |acc, curry| {
                    acc.merge({
                        curry.values().try_fold(None, |acc, t| {
                            acc.merge(Some(t.update.input_t.clone())).map_or_else(
                                |(a, b)| {
                                    if a == b {
                                        Ok(Some(a))
                                    } else {
                                        Err(IllFormed::TypeMismatch(a, b))
                                    }
                                },
                                Ok,
                            )
                        })?
                    })
                    .map_or_else(
                        |(a, b)| {
                            if a == b {
                                Ok(Some(a))
                            } else {
                                Err(IllFormed::TypeMismatch(a, b))
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
                            Err(IllFormed::TypeMismatch(a, b))
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
        let index_map: BTreeMap<usize, State<_, _, _>> =
            self.states.iter().cloned().enumerate().collect();
        self.states.sort_unstable();
        self.states.dedup(); // <-- Cool that we can do this!
        self.initial = self
            .initial
            .clone()
            .map_indices(|i| unwrap!(self.states.binary_search(unwrap!(index_map.get(&i)))));
        for i in self.tags.values_mut() {
            // *i = unwrap!(self.states.binary_search(unwrap!(index_map.get(i)))); // <-- TODO: reinstate
            *i = unwrap!(self.states.binary_search(
                index_map
                    .get(i)
                    .unwrap_or_else(|| panic!("Couldn't find {i:?} in {:?}", index_map.to_src()))
            ));
        }
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

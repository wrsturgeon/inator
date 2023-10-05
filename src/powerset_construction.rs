/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The powerset construction algorithm for constructing an equivalent DFA from an arbitrary NFA.
//! Also known as the subset construction algorithm.

use crate::{dfa, nfa, Compiled as Dfa, Parser as Nfa};
use core::fmt::Debug;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

/// Subset of states (by their index).
type Subset = BTreeSet<usize>;

/// From a single state, all tokens and the transitions each would induce.
type Transitions<I> = BTreeMap<I, Transition<I>>;

/// Function (or none) to call on an edge.
type Call = Option<&'static str>;

/// A single edge triggered by a token.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Transition<I> {
    /// Set of destination states.
    dsts: Subset,
    /// Function (or none) to call on this edge.
    call: Call,
    /// Minimal reproducible input string to reach this transition.
    breadcrumbs: Vec<I>,
}

/// A collection of outgoing edges and a boolean to mark accepting states.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct SubsetState<I> {
    /// Transitions from this subset of states to other subsets on certain tokens.
    transitions: Transitions<I>,
    /// Whether we should accept a string that ends in this state.
    accepting: bool,
}

/// Map each _subset_ of NFA states to a future DFA state.
type SubsetsAsStates<I> = BTreeMap<Subset, SubsetState<I>>;

impl<I: Clone + Ord + Debug> Nfa<I> {
    /// Powerset construction algorithm mapping subsets of states to DFA nodes.
    #[inline]
    pub(crate) fn subsets(self) -> Dfa<I> {
        // Map which _subsets_ of states transition to which _subsets_ of states
        let mut subsets_as_states = SubsetsAsStates::new();
        let initial_state = traverse(
            &self,
            self.initial.iter().copied().collect(),
            &mut subsets_as_states,
            vec![],
        );

        // Fix an ordering on subsets so each can be a DFA state
        let mut ordered: Vec<_> = subsets_as_states.keys().collect();
        ordered.sort_unstable();

        // Check that binary_search works
        #[cfg(test)]
        {
            for (i, subset) in ordered.iter().enumerate() {
                assert_eq!(ordered.binary_search(subset), Ok(i));
            }
        }

        // Construct the vector of subset-mapped states
        let states = ordered
            .iter()
            .map(|&subset| {
                let &SubsetState {
                    ref transitions,
                    accepting,
                } = unwrap!(subsets_as_states.get(subset));
                dfa::State {
                    transitions: transitions
                        .iter()
                        .map(|(token, &Transition { ref dsts, call, .. })| {
                            (
                                token.clone(),
                                dfa::Transition {
                                    dst: unwrap!(ordered.binary_search(&dsts)),
                                    call,
                                },
                            )
                        })
                        .collect(),
                    accepting,
                }
            })
            .collect();

        // Wrap it in a DFA
        Dfa {
            states,
            initial: unwrap!(ordered.binary_search(&&initial_state)),
        }
    }
}

/// Map which _subsets_ of states transition to which _subsets_ of states.
/// Return the expansion of the original `queue` argument after taking all epsilon transitions.
#[inline]
#[allow(unused_mut)] // <-- FIXME
fn traverse<I: Clone + Ord + Debug>(
    nfa: &Nfa<I>,
    queue: Vec<usize>,
    subsets_as_states: &mut SubsetsAsStates<I>,
    mut so_far: Vec<I>,
) -> Subset // <-- Return the set of states after taking epsilon transitions
{
    // Take all epsilon transitions immediately
    let post_epsilon = nfa.take_all_epsilon_transitions(queue);

    // Check if we've already seen this subset
    let tmp = match subsets_as_states.entry(post_epsilon.clone()) {
        Entry::Occupied(_) => return post_epsilon,
        Entry::Vacant(empty) => empty,
    };

    // Get all _states_ from indices
    let subset = post_epsilon.iter().map(|&i| get!(nfa.states, i));

    // For now, so we can't get stuck in a cycle, cache an empty map
    let _ = tmp.insert(SubsetState {
        transitions: BTreeMap::new(),
        accepting: subset
            .clone()
            .any(|&nfa::State { accepting, .. }| accepting),
    });

    // Calculate all non-epsilon transitions out of this *subset* of states,
    // converted into transitions out of a *single* DFA state.
    let mut transitions = Transitions::<I>::new();
    // For each state we're currently inhabiting,...
    for state in subset {
        // ... for each non-epsilon transition out of that state...
        for (
            token,
            &nfa::Transition {
                dsts: ref map,
                call: fn_name,
            },
        ) in &state.non_epsilon
        {
            // ... check if we already have any other transitions on that token.
            match transitions.entry(token.clone()) {
                // If we don't, ...
                Entry::Vacant(entry) => {
                    // ... construct the input string that would reach here, ...
                    let mut breadcrumbs = so_far.clone();
                    breadcrumbs.push(token.clone());
                    // ... and insert a new transition.
                    let _ = entry.insert(Transition {
                        dsts: map.clone(),
                        call: fn_name,
                        breadcrumbs,
                    });
                }
                // If we already have a transition on this token, ...
                Entry::Occupied(entry) => {
                    // ... check what we have so far.
                    #[allow(unused_variables)] // <-- FIXME
                    let &mut Transition {
                        ref mut dsts,
                        ref mut call,
                        ref mut breadcrumbs,
                    } = entry.into_mut();
                    // If the function we want to call happens to be the same, ...
                    #[allow(clippy::todo)] // <- FIXME
                    if fn_name == *call {
                        // ... just take the set-union of this state's destinations and the existing ones.
                        dsts.extend(map.iter().copied());
                    }
                    // If we're trying to call two different functions on the same input, ...
                    else {
                        // ... make sure the functions don't use this exact input token, ...
                        todo!();
                        // assert!(!fn_name.uses_token(), "MESSAGE TODO");
                        // assert!(!call.uses_token(), "MESSAGE TODO");
                        // postpone();
                    }
                }
            }
        }
    }

    // Now, follow epsilon transitions AND recurse
    for &mut Transition {
        ref mut dsts,
        ref mut breadcrumbs,
        ..
    } in transitions.values_mut()
    {
        *dsts = traverse(
            nfa,
            dsts.iter().copied().collect(),
            subsets_as_states,
            breadcrumbs.clone(),
        );
    }

    // Rewrite the empty map we wrote earlier with the actual transitions
    unwrap!(subsets_as_states.get_mut(&post_epsilon)).transitions = transitions;

    post_epsilon
}

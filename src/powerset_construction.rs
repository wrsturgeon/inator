/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The powerset construction algorithm for constructing an equivalent DFA from an arbitrary NFA.
//! Also known as the subset construction algorithm.

use crate::{call::Call, dfa, nfa, Compiled as Dfa, Parser as Nfa};
use core::fmt::Debug;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

/// Subset of states (by their index).
type Subset = BTreeSet<usize>;

/// From a single state, all tokens and the transitions each would induce.
type Transitions<I> = BTreeMap<I, Transition<I>>;

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

/// Postpone a call from an original call site to some set of later states.
#[allow(dead_code)] // <-- FIXME
struct Postpone {
    /// Left call (not really "left" in any meaningful sense, but the first one).
    pl: Call,
    /// Right call (not really "right" in any meaningful sense, but the second one).
    pr: Call,
    /// Subset of states that will now be responsible for the call.
    to: Subset,
}

impl<I: Clone + Ord + Debug> Nfa<I> {
    /// Powerset construction algorithm mapping subsets of states to DFA nodes.
    #[inline]
    pub(crate) fn subsets(mut self) -> Dfa<I> {
        // Map which _subsets_ of states transition to which _subsets_ of states
        let (subsets_as_states, initial_state) = self.postpone_ambiguities();

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
                        .map(
                            |(
                                token,
                                &Transition {
                                    ref dsts, ref call, ..
                                },
                            )| {
                                (
                                    token.clone(),
                                    dfa::Transition {
                                        dst: unwrap!(ordered.binary_search(&dsts)),
                                        call: call.clone(),
                                    },
                                )
                            },
                        )
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

    /// Postpone calls that would be ambiguous until they're no longer ambiguous (if possible).
    #[inline]
    #[allow(clippy::todo)] // <-- FIXME
    fn postpone_ambiguities(&mut self) -> (SubsetsAsStates<I>, Subset) {
        loop {
            let mut subsets_as_states = SubsetsAsStates::new();
            match self.traverse(
                self.initial.iter().copied().collect(),
                &mut subsets_as_states,
                vec![],
            ) {
                Ok(subset) => return (subsets_as_states, subset),
                Err(_postpone) => todo!(),
            }
        }
    }

    /// Map which _subsets_ of states transition to which _subsets_ of states.
    /// Return the expansion of the original `queue` argument after taking all epsilon transitions.
    #[inline]
    #[allow(clippy::todo)] // <-- FIXME
    #[allow(clippy::too_many_lines, clippy::unwrap_in_result)]
    fn traverse(
        &self,
        queue: Vec<usize>,
        subsets_as_states: &mut SubsetsAsStates<I>,
        mut so_far: Vec<I>,
    ) -> Result<Subset, Postpone> // <-- Return the set of states after taking epsilon transitions
    {
        // Take all epsilon transitions immediately
        let post_epsilon = self.take_all_epsilon_transitions(queue);

        // Check if we've already seen this subset
        let tmp = match subsets_as_states.entry(post_epsilon.clone()) {
            Entry::Occupied(_) => return Ok(post_epsilon),
            Entry::Vacant(empty) => empty,
        };

        // Get all _states_ from indices
        let subset = post_epsilon.iter().map(|&i| get!(self.states, i));

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
                    call: ref new_call,
                },
            ) in &state.non_epsilon
            {
                // ... compute the input string that reached this state ...
                let mut input_string = so_far.clone();
                input_string.push(token.clone());
                // ... check if we already have any other transitions on that token.
                match transitions.entry(token.clone()) {
                    // If we don't, ...
                    Entry::Vacant(entry) => {
                        // ... insert a new transition.
                        let _ = entry.insert(Transition {
                            dsts: map.clone(),
                            call: new_call.clone(),
                            breadcrumbs: input_string,
                        });
                    }
                    // If we already have a transition on this token, ...
                    Entry::Occupied(entry) => {
                        // ... check what we have so far, ...
                        let &mut Transition {
                            ref mut dsts,
                            call: ref mut existing_call,
                            ref mut breadcrumbs,
                        } = entry.into_mut();
                        // ... and if we have a shorter input string that reached here, replace it.
                        if input_string.len() < breadcrumbs.len() {
                            *breadcrumbs = input_string;
                        }
                        // Next, check for trying to call two different functions on the same input.
                        #[allow(clippy::panic)]
                        match new_call
                            .clone()
                            .compat(existing_call.clone())
                            .unwrap_or_else(|| {
                                panic!(
                                    "Parsing ambiguity after [{}] on token {token:?}: \
                                    can't immediately decide between {} and {}, \
                                    but a function reads this exact token at runtime. \
                                    (Note: if you rewrite the function without \
                                    reading the value of the token it parsed, \
                                    we can automatically postpone the decision \
                                    until some later token is different.)",
                                    so_far.pop().map_or_else(String::new, |last| {
                                        so_far.iter().fold(String::new(), |acc, i| {
                                            acc + &format!("{i:?} -> ")
                                        }) + &format!("{last:?}")
                                    }),
                                    new_call.verbal(),
                                    existing_call.verbal(),
                                )
                            }) {
                            // If it's the same function, ...
                            Ok(common) => {
                                // ... then just add some states to the subset.
                                dsts.extend(map.iter().copied());
                                *existing_call = common;
                            }
                            // If not identical but still compatible, ...
                            Err((stash, pl, pr)) => {
                                // ... update this state not to call (but to save the value if necessary), ...
                                *existing_call = if stash { Call::Stash } else { Call::Pass };
                                // ... and push the call back to the next states.
                                return Err(Postpone {
                                    pl,
                                    pr,
                                    to: self.take_all_epsilon_transitions(
                                        dsts.iter().copied().collect(),
                                    ),
                                });
                            }
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
            *dsts = self.traverse(
                dsts.iter().copied().collect(),
                subsets_as_states,
                breadcrumbs.clone(),
            )?;
        }

        // Rewrite the empty map we wrote earlier with the actual transitions
        unwrap!(subsets_as_states.get_mut(&post_epsilon)).transitions = transitions;

        Ok(post_epsilon)
    }
}

impl Call {
    // fn take_responsibility_for(&mut self, postpone: Postpone) {}
}

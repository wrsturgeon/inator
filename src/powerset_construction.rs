/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The powerset construction algorithm for constructing an equivalent deterministic automaton from an arbitrary nondeterministic automaton.
//! Also known as the subset construction algorithm.

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use crate::{Compiled as D, Parser as N};

/// Type for transitions from _subsets_ of states to _subsets_ of states.
type SubsetStates<I, S> = BTreeMap<
    BTreeSet<usize>, // <-- Subset of NFA states (will become one DFA state)
    (
        BTreeMap<(I, S), (BTreeSet<usize>, S, Option<&'static str>, Vec<I>)>, // <-- Transitions
        bool, // <-- Accepting or not
    ),
>;

impl<I: Clone + Ord + core::fmt::Debug, S: Clone + Ord> N<I, S> {
    /// Powerset construction algorithm mapping subsets of states to DFA nodes.
    #[inline]
    pub(crate) fn subsets(self) -> D<I, S> {
        // Map which _subsets_ of states transition to which _subsets_ of states
        let mut subset_states = SubsetStates::new();
        let initial_state = traverse(
            &self,
            self.initial.iter().map(|&(i, _)| i).collect(),
            &mut subset_states,
            vec![],
        );

        // Fix an ordering on subsets so each can be a DFA state
        let mut ordered: Vec<_> = subset_states.keys().collect();
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
                let &(ref states, accepting) = unwrap!(subset_states.get(subset));
                crate::deterministic::State {
                    transitions: states
                        .iter()
                        .map(|(token, &(ref set, push, fn_name, _))| {
                            (
                                token.clone(),
                                (unwrap!(ordered.binary_search(&set)), fn_name),
                            )
                        })
                        .collect(),
                    accepting,
                }
            })
            .collect();

        // Wrap it in a DFA
        D {
            states,
            initial: unwrap!(ordered.binary_search(&&initial_state)),
        }
    }
}

/// Map which _subsets_ of states transition to which _subsets_ of states.
/// Return the expansion of the original `queue` argument after taking all epsilon transitions.
#[inline]
fn traverse<I: Clone + Ord + core::fmt::Debug, S: Clone + Ord>(
    nondeterministic: &N<I, S>,
    queue: Vec<usize>,
    subset_states: &mut SubsetStates<I, S>,
    mut so_far: Vec<I>,
) -> BTreeSet<usize> // <-- Return the set of states after taking epsilon transitions
{
    // Take all epsilon transitions immediately
    let post_epsilon = nondeterministic.take_all_epsilon_transitions(queue);

    // Check if we've already seen this subset
    let tmp = match subset_states.entry(post_epsilon.clone()) {
        std::collections::btree_map::Entry::Occupied(_) => return post_epsilon,
        std::collections::btree_map::Entry::Vacant(empty) => empty,
    };

    // Get all _states_ from indices
    let subset = post_epsilon
        .iter()
        .map(|&i| get!(nondeterministic.states, i));

    // For now, so we can't get stuck in a cycle, cache an empty map
    let _ = tmp.insert((BTreeMap::new(), subset.clone().any(|state| state.accepting)));

    // Calculate the next superposition of states WITHOUT EPSILON TRANSITIONS YET
    let mut transitions =
        BTreeMap::<(I, S), (BTreeSet<usize>, S, Option<&'static str>, Vec<I>)>::new();
    for state in subset {
        for (token, &(ref map, fn_name)) in &state.non_epsilon {
            match transitions.entry(token.clone()) {
                Entry::Vacant(entry) => {
                    let _ = entry.insert((
                        map.clone(),
                        todo!(),
                        fn_name,
                        {
                            let mut app = so_far.clone();
                            app.push(token.clone());
                            app
                        },
                    ));
                }
                Entry::Occupied(entry) => {
                    let &mut (ref mut set, _, ref mut existing_fn_name, ref mut other_input) =
                        entry.into_mut();
                    assert_eq!(
                        fn_name,
                        *existing_fn_name,
                        "Parsing ambiguity after [{}] / [{}]: {token:?} can't decide between {} and {}",
                        {
                            // let mut v = nondeterministic.backtrack(queue.iter().copied().collect());
                            other_input.pop().map_or_else(
                                String::new,
                                |last| {
                                    other_input
                                        .iter()
                                        .fold(String::new(), |acc, i| acc + &format!("{i:?} -> "))
                                        + &format!("{last:?}")
                                }
                            )
                        },
                        {
                            // let mut v = nondeterministic.backtrack(queue.iter().copied().collect());
                            so_far.pop().map_or_else(
                                String::new,
                                |last| {
                                    so_far
                                        .into_iter()
                                        .fold(String::new(), |acc, i| acc + &format!("{i:?} -> "))
                                        + &format!("{last:?}")
                                }
                            )
                        },
                        fn_name.map_or_else(|| "ignoring".to_owned(), |f| format!("calling {f}")),
                        existing_fn_name
                            .map_or_else(|| "ignoring".to_owned(), |f| format!("calling {f}")),
                    );
                    set.extend(map.iter().cloned());
                }
            }
        }
    }

    // Now, follow epsilon transitions AND recurse
    for &mut (ref mut dst, _, _, ref mut app) in transitions.values_mut() {
        *dst = traverse(
            nondeterministic,
            dst.iter().cloned().collect(),
            subset_states,
            app.clone(),
        );
    }

    // Rewrite the empty map we wrote earlier with the actual transitions
    unwrap!(subset_states.get_mut(&post_epsilon)).0 = transitions;

    post_epsilon
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The powerset construction algorithm for constructing an equivalent DFA from an arbitrary NFA.
//! Also known as the subset construction algorithm.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Dfa, Nfa};

/// Type for transitions from _subsets_ of states to _subsets_ of states.
type SubsetStates<I> = BTreeMap<BTreeSet<usize>, (BTreeMap<I, BTreeSet<usize>>, bool)>;

#[allow(clippy::fallible_impl_from)]
impl<I: Clone + Ord> From<Nfa<I>> for Dfa<I> {
    #[inline]
    fn from(value: Nfa<I>) -> Self {
        // Check if we have any states at all
        if value.is_empty() {
            return Dfa { states: vec![] };
        }

        // Map which _subsets_ of states transition to which _subsets_ of states
        let mut subset_states = BTreeMap::new();
        let initial_state = traverse(&value, core::iter::once(0).collect(), &mut subset_states);

        // Fix an ordering on those subsets so each can be a DFA state
        let mut ordered: Vec<_> = subset_states.keys().collect();

        // TODO: sort `ordered`, use `binary_search` below, and add an `initial` member to DFAs & NFAs

        // Move the initial state to the first index
        {
            let initial_state_index =
                unwrap!(ordered.iter().position(|&tree| tree == &initial_state));
            ordered.swap(0, initial_state_index);
        }

        // Construct the vector of subset-mapped states
        let states = ordered
            .iter()
            .map(|subset| {
                let &(ref tree, accepting) = unwrap!(subset_states.get(subset));
                crate::dfa::State {
                    transitions: tree
                        .iter()
                        .map(|(k, v)| (k.clone(), unwrap!(ordered.iter().position(|&t| t == v))))
                        .collect::<BTreeMap<I, usize>>(),
                    accepting,
                }
            })
            .collect();

        // Wrap it in a DFA
        Dfa { states }
    }
}

/// Map which _subsets_ of states transition to which _subsets_ of states.
/// Return the expansion of the original `queue` argument after taking all epsilon transitions.
#[inline]
fn traverse<I: Clone + Ord>(
    nfa: &Nfa<I>,
    queue: BTreeSet<usize>,
    subset_states: &mut SubsetStates<I>,
) -> BTreeSet<usize> {
    // Take all epsilon transitions immediately
    let superposition = nfa.take_all_epsilon_transitions(queue);

    // Check if we've already seen this state
    let entry = match subset_states.entry(superposition.clone()) {
        std::collections::btree_map::Entry::Occupied(_) => return superposition,
        std::collections::btree_map::Entry::Vacant(empty) => empty,
    };

    // Get all _states_ from indices
    let states = superposition.iter().map(|&i| unwrap!(nfa.get(i)));

    // For now, so we can't get stuck in a cycle, cache an empty map:
    let _ = entry.insert((
        BTreeMap::new(),
        states.clone().any(crate::nfa::State::is_accepting),
    ));

    // Calculate the next superposition of states WITHOUT EPSILON TRANSITIONS YET
    let mut next_superposition = BTreeMap::<I, BTreeSet<usize>>::new();
    for state in states {
        for (k, v) in state.non_epsilon_transitions() {
            next_superposition.entry(k.clone()).or_default().extend(v);
        }
    }

    // Now, follow epsilon transitions AND recurse
    for v in next_superposition.values_mut() {
        *v = traverse(nfa, v.clone(), subset_states);
    }

    // TODO:
    // could be a stack explosion above on a large NFA;
    // think about how to make this iterative instead

    // Insert the new values!
    unwrap!(subset_states.get_mut(&superposition)).0 = next_superposition;

    superposition
}

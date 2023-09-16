//! The powerset construction algorithm for constructing an equivalent DFA from an arbitrary NFA.
//! Also known as the subset construction algorithm.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Dfa, Nfa};

impl<I: Clone + Ord> From<Nfa<I>> for Dfa<I> {
    #[inline]
    fn from(value: Nfa<I>) -> Self {
        // Check if we have any states at all
        if value.is_empty() {
            return Dfa::new(vec![]);
        }

        // Map which _subsets_ of states transition to which _subsets_ of states
        let mut subset_states = BTreeMap::new();
        traverse(&value, core::iter::once(0).collect(), &mut subset_states);

        // Fix an ordering on those subsets so each can be a DFA state
        let mut ordered: Vec<_> = subset_states.keys().collect();
        ordered.sort_unstable();

        Dfa::new(
            ordered
                .iter()
                .enumerate()
                .map(|(i, subset)| {
                    debug_assert_eq!(ordered.binary_search(subset), Ok(i));
                    unwrap!(subset_states.get(subset))
                        .iter()
                        .map(|(k, v)| (k.clone(), unwrap!(ordered.binary_search(&v))))
                        .collect::<BTreeMap<I, usize>>()
                        .into()
                })
                .collect(),
        )
    }
}

/// Map which _subsets_ of states transition to which _subsets_ of states.
/// Return the expansion of the original `queue` argument after taking all epsilon transitions.
#[inline]
fn traverse<I: Clone + Ord>(
    nfa: &Nfa<I>,
    queue: BTreeSet<usize>,
    subset_states: &mut BTreeMap<BTreeSet<usize>, BTreeMap<I, BTreeSet<usize>>>,
) -> BTreeSet<usize> {
    // Take all epsilon transitions immediately
    let superposition = nfa.take_all_epsilon_transitions(queue);

    // Check if we've already seen this state
    if subset_states.contains_key(&superposition) {
        return superposition;
    }

    // For now, so we can't get stuck in a cycle, cache an empty map:
    subset_states.insert(superposition.clone(), BTreeMap::new());

    // Calculate the next superposition of states WITHOUT EPSILON TRANSITIONS YET
    let mut next_superposition = BTreeMap::<I, BTreeSet<usize>>::new();
    for &state in &superposition {
        for (k, v) in unwrap!(nfa.get(state)).non_epsilon_transitions() {
            next_superposition
                .entry(k.clone())
                .or_insert(BTreeSet::new())
                .extend(v);
        }
    }

    // Now, follow epsilon transitions AND recurse
    for v in next_superposition.values_mut() {
        *v = traverse(nfa, v.clone(), subset_states);
    }

    // Insert the new values!
    subset_states.insert(superposition.clone(), next_superposition);

    superposition
}

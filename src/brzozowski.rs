/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Brzozowski's algorithm for minimizing automata.

use crate::{nfa, Compiled as Dfa, Parser as Nfa};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

impl<I: Clone + Ord> Nfa<I> {
    /// Reverse all transitions and swap initial with accepting states.
    /// # Panics
    /// TODO
    #[inline]
    #[must_use]
    pub fn reverse(&self) -> Self {
        let mut states = core::iter::repeat(nfa::State {
            epsilon: BTreeSet::new(),
            non_epsilon: BTreeMap::new(),
            accepting: false,
        })
        .take(self.states.len())
        .collect::<Vec<_>>();
        let mut initial = BTreeSet::new();
        for (src, state) in self.states.iter().enumerate() {
            for &dst in &state.epsilon {
                let _ = get_mut!(states, dst).epsilon.insert(src);
            }
            for (token, &(ref set, fn_name)) in &state.non_epsilon {
                for &dst in set {
                    match get_mut!(states, dst).non_epsilon.entry(token.clone()) {
                        Entry::Vacant(entry) => {
                            let _ = entry.insert((core::iter::once(src).collect(), fn_name));
                        }
                        Entry::Occupied(entry) => {
                            let &mut (ref mut mut_set, existing_fn_name) = entry.into_mut();
                            assert_eq!(fn_name, existing_fn_name, "MESSAGE TODO");
                            let _ = mut_set.insert(src);
                        }
                    }
                }
            }
            if state.accepting {
                let _ = initial.insert(src);
            }
        }
        for &index in &self.initial {
            get_mut!(states, index).accepting = true;
        }
        Self { states, initial }
    }

    /// Brzozowski's algorithm for minimizing automata.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_assert_message)]
    pub fn compile(&self) -> Dfa<I> {
        let rev = self.reverse();
        let halfway = rev.subsets();
        let nfa = halfway.generalize();
        let revrev = nfa.reverse();
        revrev.subsets()
    }
}

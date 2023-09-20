/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Brzozowski's algorithm for minimizing automata.

use crate::{decision::nfa, Dfa, Nfa};
use std::collections::{BTreeMap, BTreeSet};

impl<I: Clone + Ord> Nfa<I> {
    /// Reverrse all transitions and swap initial with accepting states.
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
            for (k, v) in &state.non_epsilon {
                for &dst in v {
                    let _ = get_mut!(states, dst)
                        .non_epsilon
                        .entry(k.clone())
                        .or_default()
                        .insert(src);
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

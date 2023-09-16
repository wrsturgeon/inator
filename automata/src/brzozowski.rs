/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Brzozowski's algorithm for minimizing automata.

use crate::{nfa, Dfa, Nfa};
use std::collections::BTreeSet;

impl<I: Clone + Ord> From<Dfa<I>> for Nfa<I> {
    #[inline]
    fn from(value: Dfa<I>) -> Self {
        Nfa {
            states: value
                .states
                .into_iter()
                .map(|state| nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: state
                        .transitions
                        .into_iter()
                        .map(|(k, v)| (k, core::iter::once(v).collect()))
                        .collect(),
                    accepting: state.accepting,
                })
                .collect(),
            initial: core::iter::once(value.initial).collect(),
        }
    }
}

impl<I: Clone + Ord> Nfa<I> {}

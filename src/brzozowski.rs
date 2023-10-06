/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Brzozowski's algorithm for minimizing automata.

use crate::{nfa, Compiled as Dfa, Parser as Nfa};
use core::{
    fmt::Debug,
    iter::{once, repeat},
};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

impl<I: Clone + Ord> Nfa<I> {
    /// Reverse all transitions and swap initial with accepting states.
    /// # Panics
    /// TODO
    #[inline]
    #[must_use]
    pub fn reverse(&self) -> Self {
        let mut states = repeat(nfa::State {
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
            for (
                token,
                &nfa::Transition {
                    dsts: ref set,
                    ref call,
                },
            ) in &state.non_epsilon
            {
                for &dst in set {
                    match get_mut!(states, dst).non_epsilon.entry(token.clone()) {
                        Entry::Vacant(entry) => {
                            let _ = entry.insert(nfa::Transition {
                                dsts: once(src).collect(),
                                call: call.clone(),
                            });
                        }
                        Entry::Occupied(entry) => {
                            let &mut nfa::Transition {
                                dsts: ref mut mut_set,
                                call: ref existing_fn_name,
                            } = entry.into_mut();
                            assert_eq!(*call, *existing_fn_name, "MESSAGE TODO");
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
    pub fn compile(&self) -> Dfa<I>
    where
        I: Debug,
    {
        /// Print if and only if we're debugging or testing.
        #[cfg(any(test, debug_assertions))]
        macro_rules! dbg_print {
            ($($arg:tt)*) => {
                println!($($arg)*)
            };
        }
        /// Print if and only if we're debugging or testing.
        #[cfg(not(any(test, debug_assertions)))]
        macro_rules! dbg_print {
            ($($arg:tt)*) => {};
        }
        dbg_print!("Brzozowski: powerset construction (1st time)...");
        let dfa = self.subsets();
        dbg_print!("Brzozowski: reversing (1st time)...");
        let rev = dfa.generalize().reverse();
        dbg_print!("Brzozowski: powerset construction (2nd time)...");
        let halfway = rev.subsets();
        dbg_print!("Brzozowski: reversing (2nd time)...");
        let revrev = halfway.generalize().reverse();
        dbg_print!("Brzozowski: powerset construction (3rd time)...");
        revrev.subsets()
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Brzozowski's algorithm for minimizing automata.

use crate::{call::Call, nfa, Compiled as Dfa, Expression, Parser as Nfa};
use core::{
    fmt::Debug,
    iter::{once, repeat},
};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

impl<I: Clone + Expression + Ord> Nfa<I> {
    /// Reverse all transitions and swap initial with accepting states.
    /// # Panics
    /// TODO
    #[inline]
    #[must_use]
    pub fn reverse(&self) -> Self {
        let mut states = repeat(nfa::State {
            epsilon: BTreeSet::new(),
            non_epsilon: BTreeMap::new(),
            accepting: None,
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
            if state.accepting.is_some() {
                let _ = initial.insert(src);
            }
        }
        for &index in &self.initial {
            get_mut!(states, index).accepting = Some(Call::Pass);
        }
        Self { states, initial }
    }

    /// Brzozowski's algorithm for minimizing automata.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_assert_message)]
    pub fn compile(self) -> Dfa<I>
    where
        I: Debug,
    {
        /// Print if and only if we're debugging or testing.
        #[cfg(debug_assertions)]
        macro_rules! dbg_println {
            ($($arg:tt)*) => {
                println!($($arg)*)
            };
        }
        /// Print if and only if we're debugging or testing.
        #[cfg(not(debug_assertions))]
        macro_rules! dbg_println {
            ($($arg:tt)*) => {};
        }
        dbg_println!("Brzozowski: powerset construction (1st time)...");
        let dfa = self.subsets();
        dbg_println!(" -> {dfa}");
        dbg_println!("Brzozowski: generalizing (1st time)...");
        let nfa = dfa.generalize();
        dbg_println!(" -> {nfa}");
        dbg_println!("Brzozowski: reversing (1st time)...");
        let rev = nfa.reverse();
        dbg_println!(" -> {rev}");
        dbg_println!("Brzozowski: powerset construction (2nd time)...");
        let halfway = rev.subsets();
        dbg_println!("Brzozowski: generalizing (2nd time)...");
        let halfway_nfa = halfway.generalize();
        dbg_println!(" -> {halfway_nfa}");
        dbg_println!("Brzozowski: reversing (2nd time)...");
        let revrev = halfway_nfa.reverse();
        dbg_println!(" -> {revrev}");
        dbg_println!("Brzozowski: powerset construction (3rd time)...");
        revrev.subsets()
    }
}

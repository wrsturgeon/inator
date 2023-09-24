/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Infinite iterators over inputs guaranteed to be accepted by a given automaton.

use crate::Compiled as Dfa;
use rand::RngCore;

/// Infinite iterator over inputs guaranteed to be accepted by a given automaton.
#[derive(Clone, Debug)]
pub struct Fuzzer<I: Clone + Ord> {
    /// Reversed automaton.
    dfa: Dfa<I>,
    /// Random number generator.
    rng: rand::rngs::ThreadRng,
}

/// Tried to fuzz an automaton that never accepts any input.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NeverAccepts;

impl<I: Clone + Ord> Iterator for Fuzzer<I> {
    type Item = Vec<I>;
    #[inline]
    #[allow(clippy::unwrap_in_result)]
    fn next(&mut self) -> Option<Self::Item> {
        'start_over: loop {
            let mut index = self.dfa.initial;
            let mut v = vec![];
            loop {
                let state = get!(self.dfa.states, index);
                if state.accepting && ((self.rng.next_u32() & 1) == 0) {
                    v.reverse();
                    return Some(v);
                }
                if state.transitions.is_empty() {
                    continue 'start_over;
                }
                #[allow(clippy::arithmetic_side_effects, clippy::as_conversions)]
                let key = unwrap!(state
                    .transitions
                    .keys()
                    .nth((self.rng.next_u32() as usize) % state.transitions.len()));
                v.push(key.clone());
                index = *unwrap!(state.transitions.get(key));
            }
        }
    }
}

impl<I: Clone + Ord> Fuzzer<I> {
    /// Wrap this (ALREADY REVERSED) automaton in fuzzing capabilities.
    /// # Errors
    /// If this automaton never accepts any input.
    #[inline]
    pub fn try_from_reversed(reversed: Dfa<I>) -> Result<Self, NeverAccepts> {
        if reversed.would_ever_accept() {
            Ok(Self {
                dfa: reversed,
                rng: rand::thread_rng(),
            })
        } else {
            Err(NeverAccepts)
        }
    }
}

impl core::fmt::Display for NeverAccepts {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Tried to fuzz an automaton that never accepts any input."
        )
    }
}

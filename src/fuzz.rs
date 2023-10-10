/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Infinite iterators over inputs guaranteed to be accepted by a given automaton.

use crate::{Compiled as Dfa, Expression};
use core::fmt;
use rand::{distributions::uniform::SampleUniform, rngs::ThreadRng, Rng};

/// Infinite iterator over inputs guaranteed to be accepted by a given automaton.
#[derive(Clone, Debug)]
pub struct Fuzzer<I: Clone + Expression + Ord> {
    /// Reversed automaton.
    dfa: Dfa<I>,
    /// Random number generator.
    rng: ThreadRng,
}

/// Tried to fuzz an automaton that never accepts any input.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NeverAccepts;

impl<I: Clone + Expression + Ord + SampleUniform> Iterator for Fuzzer<I> {
    type Item = Vec<I>;
    #[inline]
    #[allow(clippy::unwrap_in_result)]
    fn next(&mut self) -> Option<Self::Item> {
        'start_over: loop {
            let mut index = self.dfa.initial;
            let mut v = vec![];
            loop {
                let state = get!(self.dfa.states, index);
                if state.accepting.is_some() && self.rng.gen() {
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
                    .nth(self.rng.gen_range(0..state.transitions.len())));
                v.push(key.random(&mut self.rng));
                index = unwrap!(state.transitions.get(key)).dst;
            }
        }
    }
}

impl<I: Clone + Expression + Ord> Fuzzer<I> {
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

impl fmt::Display for NeverAccepts {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tried to fuzz an automaton that never accepts any input."
        )
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on NFAs.

use crate::{call::Call, nfa, range::Range, Expression, Parser as Nfa};
use core::{
    iter::once,
    ops::{Add, AddAssign, BitOr, Shr},
};

impl<I: Clone + Expression + Ord> AddAssign<usize> for nfa::State<I> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        // TODO: We can totally use unsafe here since the order doesn't change
        self.epsilon = self
            .epsilon
            .iter()
            .map(|x| x.checked_add(rhs).expect("Huge number of states"))
            .collect();
        for &mut nfa::Transition { ref mut dsts, .. } in &mut self.non_epsilon.values_mut() {
            *dsts = dsts
                .iter()
                .map(|&x| x.checked_add(rhs).expect("Huge number of states"))
                .collect();
        }
    }
}

impl<I: Clone + Expression + Ord> BitOr for Nfa<I> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn bitor(mut self, mut rhs: Self) -> Self::Output {
        let index = self.states.len();
        for state in &mut rhs.states {
            *state += index;
        }
        self.states.extend(rhs.states);
        self.initial.extend(
            rhs.initial
                .into_iter()
                .map(|x| x.checked_add(index).expect("Huge number of states")),
        );
        self
    }
}

impl<I: Clone + Expression + Ord> Shr<(Range<I>, Call, Nfa<I>)> for Nfa<I> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn shr(mut self, (token, fn_name, mut rhs): (Range<I>, Call, Nfa<I>)) -> Self::Output {
        let index = self.states.len();
        for state in &mut rhs.states {
            *state += index;
        }
        let incr_initial = rhs
            .initial
            .iter()
            .map(|x| x.checked_add(index).expect("Huge number of states"));
        for state in &mut self.states {
            if state.accepting.is_some() {
                state.accepting = None;
                state.non_epsilon.extend(
                    incr_initial
                        .clone()
                        .map(|i| {
                            (
                                token.clone(),
                                nfa::Transition {
                                    dsts: once(i).collect(),
                                    call: fn_name.clone(),
                                },
                            )
                        })
                        .clone(),
                );
            }
        }
        self.states.extend(rhs.states);
        self
    }
}

impl<I: Clone + Expression + Ord> Shr for Nfa<I> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn shr(mut self, mut rhs: Self) -> Self::Output {
        let index = self.states.len();
        for state in &mut rhs.states {
            *state += index;
        }
        let incr_initial = rhs
            .initial
            .iter()
            .map(|x| x.checked_add(index).expect("Huge number of states"));
        for state in &mut self.states {
            if state.accepting.is_some() {
                state.accepting = None;
                state.epsilon.extend(incr_initial.clone());
            }
        }
        self.states.extend(rhs.states);
        self
    }
}

impl Add for Nfa<char> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        self >> crate::space() >> rhs
    }
}

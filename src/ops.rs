/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on NFAs.

use crate::{nfa, Nfa};

impl<I: Clone + Ord> core::ops::AddAssign<usize> for nfa::State<I> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        // TODO: We can totally use unsafe here since the order doesn't change
        self.epsilon = self
            .epsilon
            .iter()
            .map(|x| x.checked_add(rhs).expect("Huge number of states"))
            .collect();
        for v in &mut self.non_epsilon.values_mut() {
            *v = v
                .iter()
                .map(|x| x.checked_add(rhs).expect("Huge number of states"))
                .collect();
        }
    }
}

impl<I: Clone + Ord> core::ops::BitOr for Nfa<I> {
    type Output = Nfa<I>;
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

impl<I: Clone + Ord> core::ops::Shr for Nfa<I> {
    type Output = Nfa<I>;
    #[inline]
    #[allow(clippy::todo)] // FIXME
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
            if state.accepting {
                state.accepting = false;
                state.epsilon.extend(incr_initial.clone());
            }
        }
        self.states.extend(rhs.states);
        self
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on NFAs.

use crate::{nondeterministic as n, Parser as N};

impl<I: Clone + Ord, S: Clone + Ord> core::ops::AddAssign<usize> for n::State<I, S> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        // TODO: We can totally use unsafe here since the order doesn't change
        self.epsilon = self
            .epsilon
            .iter()
            .map(|x| x.checked_add(rhs).expect("Huge number of states"))
            .collect();
        for &mut (ref mut set, _fn_name) in &mut self.non_epsilon.values_mut() {
            *set = set
                .iter()
                .map(|&x| x.checked_add(rhs).expect("Huge number of states"))
                .collect();
        }
    }
}

impl<I: Clone + Ord, S: Clone + Ord> core::ops::BitOr for N<I, S> {
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

impl<I: Clone + Ord, S: Clone + Ord> core::ops::Shr<(I, Option<&'static str>, N<I, S>)>
    for N<I, S>
{
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn shr(
        mut self,
        (token, fn_name, mut rhs): (I, Option<&'static str>, N<I, S>),
    ) -> Self::Output {
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
                state.non_epsilon.extend(
                    incr_initial
                        .clone()
                        .map(|i| (token.clone(), (core::iter::once(i).collect(), fn_name)))
                        .clone(),
                );
            }
        }
        self.states.extend(rhs.states);
        self
    }
}

impl<I: Clone + Ord, S: Clone + Ord> core::ops::Shr for N<I, S> {
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
            if state.accepting {
                state.accepting = false;
                state.epsilon.extend(incr_initial.clone());
            }
        }
        self.states.extend(rhs.states);
        self
    }
}

impl<S: Clone + Ord> core::ops::Add for N<char, S> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        self >> crate::space() >> rhs
    }
}

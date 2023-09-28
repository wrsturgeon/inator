/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on NFAs.

use crate::nfa::{Graph as Nfa, State};

/// Unevaluated binary operation.
#[non_exhaustive]
#[allow(clippy::ref_option_ref)]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Lazy<I: Clone + Ord> {
    /// NFA already made.
    Immediate(Nfa<I>),
    /// NFA promised.
    Postponed,
    /// NFA promised.
    PostponedReference(*const Self),
    /// Either one NFA or another, in parallel.
    Or(Box<Self>, Box<Self>),
    /// One then an epsilon transition to another.
    ShrEps(Box<Self>, Box<Self>),
    /// One then a non-epsilon transition (on a particular token) to another.
    ShrNonEps(Box<Self>, (I, Option<&'static str>, Box<Self>)),
    /// Repeat an NFA.
    Repeat(Box<Self>),
}

impl<I: Clone + Ord> Clone for Lazy<I> {
    #[inline]
    fn clone(&self) -> Self {
        match *self {
            Self::Immediate(ref lhs) => Self::Immediate(lhs.clone()),
            Self::Postponed => Self::PostponedReference(self), // <-- This is the crucial bit
            Self::PostponedReference(ptr) => Self::PostponedReference(ptr),
            Self::Or(ref lhs, ref rhs) => Self::Or(lhs.clone(), rhs.clone()),
            Self::ShrEps(ref lhs, ref rhs) => Self::ShrEps(lhs.clone(), rhs.clone()),
            Self::ShrNonEps(ref lhs, ref rhs) => Self::ShrNonEps(lhs.clone(), rhs.clone()),
            Self::Repeat(ref lhs) => Self::Repeat(lhs.clone()),
        }
    }
}

impl<I: Clone + Ord> Lazy<I> {
    /// Define a postponed value.
    /// # Panics
    /// If this value was already defined or if the definition is still postponed.
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    pub fn finally(&mut self, value: Self) {
        if *self != Self::Postponed {
            panic!("Called `finally` on a value that was already defined");
        }
        if value == Self::Postponed {
            panic!("Called `finally` with a definition that is itself postponed");
        }
        *self = value;
    }

    /// Brzozowski's algorithm for minimizing automata.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_assert_message)]
    pub fn compile(self) -> crate::dfa::Graph<I> {
        self.evaluate().compile()
    }

    /// Match at least one time, then as many times as we want.
    /// Note that if ANY number of times leads to an accepting state, we take it!
    #[inline]
    #[must_use]
    pub fn repeat(self) -> Self {
        Lazy::Repeat(Box::new(self))
    }

    /// Match at most one time (i.e. ignore if not present).
    #[inline]
    #[must_use]
    pub fn optional(self) -> Self {
        crate::empty() | self
    }

    /// Match zero or more times (a.k.a. Kleene star).
    #[inline]
    #[must_use]
    pub fn star(self) -> Self {
        self.repeat().optional()
    }

    /// Turn an expression into a value.
    /// Note that this requires all postponed terms to be present.
    /// # Panics
    /// If we postponed a value and never defined it.
    #[inline]
    #[allow(
        clippy::arithmetic_side_effects,
        clippy::panic,
        clippy::shadow_reuse,
        clippy::suspicious_arithmetic_impl,
        unsafe_code
    )]
    pub fn evaluate(self) -> Nfa<I> {
        match self {
            Self::Immediate(nfa) => nfa,
            Self::Postponed => panic!("Needed a postponed value that had not been initialized"),
            // SAFETY: Up to you. Don't be stupid. <3
            Self::PostponedReference(ptr) => unsafe { &*ptr }.clone().evaluate(),
            Self::Or(lhs, rhs) => {
                let mut lhs = lhs.evaluate();
                let mut rhs = rhs.evaluate();
                let index = lhs.states.len();
                for state in &mut rhs.states {
                    *state += index;
                }
                lhs.states.extend(rhs.states);
                lhs.initial.extend(
                    rhs.initial
                        .into_iter()
                        .map(|x| x.checked_add(index).expect("Huge number of states")),
                );
                lhs
            }
            Self::ShrEps(lhs, rhs) => {
                let mut lhs = lhs.evaluate();
                let mut rhs = rhs.evaluate();
                let index = lhs.states.len();
                for state in &mut rhs.states {
                    *state += index;
                }
                let incr_initial = rhs
                    .initial
                    .iter()
                    .map(|x| x.checked_add(index).expect("Huge number of states"));
                for state in &mut lhs.states {
                    if state.accepting {
                        state.accepting = false;
                        state.epsilon.extend(incr_initial.clone());
                    }
                }
                lhs.states.extend(rhs.states);
                lhs
            }
            Self::ShrNonEps(lhs, (token, fn_name, rhs)) => {
                let mut lhs = lhs.evaluate();
                let mut rhs = rhs.evaluate();
                let index = lhs.states.len();
                for state in &mut rhs.states {
                    *state += index;
                }
                let incr_initial = rhs
                    .initial
                    .iter()
                    .map(|x| x.checked_add(index).expect("Huge number of states"));
                for state in &mut lhs.states {
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
                lhs.states.extend(rhs.states);
                lhs
            }
            Self::Repeat(lhs) => {
                let mut eval = lhs.evaluate();
                for state in &mut eval.states {
                    if state.accepting {
                        state.epsilon.extend(eval.initial.iter());
                    }
                }
                eval
            }
        }
    }
}

impl<I: Clone + Ord> core::ops::AddAssign<usize> for State<I> {
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

impl<I: Clone + Ord> core::ops::BitOr for Lazy<I> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Lazy::Or(Box::new(self), Box::new(rhs))
    }
}

impl<I: Clone + Ord> core::ops::Shr<(I, Option<&'static str>, Lazy<I>)> for Lazy<I> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn shr(self, (token, fn_name, rhs): (I, Option<&'static str>, Self)) -> Self::Output {
        Lazy::ShrNonEps(Box::new(self), (token, fn_name, Box::new(rhs)))
    }
}

impl<I: Clone + Ord> core::ops::Shr for Lazy<I> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn shr(self, rhs: Self) -> Self::Output {
        Lazy::ShrEps(Box::new(self), Box::new(rhs))
    }
}

impl core::ops::Add for Lazy<char> {
    type Output = Self;
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        self >> crate::space() >> rhs
    }
}

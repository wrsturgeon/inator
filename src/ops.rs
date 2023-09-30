/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on NFAs.

use std::{
    cell::OnceCell,
    hash::Hash,
    rc::{Rc, Weak},
};

use crate::nfa::{Graph as Nfa, State};

#[derive(Debug, Eq, PartialEq)]
pub struct Postponed<I: Clone + Ord>(pub(crate) Rc<OnceCell<Lazy<I>>>);
#[derive(Debug, Clone)]
pub struct PostponedRef<I: Clone + Ord>(pub(crate) Weak<OnceCell<Lazy<I>>>);

impl<I: Clone + Ord + Hash> Hash for Postponed<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.get().unwrap().hash(state)
    }
}

impl<I: Clone + Ord> PartialOrd for Postponed<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.get().unwrap().partial_cmp(other.0.get().unwrap())
    }
}

impl<I: Clone + Ord> Ord for Postponed<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.get().unwrap().cmp(other.0.get().unwrap())
    }
}

impl<I: Clone + Ord> PartialEq for PostponedRef<I> {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .upgrade()
            .unwrap()
            .get()
            .unwrap()
            .eq(other.0.upgrade().unwrap().get().unwrap())
    }
}

impl<I: Clone + Ord> Eq for PostponedRef<I> {}

impl<I: Clone + Ord + Hash> Hash for PostponedRef<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.upgrade().unwrap().get().unwrap().hash(state)
    }
}

impl<I: Clone + Ord> PartialOrd for PostponedRef<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0
            .upgrade()
            .unwrap()
            .get()
            .unwrap()
            .partial_cmp(other.0.upgrade().unwrap().get().unwrap())
    }
}

impl<I: Clone + Ord> Ord for PostponedRef<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .upgrade()
            .unwrap()
            .get()
            .unwrap()
            .cmp(other.0.upgrade().unwrap().get().unwrap())
    }
}

/// Unevaluated binary operation.
#[non_exhaustive]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Lazy<I: Clone + Ord> {
    /// NFA already made.
    Immediate(Nfa<I>),
    /// NFA promised.
    Postponed(Postponed<I>),
    /// NFA promised.
    PostponedReference(PostponedRef<I>),
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
        match self {
            Self::Immediate(ref lhs) => Self::Immediate(lhs.clone()),
            Self::Postponed(Postponed(cell)) => {
                Self::PostponedReference(PostponedRef(Rc::downgrade(&cell)))
            }
            Self::PostponedReference(reference) => Self::PostponedReference(reference.clone()),
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
    /// If this value was already defined.
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    pub fn finally(&mut self, value: Self) {
        if let Self::Postponed(Postponed(cell)) = self {
            // map_err to discard the "error", since I doesn't implement Debug
            cell.set(value)
                .map_err(|_| "value already defined")
                .unwrap();
        }
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
            Self::Postponed(Postponed(cell)) => cell
                .get()
                .cloned()
                .expect("value still postponed")
                .evaluate(),
            Self::PostponedReference(PostponedRef(cell)) => cell
                .upgrade()
                .unwrap()
                .get()
                .cloned()
                .expect("value still postponed")
                .evaluate(),
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

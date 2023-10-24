/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Check well-formedness.

use crate::{
    Action, Ctrl, CurryInput, CurryStack, Input, Output, Range, RangeMap, Stack, State, Transition,
    Update,
};
use core::num::NonZeroUsize;
use std::collections::BTreeSet;

/// Witness to an ill-formed automaton (or part thereof).
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IllFormed<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// An index points to a state greater than the total number of states.
    OutOfBounds(usize),
    /// A set of indices contains no elements (we should just delete the transition).
    ProlongingDeath,
    /// A `Range`'s `first` field measured greater than its `last` field.
    InvertedRange(I, I),
    /// In a `RangeMap`, at least one key could be accepted by two existing ranges of keys.
    RangeMapOverlap(Range<I>),
    /// In a `CurryStack`, a wildcard matches an input that a specific key also matches.
    WildcardMask {
        /// Top of the stack, if one existed.
        arg_stack: Option<S>,
        /// Input token (or range thereof) that could be ambiguous.
        arg_token: Option<Range<I>>,
        /// First output possibility.
        possibility_1: Transition<I, S, O, C>,
        /// Second output possibility.
        possibility_2: Transition<I, S, O, C>,
    },
    /// Can't go to two different (deterministic) states at the same time.
    Superposition(usize, usize),
    /// Can't e.g. push and pop from the stack at the same time.
    IncompatibleStackActions(Action<S>, Action<S>),
    /// Can't call two different functions on half-constructed outputs at the same time.
    IncompatibleCallbacks(Update<I, O>, Update<I, O>),
    /// Two identical states at different indices.
    DuplicateState,
    /// States out of sorted order in memory.
    UnsortedStates,
    /// Reference to a tagged state, but no state has that tag.
    TagDNE(String),
    /// Two states have identical tags.
    DuplicateTag(String),
}

impl<I: Input, S: Stack, O: Output> IllFormed<I, S, O, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I, S, O>>(self) -> IllFormed<I, S, O, C> {
        match self {
            IllFormed::OutOfBounds(i) => IllFormed::OutOfBounds(i),
            IllFormed::ProlongingDeath => IllFormed::ProlongingDeath,
            IllFormed::InvertedRange(a, b) => IllFormed::InvertedRange(a, b),
            IllFormed::RangeMapOverlap(range) => IllFormed::RangeMapOverlap(range),
            IllFormed::WildcardMask {
                arg_stack,
                arg_token,
                possibility_1,
                possibility_2,
            } => IllFormed::WildcardMask {
                arg_stack,
                arg_token,
                possibility_1: possibility_1.convert_ctrl(),
                possibility_2: possibility_2.convert_ctrl(),
            },
            IllFormed::Superposition(a, b) => IllFormed::Superposition(a, b),
            IllFormed::IncompatibleStackActions(a, b) => IllFormed::IncompatibleStackActions(a, b),
            IllFormed::IncompatibleCallbacks(a, b) => IllFormed::IncompatibleCallbacks(a, b),
            IllFormed::DuplicateState => IllFormed::DuplicateState,
            IllFormed::UnsortedStates => IllFormed::UnsortedStates,
            IllFormed::TagDNE(s) => IllFormed::TagDNE(s),
            IllFormed::DuplicateTag(s) => IllFormed::DuplicateTag(s),
        }
    }
}

/// Check well-formedness.
pub trait Check<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Check well-formedness.
    /// # Errors
    /// When not well-formed (with a witness).
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>>;
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for Action<S> {
    #[inline]
    fn check(&self, _: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        Ok(())
    }
}

impl<I: Input, S: Stack, O: Output> Check<I, S, O, BTreeSet<Result<usize, String>>>
    for BTreeSet<Result<usize, String>>
{
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, Self>> {
        if self.is_empty() {
            return Err(IllFormed::ProlongingDeath);
        }
        for r in self {
            if let &Ok(i) = r {
                if i >= n_states.into() {
                    return Err(IllFormed::OutOfBounds(i));
                }
            }
        }
        Ok(())
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for CurryStack<I, S, O, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        if let Some(ref wild) = self.wildcard {
            for (key, some) in &self.map_some {
                wild.disjoint(some)
                    .map_err(|(arg_token, possibility_1, possibility_2)| {
                        IllFormed::WildcardMask {
                            arg_stack: Some(key.clone()),
                            arg_token,
                            possibility_1,
                            possibility_2,
                        }
                    })?;
            }
            if let Some(ref none) = self.map_none {
                wild.disjoint(none)
                    .map_err(|(arg_token, possibility_1, possibility_2)| {
                        IllFormed::WildcardMask {
                            arg_stack: None,
                            arg_token,
                            possibility_1,
                            possibility_2,
                        }
                    })?;
            }
            wild.check(n_states)?;
        }
        for some in self.map_some.values() {
            some.check(n_states)?;
        }
        self.map_none
            .as_ref()
            .map_or(Ok(()), |none| none.check(n_states))
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for CurryInput<I, S, O, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        match *self {
            Self::Wildcard(ref etc) => etc.check(n_states),
            Self::Scrutinize(ref etc) => etc.check(n_states),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for Range<I> {
    #[inline]
    fn check(&self, _: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        if self.first <= self.last {
            Ok(())
        } else {
            Err(IllFormed::InvertedRange(
                self.first.clone(),
                self.last.clone(),
            ))
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for RangeMap<I, S, O, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        self.iter().try_fold((), |(), (k, v)| {
            self.entries
                .range(..k.clone())
                .fold(None, |acc, (range, _)| {
                    acc.or_else(|| range.clone().intersection(k.clone()))
                })
                .map_or_else(
                    || v.check(n_states),
                    |overlap| Err(IllFormed::RangeMapOverlap(overlap)),
                )
        })
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for State<I, S, O, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        self.transitions.check(n_states)
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Check<I, S, O, C> for Transition<I, S, O, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, C>> {
        self.dst.check(n_states)?;
        self.act.check(n_states)
    }
}

impl<I: Input, S: Stack, O: Output> Check<I, S, O, usize> for usize {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, Self>> {
        if *self >= n_states.into() {
            Err(IllFormed::OutOfBounds(*self))
        } else {
            Ok(())
        }
    }
}

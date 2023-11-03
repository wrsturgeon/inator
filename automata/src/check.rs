/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Check well-formedness.

use crate::{
    Action, Ctrl, CurryInput, CurryStack, Input, Range, RangeMap, Stack, State, ToSrc, Transition,
    Update,
};
use core::{fmt, mem, num::NonZeroUsize};
use std::collections::BTreeSet;

/// Maximum size we're willing to tolerate in an `Err` variant (for performance reasons).
const _MAX_ILL_FORMED_BYTES: usize = 64;
/// Check that the above holds by throwing a compile-time out-of-bounds error if it doesn't.
#[allow(clippy::indexing_slicing)] // <-- that's the point
const _: () = [(); _MAX_ILL_FORMED_BYTES][mem::size_of::<IllFormed<(), (), usize>>()];

/// Witness to an ill-formed automaton (or part thereof).
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IllFormed<I: Input, S: Stack, C: Ctrl<I, S>> {
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
        possibility_1: Box<Transition<I, S, C>>,
        /// Second output possibility.
        possibility_2: Box<Transition<I, S, C>>,
    },
    /// Can't go to two different (deterministic) states at the same time.
    Superposition(usize, usize),
    /// Can't e.g. push and pop from the stack at the same time.
    IncompatibleStackActions(Action<S>, Action<S>),
    /// Can't call two different functions on half-constructed outputs at the same time.
    IncompatibleCallbacks(Box<Update<I>>, Box<Update<I>>),
    /// Two identical states at different indices.
    DuplicateState(Box<State<I, S, C>>),
    /// Reference to a tagged state, but no state has that tag.
    TagDNE(String),
    /// An initial state expects an accumulator argument that is not `()`.
    InitialNotUnit(String),
    /// Tried to merge two states who need different output types.
    TypeMismatch(String, String),
    /// An accepting state returns the wrong type.
    WrongReturnType(String, String),
}

impl<I: Input, S: Stack> IllFormed<I, S, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I, S>>(self) -> IllFormed<I, S, C> {
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
                possibility_1: Box::new(possibility_1.convert_ctrl()),
                possibility_2: Box::new(possibility_2.convert_ctrl()),
            },
            IllFormed::Superposition(a, b) => IllFormed::Superposition(a, b),
            IllFormed::IncompatibleStackActions(a, b) => IllFormed::IncompatibleStackActions(a, b),
            IllFormed::IncompatibleCallbacks(a, b) => IllFormed::IncompatibleCallbacks(a, b),
            IllFormed::DuplicateState(s) => IllFormed::DuplicateState(Box::new(s.convert_ctrl())),
            IllFormed::TagDNE(s) => IllFormed::TagDNE(s),
            IllFormed::InitialNotUnit(s) => IllFormed::InitialNotUnit(s),
            IllFormed::TypeMismatch(a, b) => IllFormed::TypeMismatch(a, b),
            IllFormed::WrongReturnType(a, b) => IllFormed::WrongReturnType(a, b),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> fmt::Display for IllFormed<I, S, C> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::OutOfBounds(i) => write!(f, "State index out of bounds: {i}"),
            Self::ProlongingDeath => write!(
                f,
                "Transition to a state that will never accept. \
                Try removing the state along with any transitions to it.",
            ),
            Self::InvertedRange(ref a, ref b) => {
                write!(
                    f,
                    "Range with endpoints flipped: {}..={}",
                    a.to_src(),
                    b.to_src(),
                )
            }
            Self::RangeMapOverlap(ref r) => {
                write!(f, "Multiple ranges would accept {}", r.to_src())
            }
            Self::WildcardMask {
                ref arg_stack,
                ref arg_token,
                ref possibility_1,
                ref possibility_2,
            } => {
                write!(
                    f,
                    "When the stack top is {} and the token is {}, \
                    a wildcard match succeeds (`{}`), \
                    but so does a specific match (`{}`).",
                    arg_stack.to_src(),
                    arg_token
                        .as_ref()
                        .map_or("None".to_owned(), |r| if r.first == r.last {
                            r.first.to_src()
                        } else {
                            r.to_src()
                        }),
                    possibility_1.to_src(),
                    possibility_2.to_src(),
                )
            }
            Self::Superposition(a, b) => write!(
                f,
                "Tried to visit two different deterministic states \
                ({a} and {b}) at the same time.",
            ),
            Self::IncompatibleStackActions(ref a, ref b) => {
                write!(
                    f,
                    "Can't {} and {} the stack at the same time.",
                    a.in_english(),
                    b.in_english(),
                )
            }
            Self::IncompatibleCallbacks(ref a, ref b) => {
                write!(
                    f,
                    "Tried to call both `{}` and `{}` at the same time.",
                    a.src, b.src,
                )
            }
            Self::DuplicateState(ref s) => write!(f, "Duplicate state: {}", s.to_src()),
            Self::TagDNE(ref tag) => write!(
                f,
                "Requested a transition to a tag that does not exist: \"{tag}\"",
            ),
            Self::InitialNotUnit(ref s) => write!(
                f,
                "Initial state needs to take a unit-type input (`()`) but takes `{s}` instead.",
            ),
            Self::TypeMismatch(ref a, ref b) => write!(f, "Type mismatch: `{a}` =/= `{b}`."),
            Self::WrongReturnType(ref a, ref b) => write!(f, "Wrong output type: `{a}` =/= `{b}`"),
        }
    }
}

/// Check well-formedness.
pub trait Check<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Check well-formedness.
    /// # Errors
    /// When not well-formed (with a witness).
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>>;
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for Action<S> {
    #[inline]
    fn check(&self, _: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        Ok(())
    }
}

impl<I: Input, S: Stack> Check<I, S, BTreeSet<Result<usize, String>>>
    for BTreeSet<Result<usize, String>>
{
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, Self>> {
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

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for CurryStack<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        if let Some(ref wild) = self.wildcard {
            for (key, some) in &self.map_some {
                wild.disjoint(some)
                    .map_err(|(arg_token, possibility_1, possibility_2)| {
                        IllFormed::WildcardMask {
                            arg_stack: Some(key.clone()),
                            arg_token,
                            possibility_1: Box::new(possibility_1),
                            possibility_2: Box::new(possibility_2),
                        }
                    })?;
            }
            if let Some(ref none) = self.map_none {
                wild.disjoint(none)
                    .map_err(|(arg_token, possibility_1, possibility_2)| {
                        IllFormed::WildcardMask {
                            arg_stack: None,
                            arg_token,
                            possibility_1: Box::new(possibility_1),
                            possibility_2: Box::new(possibility_2),
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

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for CurryInput<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        match *self {
            Self::Wildcard(ref etc) => etc.check(n_states),
            Self::Scrutinize(ref etc) => etc.check(n_states),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for Range<I> {
    #[inline]
    fn check(&self, _: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
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

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for RangeMap<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
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

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for State<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        self.transitions.check(n_states)
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Check<I, S, C> for Transition<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        self.dst.check(n_states)?;
        self.act.check(n_states)
    }
}

impl<I: Input, S: Stack> Check<I, S, usize> for usize {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, Self>> {
        if *self >= n_states.into() {
            Err(IllFormed::OutOfBounds(*self))
        } else {
            Ok(())
        }
    }
}

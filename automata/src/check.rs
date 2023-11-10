/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Check well-formedness.

use crate::{Ctrl, Curry, Input, Range, RangeMap, State, ToSrc, Transition, Update, FF};
use core::{fmt, mem, num::NonZeroUsize};
use std::collections::BTreeSet;

/// Maximum size we're willing to tolerate in an `Err` variant (for performance reasons).
const _MAX_ILL_FORMED_BYTES: usize = 64;
/// Check that the above holds by throwing a compile-time out-of-bounds error if it doesn't.
#[allow(clippy::indexing_slicing)] // <-- that's the point
const _: () = [(); _MAX_ILL_FORMED_BYTES][mem::size_of::<IllFormed<(), usize>>()];

/// Witness to an ill-formed automaton (or part thereof).
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IllFormed<I: Input, C: Ctrl<I>> {
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
        /// Input token (or range thereof) that could be ambiguous.
        arg_token: Option<Range<I>>,
        /// First output possibility.
        possibility_1: Box<Transition<I, C>>,
        /// Second output possibility.
        possibility_2: Box<Transition<I, C>>,
    },
    /// Can't go to two different (deterministic) states at the same time.
    Superposition(usize, usize),
    /// Can't call two different functions on half-constructed outputs at the same time.
    IncompatibleCallbacks(Box<Update<I>>, Box<Update<I>>),
    /// Can't call two different functions to combine a returned value with a saved one at the same time.
    IncompatibleCombinators(Box<FF>, Box<FF>),
    /// Can't e.g. push to the stack and pop from it at the same time.
    IncompatibleActions(Box<Transition<I, C>>, Box<Transition<I, C>>),
    /// Two identical states at different indices.
    DuplicateState(Box<State<I, C>>),
    /// Reference to a tagged state, but no state has that tag.
    TagDNE(String),
    /// An initial state expects an accumulator argument that is not `()`.
    InitialNotUnit(String),
    /// Tried to merge two states who need different output types.
    TypeMismatch(String, String),
    /// An accepting state returns the wrong type.
    WrongReturnType(String, String),
    /// Ambiguous regions: e.g. claiming to be opening both parentheses and brackets at the same time.
    AmbiguousRegions(&'static str, &'static str),
}

impl<I: Input> IllFormed<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> IllFormed<I, C> {
        match self {
            IllFormed::OutOfBounds(i) => IllFormed::OutOfBounds(i),
            IllFormed::ProlongingDeath => IllFormed::ProlongingDeath,
            IllFormed::InvertedRange(a, b) => IllFormed::InvertedRange(a, b),
            IllFormed::RangeMapOverlap(range) => IllFormed::RangeMapOverlap(range),
            IllFormed::WildcardMask {
                arg_token,
                possibility_1,
                possibility_2,
            } => IllFormed::WildcardMask {
                arg_token,
                possibility_1: Box::new(possibility_1.convert_ctrl()),
                possibility_2: Box::new(possibility_2.convert_ctrl()),
            },
            IllFormed::Superposition(a, b) => IllFormed::Superposition(a, b),
            IllFormed::IncompatibleCallbacks(a, b) => IllFormed::IncompatibleCallbacks(a, b),
            IllFormed::IncompatibleCombinators(a, b) => IllFormed::IncompatibleCombinators(a, b),
            IllFormed::IncompatibleActions(a, b) => IllFormed::IncompatibleActions(
                Box::new(a.convert_ctrl()),
                Box::new(b.convert_ctrl()),
            ),
            IllFormed::DuplicateState(s) => IllFormed::DuplicateState(Box::new(s.convert_ctrl())),
            IllFormed::TagDNE(s) => IllFormed::TagDNE(s),
            IllFormed::InitialNotUnit(s) => IllFormed::InitialNotUnit(s),
            IllFormed::TypeMismatch(a, b) => IllFormed::TypeMismatch(a, b),
            IllFormed::WrongReturnType(a, b) => IllFormed::WrongReturnType(a, b),
            IllFormed::AmbiguousRegions(a, b) => IllFormed::AmbiguousRegions(a, b),
        }
    }
}

impl<I: Input, C: Ctrl<I>> fmt::Display for IllFormed<I, C> {
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
                ref arg_token,
                ref possibility_1,
                ref possibility_2,
            } => {
                write!(
                    f,
                    "On token {}, \
                    a wildcard match succeeds (`{}`), \
                    but so does a specific match (`{}`).",
                    arg_token.as_ref().map_or("[end of input]".to_owned(), |r| {
                        if r.first == r.last {
                            r.first.to_src()
                        } else {
                            r.to_src()
                        }
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
            Self::IncompatibleCallbacks(ref a, ref b) => {
                write!(
                    f,
                    "Tried to call both `{}` and `{}` at the same time.",
                    a.src, b.src,
                )
            }
            Self::IncompatibleCombinators(ref a, ref b) => {
                write!(
                    f,
                    "Tried to call both `{}` and `{}` at the same time.",
                    a.src, b.src,
                )
            }
            Self::IncompatibleActions(ref a, ref b) => write!(
                f,
                "Tried to {} and {} at the same time.",
                a.in_english(),
                b.in_english()
            ),
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
            Self::AmbiguousRegions(a, b) => write!(
                f,
                "Claiming to open two different regions (\"{a}\" and \"{b}\") simultaneously."
            ),
        }
    }
}

/// Check well-formedness.
pub trait Check<I: Input, C: Ctrl<I>> {
    /// Check well-formedness.
    /// # Errors
    /// When not well-formed (with a witness).
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, C>>;
}

impl<I: Input> Check<I, BTreeSet<usize>> for BTreeSet<usize> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, Self>> {
        if self.is_empty() {
            return Err(IllFormed::ProlongingDeath);
        }
        for &i in self {
            if i >= n_states.into() {
                return Err(IllFormed::OutOfBounds(i));
            }
        }
        Ok(())
    }
}

impl<I: Input, C: Ctrl<I>> Check<I, C> for Curry<I, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, C>> {
        match *self {
            Self::Wildcard(ref etc) => etc.check(n_states),
            Self::Scrutinize(ref etc) => etc.check(n_states),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Check<I, C> for Range<I> {
    #[inline]
    fn check(&self, _: NonZeroUsize) -> Result<(), IllFormed<I, C>> {
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

impl<I: Input, C: Ctrl<I>> Check<I, C> for RangeMap<I, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, C>> {
        self.iter().try_fold((), |(), (k, v)| {
            self.0
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

impl<I: Input, C: Ctrl<I>> Check<I, C> for State<I, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, C>> {
        self.transitions.check(n_states)
    }
}

impl<I: Input, C: Ctrl<I>> Check<I, C> for Transition<I, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, C>> {
        match *self {
            Self::Lateral { ref dst, .. } | Self::Call { ref dst, .. } => dst.check(n_states),
            Self::Return { .. } => Ok(()),
        }
    }
}

impl<I: Input> Check<I, usize> for usize {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, Self>> {
        if *self >= n_states.into() {
            Err(IllFormed::OutOfBounds(*self))
        } else {
            Ok(())
        }
    }
}

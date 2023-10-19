/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Check well-formedness.

use crate::{Action, Ctrl, Input, Output, Range, RangeMap, Stack, Transition};
use core::num::NonZeroUsize;
use std::collections::BTreeSet;

/// Witness to an ill-formed automaton (or part thereof).
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IllFormed<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// An index points to a state greater than the total number of states.
    OutOfBounds(usize),
    /// A set of indices contains no elements (we should just delete the transition).
    ProlongingDeath,
    /// A `Range`'s `first` field measured greater than its `last` field.
    InvertedRange(I, I),
    /// In a `RangeMap`, at least one key could be accepted by two existing ranges of keys.
    RangeMapOverlap(Range<I>),
    /// In a `CurryState`, a wildcard matches an input that a specific key also matches.
    WildcardMask(Transition<I, S, O, C>, Transition<I, S, O, C>),
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
        self.iter()
            .enumerate()
            .try_fold((), |(), (i, &(ref k, ref v))| {
                get!(self.entries, ..i)
                    .iter()
                    .fold(None, |acc, &(ref range, _)| {
                        acc.or_else(|| range.clone().intersection(k.clone()))
                    })
                    .map_or_else(
                        || v.check(n_states),
                        |overlap| Err(IllFormed::RangeMapOverlap(overlap)),
                    )
            })
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

impl<I: Input, S: Stack, O: Output> Check<I, S, O, BTreeSet<usize>> for BTreeSet<usize> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, O, Self>> {
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

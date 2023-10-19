/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Check well-formedness.

use crate::{Action, Ctrl, Input, Range, RangeMap, Transition};
use core::{convert::Infallible, marker::PhantomData, num::NonZeroUsize};

/// Witness to an ill-formed automaton (or part thereof).
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IllFormed<I: Input, S, C: Ctrl> {
    /// An index points to a state greater than the total number of states.
    OutOfBounds(usize),
    /// A `Range`'s `first` field measured greater than its `last` field.
    InvertedRange(I, I),
    /// In a `RangeMap`, at least one key could be accepted by two existing ranges of keys.
    RangeMapOverlap(Range<I>),
    /// Bullshit state for the type checker.
    Phantom(Infallible, PhantomData<(S, C)>), // <-- FIXME: remove
}

/// Check well-formedness.
pub trait Check<I: Input, S, C: Ctrl> {
    /// Check well-formedness.
    /// # Errors
    /// When not well-formed (with a witness).
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>>;
}

impl<I: Input, S, C: Ctrl> Check<I, S, C> for Action<I, S> {
    #[inline]
    fn check(&self, _: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        Ok(())
    }
}

impl<I: Input, S, C: Ctrl> Check<I, S, C> for Range<I> {
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

impl<I: Input, S, C: Ctrl> Check<I, S, C> for RangeMap<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        self.iter()
            .enumerate()
            .try_fold((), |(), (i, &(ref k, ref v))| {
                get!(self, ..i)
                    .iter()
                    .fold(None, |acc, &(ref range, _)| {
                        acc.or_else(|| range.intersection(k))
                    })
                    .map_or_else(
                        || v.check(n_states),
                        |overlap| Err(IllFormed::RangeMapOverlap(overlap)),
                    )
            })
    }
}

impl<I: Input, S, C: Ctrl> Check<I, S, C> for Transition<I, S, C> {
    #[inline]
    fn check(&self, n_states: NonZeroUsize) -> Result<(), IllFormed<I, S, C>> {
        let size = n_states.into();
        for i in self.dst.view() {
            if i >= size {
                return Err(IllFormed::OutOfBounds(i));
            }
        }
        self.act.check(n_states)
    }
}

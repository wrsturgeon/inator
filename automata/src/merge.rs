/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::{Action, Ctrl, CtrlMergeConflict, IllFormed, Input, Output, Stack, Transition, Update};
use std::collections::BTreeSet;

/// Trait to fallibly combine multiple values into one value with identical semantics.
pub trait Merge: Sized {
    /// Implementation-defined error providing a witness to the reason the merge failed.
    type Error;
    /// Fallibly combine multiple values into one value with identical semantics.
    /// # Errors
    /// Implementation-defined: if the merge as we define it can't work.
    fn merge(self, other: Self) -> Result<Self, Self::Error>;
}

/// Merge a collection of elements into one.
/// Return `None` if the collection was empty.
#[inline]
pub fn merge<M: Merge, In: IntoIterator<Item = M>>(input: In) -> Option<Result<M, M::Error>> {
    let mut iter = input.into_iter();
    let first = iter.next()?;
    Some(iter.try_fold(first, Merge::merge))
}

/// Merge a collection of `Result`s possibly containing elements into one.
/// Return `None` if the collection was empty.
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn try_merge<M: Merge, In: IntoIterator<Item = Result<M, M::Error>>>(
    input: In,
) -> Option<Result<M, M::Error>> {
    let mut iter = input.into_iter();
    iter.next()?.map_or_else(
        |e| Some(Err(e)),
        |first| Some(iter.try_fold(first, |acc, m| acc.merge(m?))),
    )
}

impl Merge for usize {
    type Error = CtrlMergeConflict;
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err(CtrlMergeConflict::NotEqual(self, other))
        }
    }
}

impl<T: Ord> Merge for BTreeSet<T> {
    type Error = CtrlMergeConflict;
    #[inline]
    fn merge(mut self, other: Self) -> Result<Self, Self::Error> {
        self.extend(other);
        Ok(self)
    }
}

impl<T: Clone> Merge for Option<T> {
    type Error = (T, T);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        match (self, other) {
            (None, None) => Ok(None),
            (Some(a), None) => Ok(Some(a)),
            (None, Some(b)) => Ok(Some(b)),
            (Some(a), Some(b)) => Err((a, b)),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Merge for Transition<I, S, O, C> {
    type Error = IllFormed<I, S, O, C>;
    #[inline]
    #[allow(clippy::todo, unused_variables)] // <-- TODO: formalize errors
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        Ok(Self {
            dst: self
                .dst
                .merge(other.dst)
                .map_err(|e: CtrlMergeConflict| match e {
                    CtrlMergeConflict::NotEqual(a, b) => todo!(),
                })?,
            update: self
                .update
                .merge(other.update)
                .map_err(|(a, b): (Update<I, O>, Update<I, O>)| todo!())?,
            act: self
                .act
                .merge(other.act)
                .map_err(|(a, b): (Action<S>, Action<S>)| todo!())?,
        })
    }
}

impl<I: Input, O: Output> Merge for Update<I, O> {
    type Error = (Self, Self);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err((self, other))
        }
    }
}

impl<S: Stack> Merge for Action<S> {
    type Error = (Self, Self);
    #[inline]
    fn merge(self, other: Self) -> Result<Self, Self::Error> {
        if self == other {
            Ok(self)
        } else {
            Err((self, other))
        }
    }
}

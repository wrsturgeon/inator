/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Necessary preconditions to function as an index.

use crate::{Check, Input, Output, Stack};
use core::iter;
use std::collections::{btree_set, BTreeSet};

/// Necessary preconditions to function as an index.
pub trait Ctrl<I: Input, S: Stack, O: Output>: Check<I, S, O, Self> + Clone {
    /// Non-owning view over each index in what may be a collection.
    type View<'s>: Iterator<Item = usize>
    where
        Self: 's;
    /// View each index in what may be a collection.
    fn view(&self) -> Self::View<'_>;
}

impl<I: Input, S: Stack, O: Output> Ctrl<I, S, O> for usize {
    type View<'s> = iter::Once<usize>;
    #[inline]
    fn view(&self) -> Self::View<'_> {
        iter::once(*self)
    }
}

impl<I: Input, S: Stack, O: Output> Ctrl<I, S, O> for BTreeSet<usize> {
    type View<'s> = iter::Copied<btree_set::Iter<'s, usize>>;
    #[inline]
    fn view(&self) -> Self::View<'_> {
        self.iter().copied()
    }
}

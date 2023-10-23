/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Necessary preconditions to function as an index.

use crate::{Check, Input, Merge, Output, Stack, ToSrc};
use core::iter;
use std::collections::{btree_set, BTreeSet};

#[cfg(feature = "quickcheck")]
use core::num::NonZeroUsize;

/// Everything that could go wrong merging _any_ kind of indices.
/// No claim to be exhaustive: just the kinds that I've implemented so far.
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CtrlMergeConflict {
    /// Tried to merge two literal `usize`s that were not equal.
    NotEqual(usize, usize),
}

/// Necessary preconditions to function as an index.
pub trait Ctrl<I: Input, S: Stack, O: Output>:
    Check<I, S, O, Self> + Clone + Merge<Error = CtrlMergeConflict> + Ord + PartialEq + ToSrc
{
    /// Non-owning view over each index in what may be a collection.
    type View<'s>: Iterator<Item = usize>
    where
        Self: 's;
    /// View each index in what may be a collection.
    fn view(&self) -> Self::View<'_>;
    /// Arbitrary value of this type, given an automaton with this many states.
    /// Should fail occasionally but not often.
    #[must_use]
    #[cfg(feature = "quickcheck")]
    fn arbitrary_given(n_states: usize, g: &mut quickcheck::Gen, well_formed: bool) -> Self;
    /// Apply a function to each index.
    #[must_use]
    fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self;
}

impl<I: Input, S: Stack, O: Output> Ctrl<I, S, O> for usize {
    type View<'s> = iter::Once<usize>;
    #[inline]
    fn view(&self) -> Self::View<'_> {
        iter::once(*self)
    }
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::unwrap_used, unsafe_code)]
    #[cfg(feature = "quickcheck")]
    fn arbitrary_given(n_states: usize, g: &mut quickcheck::Gen, well_formed: bool) -> Self {
        use quickcheck::Arbitrary;
        Self::arbitrary(g)
            % NonZeroUsize::new(if well_formed { n_states } else { n_states + 1 }).unwrap()
    }
    #[inline]
    fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        f(self)
    }
}

impl<I: Input, S: Stack, O: Output> Ctrl<I, S, O> for BTreeSet<usize> {
    type View<'s> = iter::Copied<btree_set::Iter<'s, usize>>;
    #[inline]
    fn view(&self) -> Self::View<'_> {
        self.iter().copied()
    }
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::unwrap_used, unsafe_code)]
    #[cfg(feature = "quickcheck")]
    fn arbitrary_given(n_states: usize, g: &mut quickcheck::Gen, well_formed: bool) -> Self {
        use quickcheck::Arbitrary;
        let collection = Self::arbitrary(g);
        let nz = NonZeroUsize::new(if well_formed { n_states } else { n_states + 1 }).unwrap();
        collection.into_iter().map(|i| i % nz).collect()
    }
    #[inline]
    fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self {
        self.into_iter().map(f).collect()
    }
}

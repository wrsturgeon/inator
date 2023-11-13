/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Necessary preconditions to function as an index.

use crate::{Check, Input, Merge, ToSrc};
use core::iter;
use std::collections::{btree_set, BTreeSet};

#[cfg(feature = "quickcheck")]
use core::num::NonZeroUsize;

/// Necessary preconditions to function as an index.
pub trait Ctrl<I: Input>:
    Check<I, Self> + Clone + Merge<Error = (usize, usize)> + Ord + PartialEq + ToSrc
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
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut quickcheck::Gen) -> Self;
    /// Apply a function to each index.
    #[must_use]
    fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self;
    /// Turn a single index into its equivalent value in this type.
    #[must_use]
    fn from_usize(i: usize) -> Self;
}

impl<I: Input> Ctrl<I> for usize {
    type View<'s> = iter::Once<usize>;
    #[inline]
    fn view(&self) -> Self::View<'_> {
        iter::once(*self)
    }
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::unwrap_used, unsafe_code)]
    #[cfg(feature = "quickcheck")]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut quickcheck::Gen) -> Self {
        use quickcheck::Arbitrary;
        Self::arbitrary(g) % n_states
    }
    #[inline]
    fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        f(self)
    }
    #[inline(always)]
    fn from_usize(i: usize) -> Self {
        i
    }
}

impl<I: Input> Ctrl<I> for BTreeSet<usize> {
    type View<'s> = iter::Copied<btree_set::Iter<'s, usize>>;
    #[inline]
    fn view(&self) -> Self::View<'_> {
        self.iter().copied()
    }
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::unwrap_used, unsafe_code)]
    #[cfg(feature = "quickcheck")]
    fn arbitrary_given(n_states: NonZeroUsize, g: &mut quickcheck::Gen) -> Self {
        use quickcheck::Arbitrary;
        'restart: loop {
            let set = BTreeSet::<usize>::arbitrary(g);
            if set.is_empty() {
                continue 'restart;
            }
            return set.into_iter().map(|i| i % n_states).collect();
        }
    }
    #[inline]
    fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self {
        self.into_iter().map(f).collect()
    }
    #[inline]
    fn from_usize(i: usize) -> Self {
        iter::once(i).collect()
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from ranges of keys to values.

use crate::{Ctrl, Input, Range, Transition};
use core::slice::SliceIndex;

/// Map from ranges of keys to values.
#[repr(transparent)]
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RangeMap<I: Input, S, C: Ctrl> {
    /// Key-value entries as tuples.
    pub entries: Vec<(Range<I>, Transition<I, S, C>)>,
}

impl<I: Input, S, C: Ctrl> RangeMap<I, S, C> {
    /// Iterate over references to keys and values without consuming anything.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &(Range<I>, Transition<I, S, C>)> {
        self.entries.iter()
    }

    /// Get key-value pairs by index.
    #[inline]
    #[must_use]
    pub fn get<Index: SliceIndex<[(Range<I>, Transition<I, S, C>)]>>(
        &self,
        index: Index,
    ) -> Option<&Index::Output> {
        self.entries.get(index)
    }

    /// Get key-value pairs by index.
    /// # Safety
    /// Identical to `Vec::get_unchecked`:
    /// "Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used."
    #[inline]
    #[must_use]
    #[allow(unsafe_code, unsafe_op_in_unsafe_fn)]
    pub unsafe fn get_unchecked<Index: SliceIndex<[(Range<I>, Transition<I, S, C>)]>>(
        &self,
        index: Index,
    ) -> &Index::Output {
        self.entries.get_unchecked(index)
    }
}

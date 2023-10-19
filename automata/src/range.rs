/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Inclusive range of values that, as a whole, implements `Ord`.

use crate::Input;

/// Inclusive range of values that, as a whole, implements `Ord`.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Range<I: Input> {
    /// Least element, inclusive.
    pub first: I,
    /// Greatest element, inclusive.
    pub last: I,
}

impl<I: Input> Range<I> {
    /// Trivial range with a single inhabitant.
    #[inline]
    #[must_use]
    pub fn unit(value: I) -> Self {
        Self {
            first: value.clone(),
            last: value,
        }
    }

    /// Check if a value lies within this range.
    #[inline]
    #[must_use]
    pub fn contains(&self, value: &I) -> bool {
        *value >= self.first && *value <= self.last
    }

    /// If two ranges overlap, return their intersection.
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let first = self.first.clone().max(other.first.clone());
        let last = self.last.min(other.last);
        (first <= last).then_some(Self { first, last })
    }
}

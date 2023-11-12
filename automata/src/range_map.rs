/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from ranges of keys to values.

use crate::{Ctrl, IllFormed, Input, Range, Transitions};
use core::cmp;
use std::collections::BTreeMap;

/// Map from ranges of keys to values.
#[repr(transparent)]
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Default)]
pub struct RangeMap<I: Input, C: Ctrl<I>>(
    /// Key-value entries as tuples.
    #[allow(clippy::type_complexity)]
    pub BTreeMap<Range<I>, Transitions<I, C>>,
);

impl<I: Input, C: Ctrl<I>> Clone for RangeMap<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I: Input, C: Ctrl<I>> Eq for RangeMap<I, C> {}

impl<I: Input, C: Ctrl<I>> PartialEq for RangeMap<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<I: Input, C: Ctrl<I>> Ord for RangeMap<I, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<I: Input, C: Ctrl<I>> PartialOrd for RangeMap<I, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, C: Ctrl<I>> RangeMap<I, C> {
    /// Iterate over references to keys and values without consuming anything.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Range<I>, &Transitions<I, C>)> {
        self.0.iter()
    }

    /// Look up an argument; fit any range that contains it.
    /// # Errors
    /// If multiple ranges fit an argument.
    #[inline]
    #[allow(
        clippy::missing_panics_doc,
        clippy::unwrap_in_result,
        clippy::type_complexity
    )]
    pub fn get(&self, key: &I) -> Result<Option<&Transitions<I, C>>, IllFormed<I, C>> {
        let mut acc = None;
        for (range, transition) in &self.0 {
            if range.contains(key) {
                match acc {
                    None => acc = Some((range, transition)),
                    Some((prev_range, _)) => {
                        return Err(IllFormed::RangeMapOverlap(unwrap!(range
                            .clone()
                            .intersection(prev_range.clone()))))
                    }
                }
            }
        }
        Ok(acc.map(|(_, transition)| transition))
    }

    /// Assert that this map has no keys in common with another.
    /// # Errors
    /// If there are keys in common, don't panic: instead, return them.
    #[inline]
    #[allow(clippy::result_large_err, clippy::type_complexity)]
    pub fn disjoint(
        &self,
        other: &Self,
    ) -> Result<(), (Range<I>, Transitions<I, C>, Transitions<I, C>)> {
        self.0.iter().try_fold((), |(), (lk, lv)| {
            other.0.iter().try_fold((), |(), (rk, rv)| {
                rk.clone()
                    .intersection(lk.clone())
                    .map_or(Ok(()), |range| Err((range, lv.clone(), rv.clone())))
            })
        })
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Transitions<I, C>> {
        self.0.values()
    }

    /// Remove an entry by key.
    #[inline]
    pub fn remove(&mut self, key: &Range<I>) {
        self.0
            .retain(|k, _| key.clone().intersection(k.clone()).is_none());
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Transitions<I, C>> {
        self.0.values_mut()
    }
}

impl<I: Input> RangeMap<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> RangeMap<I, C> {
        RangeMap(
            self.0
                .into_iter()
                .map(|(k, v)| (k, v.convert_ctrl()))
                .collect(),
        )
    }
}

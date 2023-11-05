/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from ranges of keys to values.

use crate::{Ctrl, IllFormed, Input, Range, Stack, Transition};
use core::cmp;
use std::collections::BTreeMap;

/// Map from ranges of keys to values.
#[repr(transparent)]
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Default)]
pub struct RangeMap<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Key-value entries as tuples.
    #[allow(clippy::type_complexity)]
    pub entries: BTreeMap<Range<I>, Transition<I, S, C>>,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for RangeMap<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Eq for RangeMap<I, S, C> {}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialEq for RangeMap<I, S, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Ord for RangeMap<I, S, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.entries.cmp(&other.entries)
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialOrd for RangeMap<I, S, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> RangeMap<I, S, C> {
    /// Iterate over references to keys and values without consuming anything.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Range<I>, &Transition<I, S, C>)> {
        self.entries.iter()
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
    pub fn get(&self, key: &I) -> Result<Option<&Transition<I, S, C>>, IllFormed<I, S, C>> {
        let mut acc = None;
        for (range, transition) in &self.entries {
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
    ) -> Result<(), (Range<I>, Transition<I, S, C>, Transition<I, S, C>)> {
        self.entries.iter().try_fold((), |(), (lk, lv)| {
            other.entries.iter().try_fold((), |(), (rk, rv)| {
                rk.clone()
                    .intersection(lk.clone())
                    .map_or(Ok(()), |range| Err((range, lv.clone(), rv.clone())))
            })
        })
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Transition<I, S, C>> {
        self.entries.values()
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Transition<I, S, C>> {
        self.entries.values_mut()
    }

    /// Remove an entry by key.
    #[inline]
    pub fn remove(&mut self, key: &Range<I>) {
        self.entries
            .retain(|k, _| key.clone().intersection(k.clone()).is_none());
    }
}

impl<I: Input, S: Stack> RangeMap<I, S, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I, S>>(self) -> RangeMap<I, S, C> {
        RangeMap {
            entries: self
                .entries
                .into_iter()
                .map(|(k, v)| (k, v.convert_ctrl()))
                .collect(),
        }
    }
}

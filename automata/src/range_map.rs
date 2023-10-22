/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from ranges of keys to values.

use crate::{Ctrl, IllFormed, Input, Output, Range, Stack, Transition};

/// Map from ranges of keys to values.
#[repr(transparent)]
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct RangeMap<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Key-value entries as tuples.
    #[allow(clippy::type_complexity)]
    pub entries: Vec<(Range<I>, Transition<I, S, O, C>)>,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Clone for RangeMap<I, S, O, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> RangeMap<I, S, O, C> {
    /// Iterate over references to keys and values without consuming anything.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &(Range<I>, Transition<I, S, O, C>)> {
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
    pub fn get(&self, key: &I) -> Result<Option<&Transition<I, S, O, C>>, IllFormed<I, S, O, C>> {
        let mut acc = None;
        for &(ref range, ref transition) in &self.entries {
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
    #[allow(clippy::type_complexity)]
    pub fn disjoint(
        &self,
        other: &Self,
    ) -> Result<(), (Range<I>, Transition<I, S, O, C>, Transition<I, S, O, C>)> {
        self.entries.iter().try_fold((), |(), &(ref lk, ref lv)| {
            other.entries.iter().try_fold((), |(), &(ref rk, ref rv)| {
                rk.clone().intersection(lk.clone()).map_or(Ok(()), |range| {
                    if lv == rv {
                        Ok(())
                    } else {
                        Err((range, lv.clone(), rv.clone()))
                    }
                })
            })
        })
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Transition<I, S, O, C>> {
        self.entries.iter().map(|&(_, ref v)| v)
    }
}

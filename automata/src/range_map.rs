/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Map from ranges of keys to values.

use crate::{Ctrl, IllFormed, Input, Output, Range, Stack, Transition};
use core::cmp;
use std::collections::BTreeSet;

/// Compare only the first element for `PartialOrd` & `Ord`;
/// compare both for `PartialEq` & `Eq`.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CmpFirst<K: Ord, V: Eq>(pub K, pub V);

impl<K: Ord, V: Eq> PartialOrd for CmpFirst<K, V> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Eq> Ord for CmpFirst<K, V> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// Map from ranges of keys to values.
#[repr(transparent)]
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Default)]
pub struct RangeMap<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Key-value entries as tuples.
    #[allow(clippy::type_complexity)]
    pub entries: BTreeSet<CmpFirst<Range<I>, Transition<I, S, O, C>>>,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Clone for RangeMap<I, S, O, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Eq for RangeMap<I, S, O, C> {}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialEq for RangeMap<I, S, O, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Ord for RangeMap<I, S, O, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.entries.cmp(&other.entries)
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialOrd for RangeMap<I, S, O, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> RangeMap<I, S, O, C> {
    /// Iterate over references to keys and values without consuming anything.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &CmpFirst<Range<I>, Transition<I, S, O, C>>> {
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
        for &CmpFirst(ref range, ref transition) in &self.entries {
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
        self.entries
            .iter()
            .try_fold((), |(), &CmpFirst(ref lk, ref lv)| {
                other
                    .entries
                    .iter()
                    .try_fold((), |(), &CmpFirst(ref rk, ref rv)| {
                        rk.clone()
                            .intersection(lk.clone())
                            .map_or(Ok(()), |range| Err((range, lv.clone(), rv.clone())))
                    })
            })
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Transition<I, S, O, C>> {
        self.entries.iter().map(|&CmpFirst(_, ref v)| v)
    }

    /// Remove an entry by key.
    #[inline]
    pub fn remove(&mut self, key: &Range<I>) {
        self.entries
            .retain(|&CmpFirst(ref k, _)| key.clone().intersection(k.clone()).is_none());
    }

    /// Chop off parts of the automaton until it's valid.
    #[inline]
    #[must_use]
    pub fn procrustes(mut self) -> Self {
        while let Some(overlap) = self
            .entries
            .iter()
            .fold(None, |acc, &CmpFirst(ref k, ref v)| {
                acc.or_else(|| {
                    self.entries.range(..CmpFirst(k.clone(), v.clone())).fold(
                        None,
                        |opt, &CmpFirst(ref range, _)| {
                            opt.or_else(|| range.clone().intersection(k.clone()))
                        },
                    )
                })
            })
        {
            self.entries
                .retain(|&CmpFirst(ref k, _)| k.clone().intersection(overlap.clone()).is_none());
        }
        self.entries
            .retain(|&CmpFirst(_, ref v)| v.dst.view().next().is_some());
        self
    }
}

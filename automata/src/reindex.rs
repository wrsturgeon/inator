/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Update index "pointers" in response to a reordered array.

use crate::*;
use std::collections::BTreeMap;

impl<I: Input, C: Ctrl<I>> State<I, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, C>],
        index_map: &BTreeMap<usize, State<I, C>>,
    ) -> Self {
        State {
            transitions: self.transitions.reindex(states, index_map),
            non_accepting: self.non_accepting.clone(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Curry<I, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, C>],
        index_map: &BTreeMap<usize, State<I, C>>,
    ) -> Self {
        match *self {
            Curry::Wildcard(ref etc) => Curry::Wildcard(etc.reindex(states, index_map)),
            Curry::Scrutinize {
                ref filter,
                ref fallback,
            } => Curry::Scrutinize {
                filter: filter.reindex(states, index_map),
                fallback: fallback.as_ref().map(|f| f.reindex(states, index_map)),
            },
        }
    }
}

impl<I: Input, C: Ctrl<I>> RangeMap<I, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, C>],
        index_map: &BTreeMap<usize, State<I, C>>,
    ) -> Self {
        RangeMap(
            self.0
                .iter()
                .map(|(k, v)| (k.clone(), v.reindex(states, index_map)))
                .collect(),
        )
    }
}

impl<I: Input, C: Ctrl<I>> Transition<I, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn reindex(
        &self,
        states: &[State<I, C>],
        index_map: &BTreeMap<usize, State<I, C>>,
    ) -> Self {
        let update_fn = |i| unwrap!(states.binary_search(unwrap!(index_map.get(&i))));
        match *self {
            Self::Lateral {
                ref dst,
                ref update,
            } => Self::Lateral {
                dst: dst.clone().map_indices(update_fn),
                update: update.clone(),
            },
            Self::Call {
                region,
                ref detour,
                ref dst,
                ref combine,
            } => Self::Call {
                region,
                detour: detour.clone().map_indices(update_fn),
                dst: Box::new(dst.clone().map_indices(update_fn)),
                combine: combine.clone(),
            },
            Self::Return { region } => Self::Return { region },
        }
    }
}

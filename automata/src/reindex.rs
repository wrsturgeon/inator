/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Update index "pointers" in response to a reordered array.

use crate::*;
use std::collections::{BTreeMap, BTreeSet};

impl<I: Input, S: Stack, C: Ctrl<I, S>> State<I, S, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, S, C>],
        index_map: &BTreeMap<usize, State<I, S, C>>,
    ) -> State<I, S, BTreeSet<Result<usize, String>>> {
        State {
            transitions: self.transitions.reindex(states, index_map),
            non_accepting: self.non_accepting.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryStack<I, S, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, S, C>],
        index_map: &BTreeMap<usize, State<I, S, C>>,
    ) -> CurryStack<I, S, BTreeSet<Result<usize, String>>> {
        CurryStack {
            wildcard: self.wildcard.as_ref().map(|w| w.reindex(states, index_map)),
            map_none: self.map_none.as_ref().map(|m| m.reindex(states, index_map)),
            map_some: self
                .map_some
                .iter()
                .map(|(k, v)| (k.clone(), v.reindex(states, index_map)))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryInput<I, S, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, S, C>],
        index_map: &BTreeMap<usize, State<I, S, C>>,
    ) -> CurryInput<I, S, BTreeSet<Result<usize, String>>> {
        match *self {
            CurryInput::Wildcard(ref etc) => CurryInput::Wildcard(etc.reindex(states, index_map)),
            CurryInput::Scrutinize(ref etc) => {
                CurryInput::Scrutinize(etc.reindex(states, index_map))
            }
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> RangeMap<I, S, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    pub fn reindex(
        &self,
        states: &[State<I, S, C>],
        index_map: &BTreeMap<usize, State<I, S, C>>,
    ) -> RangeMap<I, S, BTreeSet<Result<usize, String>>> {
        RangeMap {
            entries: self
                .entries
                .iter()
                .map(|(k, v)| (k.clone(), v.reindex(states, index_map)))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Transition<I, S, C> {
    /// Update index "pointers" in response to a reordered array.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc, unused_unsafe)]
    pub fn reindex(
        &self,
        states: &[State<I, S, C>],
        index_map: &BTreeMap<usize, State<I, S, C>>,
    ) -> Transition<I, S, BTreeSet<Result<usize, String>>> {
        Transition {
            dst: self
                .dst
                .view()
                .map(|r| {
                    r.map_err(str::to_owned)
                        .map(|i| unwrap!(states.binary_search(unwrap!(index_map.get(&i)))))
                })
                .collect(),
            act: self.act.clone(),
            update: self.update.clone(),
        }
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Apply a function to each index in a structure.

use crate::{Ctrl, Curry, Graph, Input, RangeMap, State, Transition};

impl<I: Input, C: Ctrl<I>> Graph<I, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        Self {
            states: self
                .states
                .into_iter()
                .map(|s| s.map_indices(&mut f))
                .collect(),
            initial: self.initial.map_indices(&mut f),
        }
    }
}

impl<I: Input, C: Ctrl<I>> State<I, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self {
        Self {
            transitions: self.transitions.map_indices(f),
            ..self
        }
    }
}

impl<I: Input, C: Ctrl<I>> Curry<I, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        match self {
            Self::Wildcard(etc) => Self::Wildcard(etc.map_indices(f)),
            Self::Scrutinize { filter, fallback } => Self::Scrutinize {
                fallback: fallback.map(|t| t.map_indices(&mut f)),
                filter: filter.map_indices(f),
            },
        }
    }
}

impl<I: Input, C: Ctrl<I>> RangeMap<I, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        Self(
            self.0
                .into_iter()
                .map(|(k, v)| (k, v.map_indices(&mut f)))
                .collect(),
        )
    }
}

impl<I: Input, C: Ctrl<I>> Transition<I, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        match self {
            Self::Lateral { dst, update } => Self::Lateral {
                dst: dst.map_indices(f),
                update,
            },
            Self::Call {
                region,
                detour,
                dst,
                combine,
            } => Self::Call {
                region,
                detour: detour.map_indices(&mut f),
                dst: Box::new(dst.map_indices(f)),
                combine,
            },
            Self::Return { region } => Self::Return { region },
        }
    }
}

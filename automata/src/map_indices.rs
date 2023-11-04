/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Apply a function to each index in a structure.

use crate::{Ctrl, CurryInput, CurryStack, Graph, Input, RangeMap, Stack, State, Transition};

impl<I: Input, S: Stack, C: Ctrl<I, S>> Graph<I, S, C> {
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
            tags: self.tags.into_iter().map(|(k, v)| (k, f(v))).collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> State<I, S, C> {
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

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryStack<I, S, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        Self {
            wildcard: self.wildcard.map(|c| c.map_indices(&mut f)),
            map_none: self.map_none.map(|c| c.map_indices(&mut f)),
            map_some: self
                .map_some
                .into_iter()
                .map(|(k, v)| (k, v.map_indices(&mut f)))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryInput<I, S, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self {
        match self {
            Self::Wildcard(etc) => Self::Wildcard(etc.map_indices(f)),
            Self::Scrutinize(etc) => Self::Scrutinize(etc.map_indices(f)),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> RangeMap<I, S, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        Self {
            entries: self
                .entries
                .into_iter()
                .map(|(k, v)| (k, v.map_indices(&mut f)))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Transition<I, S, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, f: F) -> Self {
        Self {
            dst: self.dst.map_indices(f),
            ..self
        }
    }
}

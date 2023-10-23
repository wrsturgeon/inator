/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Apply a function to each index in a structure.

use crate::{
    CmpFirst, Ctrl, CurryInput, CurryStack, Graph, Input, Output, RangeMap, Stack, State,
    Transition,
};

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Graph<I, S, O, C> {
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
            initial: self.initial.map_indices(f),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> State<I, S, O, C> {
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

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryStack<I, S, O, C> {
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

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryInput<I, S, O, C> {
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

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> RangeMap<I, S, O, C> {
    /// Apply a function to each index.
    #[inline]
    #[must_use]
    pub fn map_indices<F: FnMut(usize) -> usize>(self, mut f: F) -> Self {
        Self {
            entries: self
                .entries
                .into_iter()
                .map(|CmpFirst(k, v)| CmpFirst(k, v.map_indices(&mut f)))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Transition<I, S, O, C> {
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

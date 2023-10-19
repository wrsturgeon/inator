/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the next input symbol and decide an action.

use crate::{Ctrl, IllFormed, Input, Output, RangeMap, Stack, Transition};

/// Read the next input symbol and decide an action.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurryInput<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Throw away the input (without looking at it) and do this.
    Wildcard(Transition<I, S, O, C>),
    /// Map specific ranges of inputs to actions.
    Scrutinize(RangeMap<I, S, O, C>),
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryInput<I, S, O, C> {
    /// Look up a transition based on an input token.
    /// # Errors
    /// If multiple ranges fit an argument.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn get(&self, key: &I) -> Result<Option<&Transition<I, S, O, C>>, IllFormed<I, S, O, C>> {
        match *self {
            Self::Wildcard(ref transition) => Ok(Some(transition)),
            Self::Scrutinize(ref range_map) => range_map.get(key),
        }
    }
}

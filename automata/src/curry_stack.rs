/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the symbol at the top of the stack (if any), then
//! return another function that reads input and decides an action.

use crate::{Ctrl, CurryInput, IllFormed, Input, Merge, Output, Stack, Transition};
use std::collections::BTreeMap;

/// Read the symbol at the top of the stack (if any), then
/// return another function that reads input and decides an action.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurryStack<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// No matter what the stack says, try this first.
    pub wildcard: Option<CurryInput<I, S, O, C>>,
    /// If input ends (i.e. an iterator yields `None`), try this.
    pub map_none: Option<CurryInput<I, S, O, C>>,
    /// If input does not end (i.e. an iterator yields `Some(..)`), try this.
    pub map_some: BTreeMap<S, CurryInput<I, S, O, C>>,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> CurryStack<I, S, O, C> {
    /// Look up a transition based on the current stack and an input token.
    /// # Errors
    /// If these keys could match multiple ranges of inputs.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn get(
        &self,
        stack: Option<&S>,
        token: &I,
    ) -> Result<Option<Transition<I, S, O, C>>, IllFormed<I, S, O, C>> {
        self.wildcard
            .as_ref()
            .map_or(Ok(None), |wc| wc.get(token))
            .and_then(|wildcard| {
                let non_wildcard = stack.map_or_else(
                    || {
                        self.map_none
                            .as_ref()
                            .map_or(Ok(None), |none| none.get(token))
                    },
                    |symbol| {
                        self.map_some
                            .get(symbol)
                            .map_or(Ok(None), |curry| curry.get(token))
                    },
                )?;
                wildcard
                    .cloned()
                    .merge(non_wildcard.cloned())
                    .map_err(|(a, b)| IllFormed::WildcardMask(a, b))
            })
    }
}
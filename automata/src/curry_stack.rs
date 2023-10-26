/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the symbol at the top of the stack (if any), then
//! return another function that reads input and decides an action.

use crate::{Ctrl, CurryInput, IllFormed, Input, Merge, Range, Stack, Transition};
use core::cmp;
use std::collections::BTreeMap;

/// Read the symbol at the top of the stack (if any), then
/// return another function that reads input and decides an action.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct CurryStack<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// No matter what the stack says, try this first.
    pub wildcard: Option<CurryInput<I, S, C>>,
    /// If input ends (i.e. an iterator yields `None`), try this.
    pub map_none: Option<CurryInput<I, S, C>>,
    /// If input does not end (i.e. an iterator yields `Some(..)`), try this.
    pub map_some: BTreeMap<S, CurryInput<I, S, C>>,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for CurryStack<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            wildcard: self.wildcard.clone(),
            map_none: self.map_none.clone(),
            map_some: self.map_some.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Eq for CurryStack<I, S, C> {}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialEq for CurryStack<I, S, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.wildcard == other.wildcard
            && self.map_none == other.map_none
            && self.map_some == other.map_some
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Ord for CurryStack<I, S, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.wildcard
            .cmp(&other.wildcard)
            .then_with(|| self.map_none.cmp(&other.map_none))
            .then_with(|| self.map_some.cmp(&other.map_some))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialOrd for CurryStack<I, S, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryStack<I, S, C> {
    /// Look up a transition based on the current stack and an input token.
    /// # Errors
    /// If these keys could match multiple ranges of inputs.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn get(
        &self,
        stack: Option<&S>,
        token: &I,
    ) -> Result<Option<Transition<I, S, C>>, IllFormed<I, S, C>> {
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
                wildcard.cloned().merge(non_wildcard.cloned()).map_err(
                    |(possibility_1, possibility_2)| IllFormed::WildcardMask {
                        arg_stack: stack.cloned(),
                        arg_token: Some(Range::unit(token.clone())),
                        possibility_1,
                        possibility_2,
                    },
                )
            })
    }

    /// All values in this collection, without their associated keys.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &CurryInput<I, S, C>> {
        self.wildcard
            .iter()
            .chain(self.map_none.iter())
            .chain(self.map_some.values())
    }
}

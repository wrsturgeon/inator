/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, CurryStack, IllFormed, Input, Stack};
use core::cmp;
use std::collections::BTreeSet;

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct State<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Map from input tokens to actions.
    pub transitions: CurryStack<I, S, C>,
    /// If input ends while in this state, should we accept?
    // TODO: use a `BTreeSet`.
    pub non_accepting: BTreeSet<String>,
    /// Optional name for this state.
    pub tags: BTreeSet<String>,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> State<I, S, C> {
    /// Compute the input type of any run that reaches this state.
    /// # Errors
    /// If multiple transitions expect different types.
    #[inline]
    pub fn input_type(&self) -> Result<Option<String>, IllFormed<I, S, C>> {
        self.transitions
            .values()
            .flat_map(|c| c.values())
            .try_fold(None, |acc, t| {
                acc.map_or_else(
                    || Ok(Some(t.update.input_t.clone())),
                    |other| {
                        if t.update.input_t == other {
                            Ok(Some(other))
                        } else {
                            Err(IllFormed::TypeMismatch(other, t.update.input_t.clone()))
                        }
                    },
                )
            })
    }
}

impl<I: Input, S: Stack> State<I, S, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I, S>>(self) -> State<I, S, C> {
        State {
            transitions: self.transitions.convert_ctrl(),
            non_accepting: self.non_accepting,
            tags: self.tags,
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for State<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
            non_accepting: self.non_accepting.clone(),
            tags: self.tags.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Eq for State<I, S, C> {}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialEq for State<I, S, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.non_accepting == other.non_accepting && self.transitions == other.transitions
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Ord for State<I, S, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.transitions
            .cmp(&other.transitions)
            .then_with(|| self.non_accepting.cmp(&other.non_accepting))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialOrd for State<I, S, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, Curry, IllFormed, Input};
use core::cmp;
use std::collections::BTreeSet;

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct State<I: Input, C: Ctrl<I>> {
    /// Map from input tokens to actions.
    pub transitions: Curry<I, C>,
    /// If input ends while in this state, should we accept?
    pub non_accepting: BTreeSet<String>,
}

impl<I: Input, C: Ctrl<I>> State<I, C> {
    /// Compute the input type of any run that reaches this state.
    /// # Errors
    /// If multiple transitions expect different types.
    #[inline]
    pub fn input_type(&self) -> Result<Option<&str>, IllFormed<I, C>> {
        self.transitions
            .values()
            .try_fold(None, |acc: Option<&str>, t| {
                let in_t = t.input_type();
                acc.map_or(Ok(in_t), |other| match in_t {
                    None => Ok(Some(other)),
                    Some(ty) if ty == other => Ok(Some(other)),
                    Some(ty) => Err(IllFormed::TypeMismatch(other.to_owned(), ty.to_owned())),
                })
            })
    }

    /// Check if this parser ever could, at any point, involve a fallback transition.
    #[inline]
    #[must_use]
    pub const fn involves_any_fallback(&self) -> bool {
        self.transitions.involves_any_fallback()
    }
}

impl<I: Input> State<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> State<I, C> {
        State {
            transitions: self.transitions.convert_ctrl(),
            non_accepting: self.non_accepting,
        }
    }
}

impl<I: Input, C: Ctrl<I>> Clone for State<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
            non_accepting: self.non_accepting.clone(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Eq for State<I, C> {}

impl<I: Input, C: Ctrl<I>> PartialEq for State<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.transitions == other.transitions && self.non_accepting == other.non_accepting
    }
}

impl<I: Input, C: Ctrl<I>> Ord for State<I, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.transitions
            .cmp(&other.transitions)
            .then_with(|| self.non_accepting.cmp(&other.non_accepting))
    }
}

impl<I: Input, C: Ctrl<I>> PartialOrd for State<I, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

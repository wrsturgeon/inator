/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, CurryStack, Input, Output, Stack};
use core::cmp;

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct State<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Map from input tokens to actions.
    pub transitions: CurryStack<I, S, O, C>,
    /// If input ends while in this state, should we accept?
    pub accepting: bool,
    /// Optional name for this state.
    pub tag: Option<String>,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Clone for State<I, S, O, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
            accepting: self.accepting,
            tag: None,
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Eq for State<I, S, O, C> {}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialEq for State<I, S, O, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.accepting == other.accepting && self.transitions == other.transitions
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Ord for State<I, S, O, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.transitions
            .cmp(&other.transitions)
            .then_with(|| self.accepting.cmp(&other.accepting))
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialOrd for State<I, S, O, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

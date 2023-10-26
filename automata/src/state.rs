/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, CurryStack, Input, Stack};
use core::cmp;
use std::collections::BTreeSet;

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct State<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Map from input tokens to actions.
    pub transitions: CurryStack<I, S, C>,
    /// If input ends while in this state, should we accept?
    pub accepting: bool,
    /// Optional name for this state.
    pub tag: BTreeSet<String>,
    /// Type of any run that reaches this state.
    pub input_t: String,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for State<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
            accepting: self.accepting,
            tag: self.tag.clone(),
            input_t: self.input_t.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Eq for State<I, S, C> {}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialEq for State<I, S, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.accepting == other.accepting
            && self.transitions == other.transitions
            && self.tag == other.tag
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Ord for State<I, S, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.transitions
            .cmp(&other.transitions)
            .then_with(|| self.accepting.cmp(&other.accepting))
            .then_with(|| self.tag.cmp(&other.tag))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialOrd for State<I, S, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

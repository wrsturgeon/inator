/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, CurryStack, Input, Output, Stack};

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Eq, PartialEq)]
pub struct State<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Map from input tokens to actions.
    pub transitions: CurryStack<I, S, O, C>,
    /// If input ends while in this state, should we accept?
    pub accepting: bool,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Clone for State<I, S, O, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            transitions: self.transitions.clone(),
            accepting: self.accepting,
        }
    }
}

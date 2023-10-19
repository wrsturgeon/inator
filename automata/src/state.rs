/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! State, i.e. a node in an automaton graph.

use crate::{Ctrl, CurryStack, Input};

/// State, i.e. a node in an automaton graph.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State<I: Input, S, C: Ctrl> {
    /// Map from input tokens to actions.
    pub transitions: CurryStack<I, S, C>,
    /// If input ends while in this state, should we accept?
    pub accepting: bool,
}

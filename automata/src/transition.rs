/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Edge in an automaton graph: an action and a destination state.

use crate::{Action, Ctrl, Input};

/// Edge in an automaton graph: an action and a destination state.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Transition<I: Input, S, C: Ctrl> {
    /// Go to this state.
    pub dst: C,
    /// Take this action: maybe push/pop from the stack.
    pub act: Action<I, S>,
}

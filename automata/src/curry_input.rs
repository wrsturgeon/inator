/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the next input symbol and decide an action.

use crate::{Ctrl, Input, RangeMap, Transition};

/// Read the next input symbol and decide an action.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurryInput<I: Input, S, C: Ctrl> {
    /// Throw away the input (without looking at it) and do this.
    Wildcard(Transition<I, S, C>),
    /// Map specific ranges of inputs to actions.
    Scrutinize(RangeMap<I, S, C>),
}

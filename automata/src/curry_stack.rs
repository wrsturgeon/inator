/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Read the symbol at the top of the stack (if any), then
//! return another function that reads input and decides an action.

use crate::{Ctrl, CurryInput, Input};
use std::collections::BTreeMap;

/// Read the symbol at the top of the stack (if any), then
/// return another function that reads input and decides an action.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurryStack<I: Input, S, C: Ctrl> {
    /// No matter what the stack says, try this first.
    wildcard: Option<CurryInput<I, S, C>>,
    /// If input ends (i.e. an iterator yields `None`), try this.
    map_none: Option<CurryInput<I, S, C>>,
    /// If input does not end (i.e. an iterator yields `Some(..)`), try this.
    map_some: BTreeMap<S, CurryInput<I, S, C>>,
}

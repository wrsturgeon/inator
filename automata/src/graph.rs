/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automaton loosely based on visibly pushdown automata.

use crate::{Ctrl, Input, State};
use std::collections::BTreeSet;

/// One token corresponds to at most one transition.
pub type Deterministic<I, S> = Graph<I, S, usize>;

/// One token corresponds to as many transitions as it would like;
/// if any of these transitions eventually accept, the whole thing accepts.
pub type Nondeterministic<I, S> = Graph<I, S, BTreeSet<usize>>;

/// Automaton loosely based on visibly pushdown automata.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Graph<I: Input, S, C: Ctrl> {
    /// Every state, indexed.
    pub states: Vec<State<I, S, C>>,
    /// Initial state of the machine (before reading input).
    pub initial: C,
}

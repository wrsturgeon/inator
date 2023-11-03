/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.

use std::collections::BTreeSet;

use crate::{Ctrl, Graph, Input, Nondeterministic, Stack, State};

impl<I: Input, S: Stack, C: Ctrl<I, S>> Graph<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Nondeterministic<I, S> {
        Nondeterministic {
            states: self.states.into_iter().map(|s| s.generalize()).collect(),
            initial: self
                .initial
                .view()
                .map(|r| r.map_err(str::to_owned))
                .collect(),
            tags: self.tags,
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> State<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> State<I, S, BTreeSet<Result<usize, String>>> {
        State {
            transitions: todo!(),
            non_accepting: self.non_accepting,
        }
    }
}

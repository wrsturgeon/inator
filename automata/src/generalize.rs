/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.

use std::collections::BTreeSet;

use crate::{
    Ctrl, CurryInput, CurryStack, Graph, Input, Nondeterministic, RangeMap, Stack, State,
    Transition,
};

impl<I: Input, S: Stack, C: Ctrl<I, S>> Graph<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Nondeterministic<I, S> {
        Nondeterministic {
            states: self.states.into_iter().map(State::generalize).collect(),
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
            transitions: self.transitions.generalize(),
            non_accepting: self.non_accepting,
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryStack<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> CurryStack<I, S, BTreeSet<Result<usize, String>>> {
        CurryStack {
            wildcard: self.wildcard.map(CurryInput::generalize),
            map_none: self.map_none.map(CurryInput::generalize),
            map_some: self
                .map_some
                .into_iter()
                .map(|(k, v)| (k, v.generalize()))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> CurryInput<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> CurryInput<I, S, BTreeSet<Result<usize, String>>> {
        match self {
            Self::Wildcard(w) => CurryInput::Wildcard(w.generalize()),
            Self::Scrutinize(s) => CurryInput::Scrutinize(s.generalize()),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> RangeMap<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    #[must_use]
    pub fn generalize(self) -> RangeMap<I, S, BTreeSet<Result<usize, String>>> {
        RangeMap {
            entries: self
                .entries
                .into_iter()
                .map(|(k, v)| (k, v.generalize()))
                .collect(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Transition<I, S, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Transition<I, S, BTreeSet<Result<usize, String>>> {
        Transition {
            dst: self.dst.view().map(|r| r.map_err(str::to_owned)).collect(),
            act: self.act,
            update: self.update,
        }
    }
}

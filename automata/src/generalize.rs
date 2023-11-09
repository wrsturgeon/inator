/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.

use crate::{Ctrl, Curry, Graph, Input, Nondeterministic, RangeMap, State, Transition};
use std::collections::BTreeSet;

impl<I: Input, C: Ctrl<I>> Graph<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Nondeterministic<I> {
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

impl<I: Input, C: Ctrl<I>> State<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> State<I, BTreeSet<Result<usize, String>>> {
        State {
            transitions: self.transitions.generalize(),
            non_accepting: self.non_accepting,
        }
    }
}

impl<I: Input, C: Ctrl<I>> Curry<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Curry<I, BTreeSet<Result<usize, String>>> {
        match self {
            Self::Wildcard(w) => Curry::Wildcard(w.generalize()),
            Self::Scrutinize(s) => Curry::Scrutinize(s.generalize()),
        }
    }
}

impl<I: Input, C: Ctrl<I>> RangeMap<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    #[must_use]
    pub fn generalize(self) -> RangeMap<I, BTreeSet<Result<usize, String>>> {
        RangeMap(
            self.0
                .into_iter()
                .map(|(k, v)| (k, v.generalize()))
                .collect(),
        )
    }
}

impl<I: Input, C: Ctrl<I>> Transition<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Transition<I, BTreeSet<Result<usize, String>>> {
        match self {
            Self::Lateral { dst, update } => Transition::Lateral {
                dst: dst.view().map(|r| r.map_err(str::to_owned)).collect(),
                update,
            },
            Self::Call {
                region,
                detour,
                dst,
                combine,
            } => Transition::Call {
                region,
                detour: detour.view().map(|r| r.map_err(str::to_owned)).collect(),
                dst: dst.view().map(|r| r.map_err(str::to_owned)).collect(),
                combine,
            },
            Self::Return { region } => Transition::Return { region },
        }
    }
}

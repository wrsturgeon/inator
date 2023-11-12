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
            initial: self.initial.view().collect(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> State<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> State<I, BTreeSet<usize>> {
        State {
            transitions: self.transitions.generalize(),
            non_accepting: self.non_accepting,
            fallback: self.fallback.map(Transition::generalize),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Curry<I, C> {
    /// Un-determinize an automaton to return a practically identical (but nominally nondeterministic) version.
    #[inline]
    pub fn generalize(self) -> Curry<I, BTreeSet<usize>> {
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
    pub fn generalize(self) -> RangeMap<I, BTreeSet<usize>> {
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
    pub fn generalize(self) -> Transition<I, BTreeSet<usize>> {
        match self {
            Self::Lateral { dst, update } => Transition::Lateral {
                dst: dst.view().collect(),
                update,
            },
            Self::Call {
                region,
                detour,
                dst,
                combine,
            } => Transition::Call {
                region,
                detour: detour.view().collect(),
                dst: dst.view().collect(),
                combine,
            },
            Self::Return { region } => Transition::Return { region },
        }
    }
}

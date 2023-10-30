/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on nondeterministic finite automata returning nondeterministic finite automata.

use crate::{Input, Nondeterministic, Stack};
use core::ops;

impl<I: Input, S: Stack> ops::BitOr for Nondeterministic<I, S> {
    type Output = Self;
    #[inline]
    fn bitor(mut self, other: Self) -> Self {
        // Note that union on pushdown automata is undecidable;
        // we just reject a subset of automata that wouldn't work.
        if self.check().is_err() {
            return self;
        }
        let size = self.states.len();
        let Self {
            states: other_states,
            initial: other_initial,
        } = other.map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));
        self.states.extend(other_states);
        self.initial.extend(other_initial);
        while self.check_sorted().is_err() {
            self = self.sort();
        }
        self
    }
}

impl<I: Input, S: Stack> ops::Shr for Nondeterministic<I, S> {
    type Output = Self;
    #[inline]
    fn shr(self, _: Self) -> Self::Output {
        self
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Fixpoint: just a tagged state that can be called later.

use core::ops;
use inator_automata::*;

/// Tagged state that can be called later.
#[must_use = "Fixpoints do nothing unless applied to an automaton with the `>>` operator."]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Fixpoint(String);

impl<I: Input, S: Stack> ops::Shr<Deterministic<I, S>> for Fixpoint {
    type Output = Deterministic<I, S>;
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    fn shr(self, mut rhs: Deterministic<I, S>) -> Self::Output {
        if rhs.tags.insert(self.0, rhs.initial).is_some() {
            panic!("Fixpoint name already in use");
        }
        // rhs.sort();
        rhs
    }
}

/// Name a point in code so we can call it later by name.
#[inline]
pub fn fixpoint(call_by_name: &str) -> Fixpoint {
    Fixpoint(call_by_name.to_owned())
}

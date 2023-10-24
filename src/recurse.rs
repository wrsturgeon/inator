/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Call a tagged state.

use core::ops;
use inator_automata::*;

/// Call a tagged state.
#[must_use = "Recurse statements do nothing unless they're used on an automaton with the `>>` operator."]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Recurse(String);

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> ops::Shr<Recurse> for Graph<I, S, O, C> {
    type Output = Graph<I, S, O, C>;
    #[inline]
    #[allow(clippy::todo, unused_variables)] // <-- FIXME
    fn shr(self, rhs: Recurse) -> Self::Output {
        todo!()
    }
}

/// Call a point in code by name.
#[inline]
pub fn recurse(call_by_name: &str) -> Recurse {
    Recurse(call_by_name.to_owned())
}

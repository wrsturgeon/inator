/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Fixpoint: just a tagged state that can be called later.

use core::ops;
use inator_automata::*;

/// Tagged state that can be called later.
#[must_use = "Fixpoints do nothing unless they're used on an automaton with the `>>` operator."]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Fixpoint(String);

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> ops::Shr<Graph<I, S, O, C>> for Fixpoint {
    type Output = Graph<I, S, O, C>;
    #[inline]
    #[allow(clippy::panic)]
    fn shr(self, rhs: Graph<I, S, O, C>) -> Self::Output {
        let Graph {
            mut states,
            initial,
        } = rhs;
        for r in initial.view() {
            match r {
                Ok(i) => get_mut!(states, i),
                Err(tag) => find_tag_mut(&mut states, tag).map_or_else(
                    |_| {
                        panic!(
                            "Weird error: \
                            an earlier parser called a fixpoint by name, \
                            but that name was nowhere to be found.",
                        )
                    },
                    |s| s,
                ),
            }
            .tag
            .push(self.0.clone());
        }
        Graph { states, initial }
    }
}

/// Name a point in code so we can call it later by name.
#[inline]
pub fn fixpoint(call_by_name: &str) -> Fixpoint {
    Fixpoint(call_by_name.to_owned())
}

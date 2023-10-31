/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Fixpoint: just a tagged state that can be called later.

use core::{iter, ops};
use inator_automata::*;

/// Tagged state that can be called later.
#[must_use = "Fixpoints do nothing unless applied to an automaton with the `>>` operator."]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Fixpoint(String);

impl<I: Input, S: Stack, C: Ctrl<I, S>> ops::Shr<Graph<I, S, C>> for Fixpoint {
    type Output = Nondeterministic<I, S>;
    #[inline]
    #[allow(clippy::manual_assert, clippy::panic)]
    fn shr(self, rhs: Graph<I, S, C>) -> Self::Output {
        let Graph {
            mut states,
            initial,
        } = rhs;
        for r in initial.view() {
            for state in match r {
                Ok(i) => iter::once(get_mut!(states, i)).collect(),
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
            } {
                let _ = state.tags.insert(self.0.clone());
            }
        }
        let out = Graph { states, initial }.sort();
        if out.check_sorted().is_err() {
            panic!("Sorting error: {}", out.to_src());
        }
        out
    }
}

/// Name a point in code so we can call it later by name.
#[inline]
pub fn fixpoint(call_by_name: &str) -> Fixpoint {
    Fixpoint(call_by_name.to_owned())
}

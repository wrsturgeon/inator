/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Save the current value, run this second parser from scratch, then combine the results.

use core::{iter, ops};
use inator_automata::*;
use std::collections::BTreeSet;

/// Save the current value, run this second parser from scratch, then combine the results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Call<I: Input> {
    /// Parser to call.
    parser: Deterministic<I>,
    /// Combine the tabled result with the result of the call.
    combinator: FF,
}

/// Save the current value, run this second parser from scratch, then combine the results.
#[inline]
#[must_use]
pub const fn call<I: Input>(parser: Deterministic<I>, combinator: FF) -> Call<I> {
    Call { parser, combinator }
}

impl<I: Input, C: Ctrl<I>> ops::Shr<Call<I>> for Graph<I, C> {
    type Output = Deterministic<I>;
    #[inline]
    #[must_use]
    #[allow(clippy::panic)]
    fn shr(self, c: Call<I>) -> Self::Output {
        let Call { parser, combinator } = c;
        match parser.input_type() {
            Err(e) => panic!("Inconsistent types in the parser argument to `combine`: {e}."),
            Ok(None) => panic!(
                "Parser argument to `combine` has no initial states, \
                so it can never parse anything.",
            ),
            Ok(Some("()")) => {}
            Ok(Some(non_unit)) => panic!(
                "Called `call` with a parser that doesn't start from scratch \
                (it wants an input of type `{non_unit}`, \
                but it should start from scratch with an input of type `()`)."
            ),
        };

        // From `automata/src/combinators.rs` (in the original `>>` implementation):
        let mut s = self.generalize();
        let size = s.states.len();
        let Graph {
            states: call_states_d,
            initial: call_initial,
            tags: call_tags,
        } = parser.map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));
        let call_states: Vec<_> = call_states_d.into_iter().map(State::generalize).collect();
        let call_init_nd: BTreeSet<_> = iter::once(Ok(call_initial)).collect();

        let mut at_least_one_accepting = false;
        for state in &mut s.states {
            if state.non_accepting.is_empty() {
                // FIXME: This should be totally fine if the other transition is _also a call_
                state.transitions = match state.transitions {
                    Curry::Wildcard(_) => panic!("TODO: SPECIFY AN ERROR"),
                    Curry::Scrutinize(ref map) => {
                        if map.0.is_empty() {
                            Curry::Wildcard(Transition::Call {
                                region: "call", // <-- TODO
                                detour: call_init_nd.clone(),
                                dst: BTreeSet::new(),
                                combine: combinator.clone(),
                            })
                        } else {
                            panic!("TODO: SPECIFY AN ERROR");
                        }
                    }
                };
                at_least_one_accepting = true;
            }
        }
        if !at_least_one_accepting {
            panic!("TODO")
        }

        s.states.extend(call_states);
        s.tags.extend(call_tags);

        s.determinize().unwrap_or_else(|e| panic!("{e}"))
    }
}

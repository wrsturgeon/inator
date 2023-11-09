/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Save the current value, run this second parser from scratch, then combine the results.

use crate::{F, FF};
use core::{iter, ops};
use inator_automata::*;

/// Save the current value, run this second parser from scratch, then combine the results.
pub struct Call<I: Input, S: Stack> {
    /// Parser to call.
    parser: Deterministic<I, S>,
    /// Combine the tabled result with the result of the call.
    combinator: FF,
}

/// Save the current value, run this second parser from scratch, then combine the results.
#[inline]
#[must_use]
pub fn call<I: Input, S: Stack>(parser: Deterministic<I, S>, combinator: FF) -> Call<I, S> {
    Call { parser, combinator }
}

impl<I: Input, S: Stack> ops::Shr<Call<I, S>> for Deterministic<I, S> {
    type Output = Self;
    #[inline]
    #[must_use]
    #[allow(clippy::panic)]
    fn shr(self, Call { parser, combinator }: Call<I, S>) -> Self::Output {
        let Ok(maybe_parser_input_t) = parser.input_type() else {
            panic!("Inconsistent types in the parser argument to `combine`.")
        };
        let Some(parser_output_t) = maybe_parser_input_t else {
            panic!(
                "Parser argument to `combine` has no initial states, \
                so it can never parse anything.",
            )
        };
        if parser_output_t != "()" {
            panic!(
                "Called `call` with a parser that doesn't start from scratch \
                (it wants an input of type `{parser_output_t}`, \
                but it should start from scratch with an input of type `()`)."
            );
        }

        // From `automata/src/combinators.rs` (in the original `>>` implementation):
        let s = self.generalize();
        let size = s.states.len();
        let Graph {
            states: call_states,
            initial: call_initial,
            tags: call_tags,
        } = parser
            .generalize()
            .map_indices(|i| i.checked_add(size).expect("Absurdly huge number of states"));
        assert_eq!(call_initial.len(), 1);

        let call_init = get!(call_states, unwrap!(unwrap!(call_initial.first())));
        let init_trans = call_init.transitions.merge();
        for state in s.states.iter_mut().filter(|&s| {
            if s.non_accepting.is_empty() {
                s.non_accepting = iter::once(todo!()).collect();
                true
            } else {
                false
            }
        }) {
            state.transitions = state
                .transitions
                .merge(call_init.transitions)
                .unwrap_or_else(|e| panic!("{e}"));
        }

        s.states.extend(call_states);
        s.tags.extend(call_tags);

        // Split the accumulator into a passthrough unmodified first argument and a new modifiable second argument.
        let split = F {
            src: "|x| (x, ())".to_owned(),
            arg_t: parser_output_t.clone(),
            output_t: format!("({parser_output_t}, ())"),
        };
        let splat /* past tense */ = parser >> split;
    }
}

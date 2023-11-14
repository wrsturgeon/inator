/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Delimit a region with three parsers: one opens, one parses the contents, and one closes.

use crate::{Curry, Input, Parser, Transition, FF};

/// Delimit a region with three parsers: one opens, one parses the contents, and one closes.
#[inline]
#[must_use]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::todo,
    unused_mut,
    unused_variables
)]
pub fn region<I: Input>(
    name: &'static str,
    open: Parser<I>,
    contents: Parser<I>,
    mut close: Parser<I>,
    combine: FF,
) -> Parser<I> {
    // Split `close` into accepting and non-accepting states.
    // Move all accepting states out of `close` and into the post-return `dst` position.
    // Then, convert all transitions to a previously accepting state into a `Return`.
    // With this plan, maybe enforce that there is only one accepting state in `close`,
    // since there's no way to pick with a `Return`.
    // Or we could go back to `Pop` instead of `Return` and add a destination,
    // but that would introduce a function pointer at runtime in generated code.
    // TODO: resolve ^^^
    // Going with the first option (enforce one final state) for now.
    let close_final = close
        .states
        .iter()
        .enumerate()
        .fold(None, |acc, (i, s)| {
            if s.non_accepting.is_empty() {
                if let Some(already) = acc {
                    panic!("MESSAGE TODO")
                }
                Some(i)
            } else {
                acc
            }
        })
        .unwrap_or_else(|| {
            panic!(
                "No accepting states on the \
                closing parser of a region",
            )
        });

    // Each accepting state of `close` should become a non-accepting `Return` instead.
    for state in &mut close.states {
        match state.transitions {
            Curry::Wildcard(ref mut t) => sleight_of_hand(t, close_final, name),
            Curry::Scrutinize {
                ref mut filter,
                ref mut fallback,
                ..
            } => {
                for t in filter.values_mut() {
                    sleight_of_hand(t, close_final, name);
                }
                if let Some(ref mut t) = *fallback {
                    sleight_of_hand(t, close_final, name);
                }
            }
        }
    }

    open ^ (name, contents >> close, combine)
}

/// Convert a transition to an accepting state into a `Return`.
/// For use in closing parsers of a region.
#[inline]
fn sleight_of_hand<I: Input>(t: &mut Transition<I, usize>, fin: usize, region: &'static str) {
    if t.dsts().contains(&&fin) {
        *t = Transition::Return { region };
    }
}

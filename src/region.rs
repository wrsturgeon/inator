/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Delimit a region with three parsers: one opens, one parses the contents, and one closes.

use crate::{Curry, Input, Parser};

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
        .fold(None, |acc, s| {
            if s.non_accepting.is_empty() {
                if let Some(already) = acc {
                    panic!("MESSAGE TODO")
                }
                Some(s)
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
        if state.non_accepting.is_empty() {
            // CORRECTION: We don't actually have to make this non-accepting,
            // since the stack will be non-empty, so it will reject anyway.
            match state.transitions {
                Curry::Wildcard(_) => panic!("TODO"),
                Curry::Scrutinize {
                    ref filter,
                    ref fallback,
                    ..
                } => assert!(filter.0.is_empty(), "TODO"),
            }
        }
    }

    // Fuse everything after opening into one lateral parser.
    let post_open = contents >> close;

    todo!()
}

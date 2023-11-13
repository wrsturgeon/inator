/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Delimit a region with three parsers: one opens, one parses the contents, and one closes.

use crate::{Input, Parser};

/// Delimit a region with three parsers: one opens, one parses the contents, and one closes.
#[inline]
#[must_use]
#[allow(
    clippy::needless_pass_by_value,
    clippy::todo,
    unused_mut,
    unused_variables
)]
pub fn region<I: Input>(
    name: &str,
    open: Parser<I>,
    contents: Parser<I>,
    mut close: Parser<I>,
) -> Parser<I> {
    // // Each accepting state of `close` should become a non-accepting `Return` instead.
    // for state in &mut close.states {
    //     if state.non_accepting.is_empty() {
    //         state.non_accepting = iter::once(todo!()).collect();
    //     }
    // }

    // // Fuse everything after opening into one lateral parser.
    // let post_open = contents >> close;

    todo!()
}

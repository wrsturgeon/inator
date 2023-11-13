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
pub fn region<I: Input>(
    name: &str,
    open: Parser<I>,
    contents: Parser<I>,
    close: Parser<I>,
) -> Parser<I> {
    todo!()
}

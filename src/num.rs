/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Numeric utilities.

use crate::{any_of, combine, fixpoint, process, recurse};
use inator_automata::*;

/// Any digit character (0, 1, 2, 3, 4, 5, 6, 7, 8, 9).
#[inline]
#[must_use]
#[allow(
    clippy::missing_assert_message,
    clippy::missing_panics_doc,
    clippy::panic
)] // <-- FIXME
pub fn digit<S: Stack>() -> Deterministic<u8, S> {
    let out = any_of(
        Range {
            first: b'0',
            last: b'9',
        },
        update!(|_: u16, i| i),
    );
    match out.output_type() {
        Ok(maybe) => assert_eq!(maybe.as_deref(), Some("u8")),
        Err(e) => panic!("{e}"),
    }
    out
}

/// An unsigned integer consisting only of digits (e.g., no sign, no decimal point, no commas, etc.).
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn integer<S: Stack>() -> Deterministic<u8, S> {
    let shape = process(digit(), f!(|i: u8| usize::from(i)))
        >> fixpoint("integer")
        >> combine(digit(), ff!(|a: usize, b: usize| a * 10 + b))
        >> recurse("integer");
    shape.determinize().unwrap_or_else(|_| never!())
}

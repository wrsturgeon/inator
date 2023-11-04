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
pub fn digit<S: Stack>() -> Deterministic<u8, S> {
    any_of(
        Range {
            first: b'0',
            last: b'9',
        },
        update!(|(), i| i),
    )
}

/// An unsigned integer consisting only of digits (e.g., no sign, no decimal point, no commas, etc.).
#[inline]
pub fn integer<S: Stack>() -> Deterministic<u8, S> {
    let shape = process(digit(), f!(|i: u8| usize::from(i)))
        >> fixpoint("integer")
        >> combine(digit(), ff!(|a: usize, b: usize| a * 10 + b))
        >> recurse("integer");
    match shape.determinize() {
        Ok(d) => d.sort(),
        Err(_) => unreachable!(),
    }
}

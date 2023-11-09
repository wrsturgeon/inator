/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Numeric utilities.

use crate::{any_of, call, fixpoint, recurse};
use inator_automata::*;

/// Any digit character (0, 1, 2, 3, 4, 5, 6, 7, 8, 9).
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn digit() -> Deterministic<u8> {
    any_of(
        Range {
            first: b'0',
            last: b'9',
        },
        update!(|(), i| i - b'0'),
    )
}

/// An unsigned integer consisting only of digits (e.g., no sign, no decimal point, no commas, etc.).
#[inline]
#[must_use]
#[allow(clippy::arithmetic_side_effects)]
pub fn integer() -> Deterministic<u8> {
    digit()
        >> f!(|i: u8| Some(usize::from(i)))
        >> fixpoint("integer")
        >> call(
            digit(),
            ff!(|a: Option<usize>, b| a?.checked_mul(10)?.checked_add(b)),
        )
        >> recurse("integer")
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Function representations.

use inator_automata::ToSrc;

/// One-argument function.
pub struct F1 {
    pub src: String,
    pub arg_t: String,
    pub output_t: String,
}

/// Two-argument function.
pub struct F2 {
    pub src: String,
    pub lhs_t: String,
    pub rhs_t: String,
    pub output_t: String,
}

impl F1 {
    #[inline]
    #[must_use]
    pub fn _from_macro<Arg: ToSrc, Output: ToSrc>(src: String, _: fn(Arg) -> Output) -> Self {
        Self {
            src,
            arg_t: Arg::src_type(),
            output_t: Output::src_type(),
        }
    }
}

impl F2 {
    #[inline]
    #[must_use]
    pub fn _from_macro<Lhs: ToSrc, Rhs: ToSrc, Output: ToSrc>(
        src: String,
        _: fn(Lhs, Rhs) -> Output,
    ) -> Self {
        Self {
            src,
            lhs_t: Lhs::src_type(),
            rhs_t: Rhs::src_type(),
            output_t: Output::src_type(),
        }
    }
}

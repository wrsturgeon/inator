/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Function representations.

#![allow(clippy::module_name_repetitions)]

use inator_automata::ToSrc;

/// One-argument function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct F {
    /// Source-code representation of this function.
    pub src: String,
    /// Argument type.
    pub arg_t: String,
    /// Output type.
    pub output_t: String,
}

/// Two-argument function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FF {
    /// Source-code representation of this function.
    pub src: String,
    /// Type of the first argument.
    pub lhs_t: String,
    /// Type of the second argument.
    pub rhs_t: String,
    /// Output type.
    pub output_t: String,
}

impl F {
    /// Internals of the `f!(...)` macro.
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

impl FF {
    /// Internals of the `ff!(...)` macro.
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

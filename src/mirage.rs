/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Copies of names that proc-macros replace, but as actual macros so IDEs can grasp them.

#![allow(clippy::needless_pass_by_value, unused_variables)]

/// Match exactly this pattern.
/// Note that, unlike a normal Rust function, you can use any pattern here:
/// e.g. `p('A' | 'B' | 'C')`.
#[macro_export]
macro_rules! p {
    ($pat:pat) => {
        compile_error!(
            "The `p!(...)` macro from `inator` only work in functions marked `#[inator]`"
        )
    };
}

pub use p;

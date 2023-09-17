/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Copies of names that proc-macros replace, but as actual macros so IDEs can grasp them.
//! Destructive dual to `facade.rs`.

#![allow(clippy::needless_pass_by_value, unused_variables)]

/// Empty parser to represent unprocessed macros in source code for IDEs.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Parser;

/// Match exactly this pattern.
/// Note that, unlike a normal Rust function, you can use any pattern here:
/// e.g. `p('A' | 'B' | 'C')`.
#[macro_export]
macro_rules! safe_p {
    ($pat:pat) => {
        Parser
    };
}

pub use safe_p as p;

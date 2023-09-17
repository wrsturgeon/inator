/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Copies of names that proc-macros replace, but as actual macros so IDEs can grasp them.

#![allow(clippy::needless_pass_by_value, unused_variables)]

/// Empty parser to represent unprocessed macros in source code for IDEs.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Parser;

impl<I> crate::traits::Parse<I> for Parser {
    type Output = core::convert::Infallible;
}

impl core::ops::BitOr for Parser {
    type Output = Parser;
    fn bitor(self, rhs: Self) -> Self::Output {
        panic!("The `p!(...)` macro from `inator` only work in functions marked `#[inator]`");
    }
}

impl core::ops::Shl for Parser {
    type Output = Parser;
    fn shl(self, rhs: Self) -> Self::Output {
        panic!("The `p!(...)` macro from `inator` only work in functions marked `#[inator]`");
    }
}

impl core::ops::Shr for Parser {
    type Output = Parser;
    fn shr(self, rhs: Self) -> Self::Output {
        panic!("The `p!(...)` macro from `inator` only work in functions marked `#[inator]`");
    }
}

/// Match exactly this pattern.
/// Note that, unlike a normal Rust function, you can use any pattern:
/// e.g. `p!('A' | 'B' | 'C')`.
#[macro_export]
macro_rules! p {
    ($pat:pat) => {
        // compile_error!(
        //     "The `p!(...)` macro from `inator` only work in functions marked `#[inator]`"
        // )
        Parser
    };
}

pub use p;

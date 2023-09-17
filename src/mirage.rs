/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Copies of names that proc-macros replace, but as actual macros so IDEs can grasp them.

#![allow(clippy::needless_pass_by_value, unused_variables)]

use core::marker::PhantomData;

/// Empty parser to represent unprocessed macros in source code for IDEs.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Parser<O>(PhantomData<O>);

impl<O> Parser<O> {
    #[must_use]
    #[inline(always)]
    #[allow(missing_docs)]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I, O> crate::traits::Parse<I> for Parser<O> {
    type Output = O;
}

impl<O> core::ops::BitOr for Parser<O> {
    type Output = Parser<O>;
    #[allow(clippy::missing_inline_in_public_items, clippy::panic)]
    fn bitor(self, rhs: Self) -> Self::Output {
        panic!("The `p!(...)` macro from `inator` only work in functions marked `#[inator]`");
    }
}

impl<O> core::ops::Shl for Parser<O> {
    type Output = Parser<O>;
    #[allow(clippy::missing_inline_in_public_items, clippy::panic)]
    fn shl(self, rhs: Self) -> Self::Output {
        panic!("The `p!(...)` macro from `inator` only work in functions marked `#[inator]`");
    }
}

impl<O> core::ops::Shr for Parser<O> {
    type Output = Parser<O>;
    #[allow(clippy::missing_inline_in_public_items, clippy::panic)]
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
        Parser::new()
    };
}

pub use p;

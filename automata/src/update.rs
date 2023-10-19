/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A single-argument Rust function callable both in `build.rs` and in a source file.

use crate::{Input, Output};

/// A single-argument Rust function callable both in `build.rs` and in a source file.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Update<I: Input, O: Output> {
    /// The literal function in memory: you can call it if you'd like.
    pub ptr: fn(O, &I) -> O,
    /// Source-code representation that's promised to compile to a call operationally identical to `ptr`.
    pub src: &'static str,
}

impl<I: Input, O: Output> Clone for Update<I, O> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<I: Input, O: Output> Copy for Update<I, O> {}

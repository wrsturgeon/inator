/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! A single-argument Rust function callable both in `build.rs` and in a source file.

use crate::{Input, Output};
use core::{cmp, fmt};

/// A single-argument Rust function callable both in `build.rs` and in a source file.
#[allow(clippy::exhaustive_structs)]
pub struct Update<I: Input, O: Output> {
    /// The literal function in memory: you can call it if you'd like.
    pub ptr: fn(O, &I) -> O,
    /// Source-code representation that's promised to compile to a call operationally identical to `ptr`.
    pub src: &'static str,
}

impl<I: Input, O: Output> PartialEq for Update<I, O> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src
    }
}

impl<I: Input, O: Output> Eq for Update<I, O> {}

impl<I: Input, O: Output> PartialOrd for Update<I, O> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, O: Output> Ord for Update<I, O> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.src.cmp(other.src)
    }
}

impl<I: Input, O: Output> fmt::Debug for Update<I, O> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "update!({})", self.src)
    }
}

impl<I: Input, O: Output> Clone for Update<I, O> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<I: Input, O: Output> Copy for Update<I, O> {}

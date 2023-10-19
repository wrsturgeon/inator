/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait to fallibly combine multiple values into one value with identical semantics.

use crate::{Ctrl, IllFormed, Input};

/// Trait to fallibly combine multiple values into one value with identical semantics.
pub trait Merge<I: Input, S, C: Ctrl>: Sized {
    /// Fallibly combine multiple values into one value with identical semantics.
    /// # Errors
    /// Implementation-defined: if the merge as we define it can't work.
    fn merge(self, other: &Self) -> Result<Self, IllFormed<I, S, C>>;
}

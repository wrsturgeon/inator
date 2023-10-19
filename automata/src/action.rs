/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Actions inspired by visibly pushdown automata: either
//! - `Local`, which can neither push nor pop from the stack;
//! - `Push`, which can only push to the stack; or
//! - `Pop`, which can only pop from the stack.

use crate::{Call, Input};

/// Actions inspired by visibly pushdown automata: either
/// - `Local`, which can neither push nor pop from the stack;
/// - `Push`, which can only push to the stack; or
/// - `Pop`, which can only pop from the stack.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Action<I: Input, S> {
    /// Neither push nor pop from the stack.
    Local {
        /// Call this Rust function when we take this transition.
        f: Call<I, ()>,
    },
    /// Push a new symbol onto the stack.
    Push {
        /// Call this Rust function when we take this transition.
        f: Call<I, ()>,
        /// Push this token onto the stack when we take this transition.
        push: S,
    },
    /// Pop a symbol from the stack.
    Pop {
        /// Call this Rust function when we take this transition.
        f: Call<(I, S), ()>,
    },
}

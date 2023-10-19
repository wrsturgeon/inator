/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Actions inspired by visibly pushdown automata: either
//! - `Local`, which can neither push nor pop from the stack;
//! - `Push`, which can only push to the stack; or
//! - `Pop`, which can only pop from the stack.

use crate::Stack;

/// Actions inspired by visibly pushdown automata: either
/// - `Local`, which can neither push nor pop from the stack;
/// - `Push`, which can only push to the stack; or
/// - `Pop`, which can only pop from the stack.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Action<S: Stack> {
    /// Neither push nor pop from the stack.
    Local,
    /// Push a new symbol onto the stack.
    Push(S),
    /// Pop a symbol from the stack.
    Pop,
}

impl<S: Stack> Action<S> {
    /// Take this transition in an actual execution.
    /// Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop from an empty stack.
    #[inline]
    pub fn invoke(&self, stack: &mut Vec<S>) -> Result<(), bool> {
        match *self {
            Self::Local => {}
            Self::Push(symbol) => stack.push(symbol),
            Self::Pop => {
                if stack.pop().is_none() {
                    return Err(false);
                }
            }
        }
        Ok(())
    }
}

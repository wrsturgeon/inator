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
use core::cmp;

/// Actions inspired by visibly pushdown automata: either
/// - `Local`, which can neither push nor pop from the stack;
/// - `Push`, which can only push to the stack; or
/// - `Pop`, which can only pop from the stack.
#[allow(clippy::exhaustive_enums)]
#[derive(Clone, Copy, Debug)]
pub enum Action<S: Stack> {
    /// Neither push nor pop from the stack.
    Local,
    /// Push a new symbol onto the stack.
    Push(S),
    /// Pop a symbol from the stack.
    Pop,
}

impl<S: Stack> PartialEq for Action<S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Local, &Self::Local) | (&Self::Pop, &Self::Pop) => true,
            (&Self::Push(ref a), &Self::Push(ref b)) => a == b,
            _ => false,
        }
    }
}
impl<S: Stack> Eq for Action<S> {}
impl<S: Stack> PartialOrd for Action<S> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<S: Stack> Ord for Action<S> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (&Self::Local, &Self::Local) | (&Self::Pop, &Self::Pop) => cmp::Ordering::Equal,
            (&Self::Local, _) | (&Self::Push(..), &Self::Pop) => cmp::Ordering::Less,
            (&Self::Push(..), &Self::Local) | (&Self::Pop, _) => cmp::Ordering::Greater,
            (&Self::Push(ref a), &Self::Push(ref b)) => a.cmp(b),
        }
    }
}

impl<S: Stack> Action<S> {
    /// Take this transition in an actual execution.
    /// Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop from an empty stack.
    #[inline]
    pub fn invoke(&self, stack: &mut Vec<S>) -> Option<()> {
        match *self {
            Self::Local => {}
            Self::Push(ref symbol) => stack.push(symbol.clone()),
            Self::Pop => drop(stack.pop()?),
        }
        Some(())
    }
}

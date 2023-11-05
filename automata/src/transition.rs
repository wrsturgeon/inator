/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Transition in an automaton: an action and a destination state.

use crate::{Ctrl, IllFormed, Input, Update};
use core::cmp;

/// Transition in an automaton: an action and a destination state.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub enum Transition<I: Input, C: Ctrl<I>> {
    /// Neither push nor pop: just move to a different state.
    Lateral {
        /// Go to this state.
        dst: C,
        /// Call this Rust function to update the output we're building.
        update: Update<I>,
    },
    /// Call another function--i.e., push a pointer/index onto the stack.
    Call {/* TODO */},
    /// Return into the function that called us.
    /// Note that this is NOT how we return from the overall parser:
    /// that happens only when input ends AND the stack is empty.
    Return {/* TODO */},
}

impl<I: Input, C: Ctrl<I>> Clone for Transition<I, C> {
    #[inline]
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<I: Input, C: Ctrl<I>> PartialEq for Transition<I, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}
impl<I: Input, C: Ctrl<I>> Eq for Transition<I, C> {}

impl<I: Input, C: Ctrl<I>> Ord for Transition<I, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        todo!()
    }
}

impl<I: Input, C: Ctrl<I>> PartialOrd for Transition<I, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, C: Ctrl<I>> Transition<I, C> {
    /// Take this transition in an actual execution.
    /// Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop from an empty stack.
    #[inline]
    pub fn invoke(&self, output_t: &str) -> Result<Option<(C, String)>, IllFormed<I, C>> {
        todo!()
    }

    /// Compute the input type of any run that reaches this state.
    #[inline]
    pub fn input_type(&self) -> String {
        todo!()
    }
}

impl<I: Input> Transition<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> Transition<I, C> {
        todo!()
    }
}

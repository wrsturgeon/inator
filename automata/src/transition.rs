/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Transition in an automaton: an action and a destination state.

use crate::{Action, Ctrl, Input, Output, Stack, Update};
use core::cmp;

/// Transition in an automaton: an action and a destination state.
#[allow(clippy::exhaustive_structs)]
#[derive(Copy, Debug)]
pub struct Transition<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> {
    /// Go to this state.
    pub dst: C,
    /// Take this action: maybe push/pop from the stack.
    pub act: Action<S>,
    /// Call this Rust function to update the output we're building.
    pub update: Update<I, O>,
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Clone for Transition<I, S, O, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            dst: self.dst.clone(),
            act: self.act.clone(),
            update: self.update,
        }
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialEq for Transition<I, S, O, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.dst == other.dst && self.act == other.act && self.update == other.update
    }
}
impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Eq for Transition<I, S, O, C> {}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Ord for Transition<I, S, O, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.dst
            .cmp(&other.dst)
            .then_with(|| self.act.cmp(&other.act))
            .then_with(|| self.update.cmp(&other.update))
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> PartialOrd for Transition<I, S, O, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, S: Stack, O: Output, C: Ctrl<I, S, O>> Transition<I, S, O, C> {
    /// Take this transition in an actual execution.
    /// Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop from an empty stack.
    #[inline]
    pub fn invoke(&self, token: &I, stack: &mut Vec<S>, output: O) -> (Option<C>, O) {
        (
            self.act.invoke(stack).map(|()| self.dst.clone()),
            (self.update.ptr)(output, token),
        )
    }
}

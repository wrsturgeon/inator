/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Transition in an automaton: an action and a destination state.

use crate::{Action, Ctrl, IllFormed, Input, Stack, Update};
use core::cmp;

/// Transition in an automaton: an action and a destination state.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug)]
pub struct Transition<I: Input, S: Stack, C: Ctrl<I, S>> {
    /// Go to this state.
    pub dst: C,
    /// Take this action: maybe push/pop from the stack.
    pub act: Action<S>,
    /// Call this Rust function to update the output we're building.
    pub update: Update<I>,
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Clone for Transition<I, S, C> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            dst: self.dst.clone(),
            act: self.act.clone(),
            update: self.update.clone(),
        }
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialEq for Transition<I, S, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.dst == other.dst && self.act == other.act && self.update == other.update
    }
}
impl<I: Input, S: Stack, C: Ctrl<I, S>> Eq for Transition<I, S, C> {}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Ord for Transition<I, S, C> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.dst
            .cmp(&other.dst)
            .then_with(|| self.act.cmp(&other.act))
            .then_with(|| self.update.cmp(&other.update))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> PartialOrd for Transition<I, S, C> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Input, S: Stack, C: Ctrl<I, S>> Transition<I, S, C> {
    /// Take this transition in an actual execution.
    /// Return the index of the machine's state after this transition.
    /// # Errors
    /// If we try to pop from an empty stack.
    #[inline]
    pub fn invoke(
        &self,
        stack: &mut Vec<S>,
        output_t: &str,
    ) -> Result<Option<(C, String)>, IllFormed<I, S, C>> {
        (output_t == self.update.input_t)
            .then(|| {
                self.act.invoke(stack)?;
                Some((self.dst.clone(), self.update.output_t.clone()))
            })
            .ok_or(IllFormed::TypeMismatch(
                output_t.to_owned(),
                self.update.input_t.clone(),
            ))
    }
}

impl<I: Input, S: Stack> Transition<I, S, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I, S>>(self) -> Box<Transition<I, S, C>> {
        Box::new(Transition {
            dst: C::from_usize(self.dst),
            act: self.act,
            update: self.update,
        })
    }
}

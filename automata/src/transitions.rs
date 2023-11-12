/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Zero or more calls (in order), then either another state or returning to the last state on the stack.

use crate::{Call, *};

/// Zero or more calls (in order), then either another state or returning to the last state on the stack.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Transitions<I: Input, C: Ctrl<I>> {
    /// Call an arbitrary number of other states until they hit a `Return` transition, in order.
    pub calls: Vec<Call<I, C>>,
    /// After all calls have returned, execute this action.
    pub dst: Transition<I, C>,
}

impl<I: Input> Transitions<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> Transitions<I, C> {
        Transitions {
            calls: self.calls.into_iter().map(Call::convert_ctrl).collect(),
            dst: self.dst.convert_ctrl(),
        }
    }
}

impl<I: Input, C: Ctrl<I>> Transitions<I, C> {
    /// Compute the input type of any run that reaches this state.
    #[inline]
    #[must_use]
    pub fn input_type(&self) -> Option<&str> {
        self.calls.first().map_or_else(
            || self.dst.input_type(),
            |at_least_one_call| Some(at_least_one_call.input_type()),
        )
    }

    /// Immediate next destination (as a state index).
    /// For local transitions, it's what you would expect.
    /// For calls, it's both the call and the continuation after the call.
    /// For returns, it's nothing.
    #[inline]
    #[must_use]
    pub fn dsts(&self) -> Vec<&C> {
        self.calls
            .iter()
            .map(|c| &c.init)
            .chain(self.dst.dst())
            .collect()
    }
}

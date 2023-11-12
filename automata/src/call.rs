/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Call to another state, pushing a region name and return state onto the stack.
//! Currently implemented as just the above; might later separate each call into its own unique pathway.

use crate::{Ctrl, Input, FF};
use core::marker::PhantomData;

/// Call to another state, pushing a region name and return state onto the stack.
/// Currently implemented as just the above; might later separate each call into its own unique pathway.
#[allow(clippy::exhaustive_structs)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Call<I: Input, C: Ctrl<I>> {
    /// Region (user-defined name) that we're opening. Sensible to be e.g. "parentheses" for `(...)`.
    pub region: &'static str,
    /// Call (and require a successful run from) this state before continuing.
    pub init: C,
    /// Combine the cached results and the results of the called parser with this function.
    pub combine: FF,
    /// Zero-size stand-in for the input token type.
    pub ghost: PhantomData<I>,
}

impl<I: Input> Call<I, usize> {
    /// Convert the control parameter from `usize` to anything else.
    #[inline]
    #[must_use]
    pub fn convert_ctrl<C: Ctrl<I>>(self) -> Call<I, C> {
        Call {
            region: self.region,
            init: C::from_usize(self.init),
            combine: self.combine,
            ghost: self.ghost,
        }
    }
}

impl<I: Input, C: Ctrl<I>> Call<I, C> {
    /// Compute the input type of any run that reaches this state.
    #[inline]
    #[must_use]
    pub fn input_type(&self) -> &str {
        &self.combine.lhs_t
    }
}

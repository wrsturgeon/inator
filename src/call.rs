/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! User-defined functions to call on transitions.

/// A user-defined function to call on a transition.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Call {
    /// Name of the function, which becomes an identifier only AFTER `build.rs`.
    pub(crate) name: &'static str,
    /// Whether the function inspects an input token at runtime.
    pub(crate) takes_arg: bool,
}

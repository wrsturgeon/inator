/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Traits related to parsing.

/// Anything that can parse input.
pub trait Parse<Input> {
    /// Output of a successful parser run.
    type Output;
}

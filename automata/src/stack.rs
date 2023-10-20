/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Stack symbol.

use crate::ToSrc;

/// Stack symbol.
pub trait Stack: Clone + Ord + ToSrc {}

impl<S: Clone + Ord + ToSrc> Stack for S {}

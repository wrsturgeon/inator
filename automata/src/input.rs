/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Typing convenience: trait satisfying everything required for an input token.

use crate::ToSrc;

/// Typing convenience: trait satisfying everything required for an input token.
pub trait Input: Clone + Ord + ToSrc {}

impl<I: Clone + Ord + ToSrc> Input for I {}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Trait defining everything required to work as output of an automaton.

use crate::ToSrc;

/// Trait defining everything required to work as output of an automaton.
pub trait Output: Default + Sized + ToSrc {}

impl<O: Default + Sized + ToSrc> Output for O {}

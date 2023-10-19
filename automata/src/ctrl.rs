/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Necessary preconditions to function as an index.

use core::iter;

/// Necessary preconditions to function as an index.
pub trait Ctrl: Clone {
    /// Non-owning view over each index in what may be a collection.
    type View: Iterator<Item = usize>;
    /// View each index in what may be a collection.
    fn view(&self) -> Self::View;
}

impl Ctrl for usize {
    type View = iter::Once<usize>;
    #[inline]
    fn view(&self) -> Self::View {
        iter::once(*self)
    }
}

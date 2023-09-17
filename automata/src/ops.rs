/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Operations on NFAs.

use crate::Nfa;

impl<I: Clone + Ord> core::ops::BitOr for Nfa<I> {
    type Output = Nfa<I>;
    #[inline]
    #[allow(clippy::todo)] // FIXME
    fn bitor(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

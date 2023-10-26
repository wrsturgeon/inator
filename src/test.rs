/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[cfg(feature = "quickcheck")]
mod prop {
    use crate::*;
    use quickcheck::*;

    quickcheck! {
        fn prop_empty(input: Vec<u8>) -> bool {
            input.is_empty() == empty::<u8, u8>().accept(input).is_ok()
        }
    }
}

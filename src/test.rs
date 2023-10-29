/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::indexing_slicing)]

#[cfg(feature = "quickcheck")]
mod prop {
    use crate::*;
    use quickcheck::*;

    quickcheck! {
        fn prop_empty(input: Vec<u8>) -> bool {
            let parser = empty::<u8, u8>();
            if parser.check().is_err() { return false; }
            input.is_empty() == empty::<u8, u8>().accept(input).is_ok()
        }

        fn prop_any_of(range: Range<u8>, input: Vec<u8>) -> bool {
            let parser = any_of::<_, ()>(range, update!(|(), _| {}));
            if parser.check().is_err() { return false; }
            parser.accept(input.iter().copied()).is_ok() == (input.len() == 1 && range.contains(&input[0]))
        }
    }
}

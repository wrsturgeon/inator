/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::*;

mod unit {
    use super::*;

    #[test]
    fn simple_recursion() {
        let nd = fixpoint("start") >> (empty() | (toss('a') >> recurse("start")));
        let d = nd.determinize().unwrap();
    }
}

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;

    quickcheck! {
        fn prop_empty(input: Vec<u8>) -> bool {
            input.is_empty() == empty::<u8, u8>().accept(input).is_ok()
        }
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use inator_automata::*;

mod unit {
    use super::*;

    #[test]
    fn simple_recursion() {
        let nd = fixpoint("start") >> (empty() | (toss('a') >> recurse("start")));
        let d = nd.determinize().unwrap();
    }
}

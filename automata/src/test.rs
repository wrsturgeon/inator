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
        fn range_both_contains_implies_intersection(
            v: u8,
            lhs: Range<u8>,
            rhs: Range<u8> // <-- no trailing comma allowed :_(
        ) -> TestResult {
            if lhs.contains(&v) && rhs.contains(&v) {
                lhs.intersection(rhs).map_or_else(
                    TestResult::failed,
                    |range| TestResult::from_bool(range.contains(&v)),
                )
            } else {
                TestResult::discard()
            }
        }

        // With discarding, this takes a ridiculously long time.
        fn check_implies_no_runtime_errors(
            nd: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            if nd.check().is_err() {
                return true;
            }
            let mut run = input.run(&nd);
            for r in &mut run {
                if r.is_err() {
                    return false;
                }
            }
            true
        }

        fn runtime_error_implies_not_check(
            nd: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> TestResult {
            let mut run = input.run(&nd);
            for r in &mut run {
                if r.is_err() {
                    return TestResult::from_bool(
                        nd.check().is_err(),
                    );
                }
            }
            TestResult::discard()
        }
    }
}

/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::print_stdout, clippy::unreachable, clippy::use_debug)]

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
            if nd.check().is_ok() {
                !matches!(nd.accept(input), Err(ParseError::BadParser(..)))
            } else {
                true
            }
        }

        fn union(
            lhs: Nondeterministic<u8, u8, u8>,
            rhs: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            if lhs.check().is_err() || rhs.check().is_err() {
                return true;
            }
            let union = lhs.clone() | rhs.clone();
            let union_accept = union.accept(input.iter().copied());
            lhs.accept(input.iter().copied())
                .or_else(|_| rhs.accept(input.iter().copied()))
                .map_or_else(
                    |e| match e {
                        reject @ ParseError::BadInput(..) => union_accept == Err(reject),
                        ParseError::BadParser(..) => unreachable!(),
                    },
                    |ok| union_accept == Ok(ok),
                )
        }

        fn union_preserves_ill_formedness(
            lhs: Nondeterministic<u8, u8, u8>,
            rhs: Nondeterministic<u8, u8, u8>
        ) -> bool {
            (lhs.check().is_err() || rhs.check().is_err()) == (lhs | rhs).check().is_err()
        }
    }
}

mod reduced {
    use crate::*;
    use core::iter;
    use std::collections::{BTreeMap, BTreeSet};

    fn union(lhs: &Nondeterministic<u8, u8, u8>, rhs: &Nondeterministic<u8, u8, u8>, input: &[u8]) {
        if lhs.check().is_err() || rhs.check().is_err() {
            return;
        }
        println!("{lhs:?}");
        println!("union");
        println!("{rhs:?}");
        println!("yields");
        let union = lhs.clone() | rhs.clone();
        println!("{union:?}");
        let union_accept = union.accept(input.iter().copied());
        lhs.accept(input.iter().copied())
            .or_else(|_| rhs.accept(input.iter().copied()))
            .map_or_else(
                |e| match e {
                    reject @ ParseError::BadInput(..) => assert_eq!(union_accept, Err(reject)),
                    ParseError::BadParser(..) => unreachable!(),
                },
                |ok| assert_eq!(union_accept, Ok(ok)),
            );
    }

    #[test]
    fn union_1() {
        union(
            &Graph {
                states: vec![],
                initial: iter::once(0).collect(),
            },
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn union_2() {
        union(
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            &Graph {
                states: vec![],
                initial: iter::once(0).collect(),
            },
            &[],
        );
    }

    #[test]
    fn union_3() {
        union(
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: iter::once((
                            0,
                            CurryInput::Wildcard(Transition {
                                dst: BTreeSet::new(),
                                act: Action::Local,
                                update: update!(|x, _| x),
                            }),
                        ))
                        .collect(),
                    },
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            &Graph {
                states: vec![],
                initial: iter::once(0).collect(),
            },
            &[],
        );
    }
}

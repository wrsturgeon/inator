/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::integer_division,
    clippy::panic,
    clippy::unreachable,
    clippy::unwrap_used
)]

#[cfg(feature = "quickcheck")]
mod prop {
    use crate::*;
    use core::num::NonZeroUsize;
    use quickcheck::*;
    use std::{collections::BTreeSet, env, panic};

    #[inline]
    fn gen_size() -> usize {
        env::var("QUICKCHECK_GENERATOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100)
    }

    #[inline]
    fn qc_tests() -> usize {
        env::var("QUICKCHECK_TESTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100)
    }

    #[inline]
    fn nz(i: usize) -> NonZeroUsize {
        NonZeroUsize::new(i).unwrap()
    }

    #[test]
    fn arbitrary_implies_check_deterministic() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            let d = Deterministic::<u8>::arbitrary(&mut Gen::new(curved.into()));
            assert_eq!(d.check(), Ok(()), "{d:?}");
        }
    }

    #[test]
    fn arbitrary_implies_check_nondeterministic() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            let nd = Nondeterministic::<u8>::arbitrary(&mut Gen::new(curved.into()));
            assert_eq!(nd.check(), Ok(()), "{nd:?}");
        }
    }

    #[test]
    fn arbitrary_implies_check_state() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                State::<u8, BTreeSet<Result<usize, String>>>::arbitrary_given(
                    curved,
                    &mut Gen::new(curved.into())
                )
                .check(curved),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_curry() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                Curry::<u8, BTreeSet<Result<usize, String>>>::arbitrary_given(
                    curved,
                    &mut Gen::new(curved.into())
                )
                .check(curved),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_range_map() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                RangeMap::<u8, BTreeSet<Result<usize, String>>>::arbitrary_given(
                    curved,
                    &mut Gen::new(curved.into())
                )
                .check(curved),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_transition() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                Transition::<u8, BTreeSet<Result<usize, String>>>::arbitrary_given(
                    curved,
                    &mut Gen::new(curved.into())
                )
                .check(curved),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_usize() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                <usize as Check<u8, usize>>::check(
                    &<usize as Ctrl<u8>>::arbitrary_given(curved, &mut Gen::new(curved.into())),
                    curved
                ),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_btreeset_usize() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                <BTreeSet<Result<usize, String>> as Check<
                    u8,
                    BTreeSet<Result<usize, String>>,
                >>::check(
                    &<BTreeSet<Result<usize, String>> as Ctrl<u8>>::arbitrary_given(
                        curved,
                        &mut Gen::new(curved.into())
                    ),
                    curved
                ),
                Ok(())
            );
        }
    }

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

        // *** These do NOT hold! ***
        // Determinization takes exactly as long as checking if determinzation will succeed,
        // and determinization is the only way to check a priori for runtime errors.
        // fn check_implies_no_runtime_errors(
        //     nd: Nondeterministic<u8>,
        //     input: Vec<u8>
        // ) -> bool {
        //     !matches!(nd.accept(input), Err(ParseError::BadParser(..)))
        // }
        //
        // fn check_implies_determinize(nd: Nondeterministic<u8>) -> bool {
        //     nd.determinize().is_ok()
        // }

        fn determinize_implies_no_runtime_errors(
            nd: Nondeterministic<u8>,
            input: Vec<u8>
        ) -> bool {
            let Ok(d) = nd.determinize() else {
                return true; // irrelevant
            };
            !matches!(d.accept(input), Err(ParseError::BadParser(..)))
        }

        fn deterministic_implies_no_runtime_errors(
            d: Deterministic<u8>,
            input: Vec<u8>
        ) -> bool {
            !matches!(d.accept(input), Err(ParseError::BadParser(..)))
        }

        fn determinize_identity(d: Deterministic<u8>, input: Vec<u8>) -> bool {
            let Ok(dd) = d.determinize() else {
                return false;
            };
            dd.accept(input.iter().copied()) == d.accept(input)
        }

        fn union(
            lhs: Deterministic<u8>,
            rhs: Deterministic<u8>,
            input: Vec<u8>
        ) -> bool {
            let Ok(union) = panic::catch_unwind(|| lhs.clone() | rhs.clone()) else {
                return true;
            };
            if union.check().is_err() {
                return false;
            }
            if union.determinize().is_err() {
                return true;
            }
            let union_accept = union.accept(input.iter().copied());
            match (
                lhs.accept(input.iter().copied()),
                rhs.accept(input.iter().copied()),
            ) {
                (Ok(a), Ok(b)) => {
                    if a == b {
                        union_accept == Ok(a)
                    } else {
                        matches!(union_accept, Err(ParseError::BadParser(..)))
                    }
                }
                (Err(e), Ok(out)) | (Ok(out), Err(e)) => match e {
                    ParseError::BadInput(..) => union_accept == Ok(out),
                    ParseError::BadParser(..) => unreachable!(),
                },
                (Err(ParseError::BadParser(..)), Err(..))
                | (Err(..), Err(ParseError::BadParser(..))) => {
                    matches!(union_accept, Err(ParseError::BadParser(..)))
                }
                (Err(ParseError::BadInput(..)), Err(ParseError::BadInput(..))) => {
                    union_accept.is_err()
                }
            }
        }

        fn sort(parser: Nondeterministic<u8>, input: Vec<u8>) -> bool {
            let pre = parser.accept(input.iter().copied());
            let mut sorted = parser;
            sorted.sort();
            let post = sorted.accept(input);
            match pre {
                Ok(out) => Ok(out) == post,
                Err(ParseError::BadInput(_)) => {
                    matches!(post, Err(ParseError::BadInput(_)))
                }
                Err(ParseError::BadParser(_)) => true
            }
        }

        fn shr(lhs: Deterministic<u8>, rhs: Deterministic<u8>, input: Vec<u8>) -> bool {
            let splittable = (0..=input.len()).any(|i| {
                lhs.accept(input[..i].iter().copied()).is_ok() &&
                rhs.accept(input[i..].iter().copied()).is_ok()
            });
            let Ok(concat) = panic::catch_unwind(|| lhs >> rhs) else {
                return true;
            };
            if concat.check().is_err() {
                return false;
            }
            if concat.determinize().is_err() {
                return true;
            }
            concat.accept(input).is_ok() == splittable
        }
    }
}

mod reduced {
    use crate::*;

    fn deterministic_implies_no_runtime_errors(d: &Deterministic<u8>, input: Vec<u8>) {
        if let Err(ParseError::BadParser(e)) = d.accept(input) {
            panic!("{e}");
        }
    }

    fn determinize_identity(d: &Deterministic<u8>, input: Vec<u8>) {
        let dd = match d.determinize() {
            Ok(dd) => dd,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(dd.accept(input.iter().copied()), d.accept(input));
    }

    #[test]
    fn deterministic_implies_no_runtime_errors_1() {
        deterministic_implies_no_runtime_errors(
            &Graph {
                states: vec![State {
                    transitions: Curry::Wildcard(Transition::Return),
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
                tags: BTreeMap::new(),
            },
            vec![0],
        );
    }

    #[test]
    fn determinize_identity_1() {
        determinize_identity(
            &Graph {
                states: vec![
                    State {
                        transitions: Curry::Wildcard(Transition::Lateral {
                            dst: 4,
                            update: update!(|(), _| {}),
                        }),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Return),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Scrutinize(RangeMap(BTreeMap::new())),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Return),
                        non_accepting: iter::once(String::new()).collect(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Call {
                            detour: 0,
                            dst: 1,
                            combine: FF {
                                src: "|(), ()| ()".to_owned(),
                                lhs_t: "()".to_owned(),
                                rhs_t: "()".to_owned(),
                                output_t: "()".to_owned(),
                            },
                        }),
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: 0,
                tags: BTreeMap::new(),
            },
            vec![],
        );
    }
}

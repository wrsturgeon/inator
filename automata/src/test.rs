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
    clippy::print_stdout,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::use_debug
)]

use crate::*;

#[inline]
#[must_use]
fn splittable<I: Input, C: Ctrl<I>>(parser: &Graph<I, C>, input: &[I]) -> bool {
    match input.len() {
        0 => true,
        len => {
            for i in (1..=len).rev() {
                if parser.accept(get!(input, ..i).iter().cloned()).is_ok()
                    && splittable(parser, get!(input, i..))
                {
                    return true;
                }
            }
            false
        }
    }
}

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
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
                State::<u8, BTreeSet<usize>>::arbitrary_given(curved, &mut Gen::new(curved.into()))
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
                Curry::<u8, BTreeSet<usize>>::arbitrary_given(curved, &mut Gen::new(curved.into()))
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
                RangeMap::<u8, BTreeSet<usize>>::arbitrary_given(
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
                Transition::<u8, BTreeSet<usize>>::arbitrary_given(
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
                <BTreeSet<usize> as Check<u8, BTreeSet<usize>>>::check(
                    &<BTreeSet<usize> as Ctrl<u8>>::arbitrary_given(
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
            if lhs.involves_any_fallback() || rhs.involves_any_fallback() {
                return true;
            }
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
            if !match (
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
            } {
                return false;
            }
            let Ok(symm) = panic::catch_unwind(|| rhs | lhs) else {
                return false;
            };
            if symm.check().is_err() {
                return false;
            }
            if symm.determinize().is_err() {
                return false;
            }
            union_accept == symm.accept(input.iter().copied())
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
            if lhs.involves_any_fallback() || rhs.involves_any_fallback() {
                return true;
            }
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

        fn star(d: Deterministic<u8>, input: Vec<u8>) -> bool {
            if d.involves_any_fallback() {
                return true;
            }
            if !splittable(&d, &input) {
                return true;
            }
            let Ok(star) = panic::catch_unwind(|| d.star()) else {
                return true;
            };
            star.accept(input).is_ok()
        }

        // TODO:
        /*
        fn star_star_identity(d: Deterministic<u8>, input: Vec<u8>) -> bool {
            let Ok(once) = panic::catch_unwind(|| d.star()) else {
                return true;
            };
            let twice = once.clone().star();
            once.accept(input.iter().copied()) == twice.accept(input)
        }
        */
    }
}

mod reduced {
    use super::*;
    use std::{
        collections::{BTreeMap, BTreeSet},
        panic,
    };

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

    fn union(lhs: Deterministic<u8>, rhs: Deterministic<u8>, input: Vec<u8>) {
        if lhs.involves_any_fallback() || rhs.involves_any_fallback() {
            return;
        }
        let Ok(union) = panic::catch_unwind(|| lhs.clone() | rhs.clone()) else {
            return;
        };
        union.check().unwrap();
        if union.determinize().is_err() {
            return;
        }
        {
            println!();
            println!("LHS:");
            let mut run = input.iter().copied().run(&lhs);
            println!("      {run:?}");
            while let Some(r) = run.next() {
                println!("{r:?} {run:?}");
            }
        }
        {
            println!();
            println!("RHS:");
            let mut run = input.iter().copied().run(&rhs);
            println!("      {run:?}");
            while let Some(r) = run.next() {
                println!("{r:?} {run:?}");
            }
        }
        {
            println!();
            println!("Union:");
            let mut run = input.iter().copied().run(&union);
            println!("      {run:?}");
            while let Some(r) = run.next() {
                println!("{r:?} {run:?}");
            }
        }
        let union_accept = union.accept(input.iter().copied());
        match (
            lhs.accept(input.iter().copied()),
            rhs.accept(input.iter().copied()),
        ) {
            (Ok(a), Ok(b)) => {
                if a == b {
                    assert_eq!(union_accept, Ok(a));
                } else {
                    assert!(matches!(union_accept, Err(ParseError::BadParser(..))));
                }
            }
            (Err(e), Ok(out)) | (Ok(out), Err(e)) => match e {
                ParseError::BadInput(..) => assert_eq!(union_accept, Ok(out)),
                ParseError::BadParser(..) => unreachable!(),
            },
            (Err(ParseError::BadParser(..)), Err(..))
            | (Err(..), Err(ParseError::BadParser(..))) => {
                assert!(matches!(union_accept, Err(ParseError::BadParser(..))));
            }
            (Err(ParseError::BadInput(..)), Err(ParseError::BadInput(..))) => {
                let _ = union_accept.as_ref().unwrap_err();
            }
        }
        let symm = rhs | lhs;
        symm.check().unwrap();
        drop(symm.determinize().unwrap());
        assert_eq!(union_accept, symm.accept(input));
    }

    fn shr(lhs: Deterministic<u8>, rhs: Deterministic<u8>, input: Vec<u8>) {
        if lhs.involves_any_fallback() || rhs.involves_any_fallback() {
            return;
        }
        let splittable = (0..=input.len()).any(|i| {
            lhs.accept(input[..i].iter().copied()).is_ok()
                && rhs.accept(input[i..].iter().copied()).is_ok()
        });
        let Ok(concat) = panic::catch_unwind(|| lhs >> rhs) else {
            return;
        };
        concat.check().unwrap();
        if concat.determinize().is_err() {
            return;
        }
        assert_eq!(concat.accept(input).is_ok(), splittable);
    }

    fn star(d: Deterministic<u8>, input: Vec<u8>) {
        if d.involves_any_fallback() {
            return;
        }
        if !splittable(&d, &input) {
            return;
        }
        let Ok(star) = panic::catch_unwind(|| d.star()) else {
            return;
        };
        println!("{star:?}");
        drop(star.accept(input).unwrap());
    }

    #[test]
    fn deterministic_implies_no_runtime_errors_1() {
        deterministic_implies_no_runtime_errors(
            &Graph {
                states: vec![State {
                    transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
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
                            update: None,
                        }),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Scrutinize {
                            filter: RangeMap(BTreeMap::new()),
                            fallback: None,
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                        non_accepting: iter::once(String::new()).collect(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Call {
                            region: "region",
                            detour: 0,
                            dst: Box::new(Transition::Lateral {
                                dst: 1,
                                update: None,
                            }),
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
            },
            vec![],
        );
    }

    #[test]
    fn union_1() {
        union(
            Graph {
                states: vec![State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(BTreeMap::new()),
                        fallback: Some(Transition::Lateral {
                            dst: 0,
                            update: None,
                        }),
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            Graph {
                states: vec![State {
                    transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            vec![0],
        );
    }

    #[test]
    fn union_2() {
        union(
            Graph {
                states: vec![State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(
                            iter::once((
                                Range { first: 0, last: 0 },
                                Transition::Return { region: "region" },
                            ))
                            .collect(),
                        ),
                        fallback: None,
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            Graph {
                states: vec![State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(BTreeMap::new()),
                        fallback: Some(Transition::Lateral {
                            dst: 0,
                            update: None,
                        }),
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            vec![0],
        );
    }

    #[test]
    fn shr_1() {
        shr(
            Graph {
                states: vec![State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(BTreeMap::new()),
                        fallback: Some(Transition::Lateral {
                            dst: 0,
                            update: None,
                        }),
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            Graph {
                states: vec![State {
                    transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            vec![0],
        );
    }

    #[test]
    fn shr_2() {
        shr(
            Graph {
                states: vec![State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(
                            iter::once((
                                Range { first: 0, last: 0 },
                                Transition::Return { region: "region" },
                            ))
                            .collect(),
                        ),
                        fallback: None,
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            Graph {
                states: vec![State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(BTreeMap::new()),
                        fallback: Some(Transition::Lateral {
                            dst: 0,
                            update: None,
                        }),
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
            },
            vec![0],
        );
    }

    #[test]
    fn star_1() {
        star(
            Graph {
                states: vec![
                    State {
                        transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Wildcard(Transition::Call {
                            region: "region",
                            detour: 0,
                            dst: Box::new(Transition::Lateral {
                                dst: 0,
                                update: None,
                            }),
                            combine: ff!(|(), ()| ()),
                        }),
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: 1,
            },
            vec![],
        );
    }

    #[test]
    fn star_2() {
        star(
            Graph {
                states: vec![
                    State {
                        transitions: Curry::Wildcard(Transition::Return { region: "region" }),
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: Curry::Scrutinize {
                            filter: RangeMap(BTreeMap::new()),
                            fallback: Some(Transition::Lateral {
                                dst: 0,
                                update: None,
                            }),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: 1,
            },
            vec![0, 0],
        );
    }
}

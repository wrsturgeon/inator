/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::integer_division,
    clippy::panic,
    clippy::print_stdout,
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::use_debug
)]

#[cfg(feature = "quickcheck")]
mod prop {
    use crate::*;
    use core::num::NonZeroUsize;
    use quickcheck::*;
    use std::{collections::BTreeSet, env};

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
            assert_eq!(
                Deterministic::<u8, u8, u8>::arbitrary(&mut Gen::new(curved.into())).check(),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_nondeterministic() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                Nondeterministic::<u8, u8, u8>::arbitrary(&mut Gen::new(curved.into())).check(),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_state() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                State::<u8, u8, u8, BTreeSet<usize>>::arbitrary_given(
                    curved,
                    &mut Gen::new(curved.into())
                )
                .check(curved),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_curry_stack() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                CurryStack::<u8, u8, u8, BTreeSet<usize>>::arbitrary_given(
                    curved,
                    &mut Gen::new(curved.into())
                )
                .check(curved),
                Ok(())
            );
        }
    }

    #[test]
    fn arbitrary_implies_check_curry_input() {
        let gs = gen_size();
        let tests = qc_tests();
        for size in 0..tests {
            let curved = nz(2.max((gs * size * size) / (tests * tests)));
            assert_eq!(
                CurryInput::<u8, u8, u8, BTreeSet<usize>>::arbitrary_given(
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
                RangeMap::<u8, u8, u8, BTreeSet<usize>>::arbitrary_given(
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
                Transition::<u8, u8, u8, BTreeSet<usize>>::arbitrary_given(
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
                <usize as Check<u8, u8, u8, usize>>::check(
                    &<usize as Ctrl<u8, u8, u8>>::arbitrary_given(
                        curved,
                        &mut Gen::new(curved.into())
                    ),
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
                <BTreeSet<usize> as Check<u8, u8, u8, BTreeSet<usize>>>::check(
                    &<BTreeSet<usize> as Ctrl<u8, u8, u8>>::arbitrary_given(
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
        //     nd: Nondeterministic<u8, u8, u8>,
        //     input: Vec<u8>
        // ) -> bool {
        //     !matches!(nd.accept(input), Err(ParseError::BadParser(..)))
        // }
        //
        // fn check_implies_determinize(nd: Nondeterministic<u8, u8, u8>) -> bool {
        //     nd.determinize().is_ok()
        // }

        fn determinize_implies_no_runtime_errors(
            nd: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            let Ok(d) = nd.determinize() else {
                return true; // irrelevant
            };
            !matches!(d.accept(input), Err(ParseError::BadParser(..)))
        }

        fn deterministic_implies_no_runtime_errors(
            d: Deterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            !matches!(d.accept(input), Err(ParseError::BadParser(..)))
        }

        fn determinize_identity(d: Deterministic<u8, u8, u8>, input: Vec<u8>) -> bool {
            d.determinize().unwrap().accept(input.iter().copied()) == d.accept(input)
        }

        fn union(
            lhs: Nondeterministic<u8, u8, u8>,
            rhs: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            if lhs.determinize().is_err() {
                return true;
            }
            if rhs.determinize().is_err() {
                return true;
            }
            let union = lhs.clone() | rhs.clone();
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
    }
}

mod reduced {
    use crate::*;
    use core::iter;
    use std::collections::BTreeMap;

    fn determinize_implies_no_runtime_errors(nd: &Nondeterministic<u8, u8, u8>, input: &[u8]) {
        if let Ok(d) = nd.determinize() {
            if let Err(ParseError::BadParser(e)) = d.accept(input.iter().copied()) {
                panic!("{e:?}");
            }
        }
    }

    #[test]
    fn determinize_implies_no_runtime_errors_1() {
        determinize_implies_no_runtime_errors(
            &Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(0).collect(),
                                act: Action::Local,
                                update: update!(|x, _| x),
                            })),
                            map_some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(0).collect(),
                                act: Action::Local,
                                update: update!(|x, _| x),
                            })),
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                ],
                initial: [1, 2].into_iter().collect(),
            },
            &[0],
        );
    }

    /*
    fn union(lhs: &Nondeterministic<u8, u8, u8>, rhs: &Nondeterministic<u8, u8, u8>, input: &[u8]) {
        if lhs.determinize().is_err() {
            return;
        }
        if rhs.determinize().is_err() {
            return;
        }
        println!("{lhs:?}");
        println!("union");
        println!("{rhs:?}");
        println!("yields");
        let union = lhs.clone() | rhs.clone();
        assert_eq!(union.check(), Ok(()));
        println!("{union:?}");
        println!();
        if union.determinize().is_err() {
            return;
        }
        let union_accept = union.accept(input.iter().copied());
        if let Err(ParseError::BadParser(err)) = union_accept {
            panic!("{err:?}");
        }
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
                assert!(
                    union_accept.is_err(),
                    "Neither original parser accepted on {input:?}, \
                    but the union yielded {union_accept:?}",
                );
            }
        }
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

    #[test]
    fn union_4() {
        union(
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: Some(CurryInput::Wildcard(Transition {
                            dst: iter::once(0).collect(),
                            act: Action::Local,
                            update: update!(|x, _| x),
                        })),
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: iter::once(0).collect(),
            },
            &Graph {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[0],
        );
    }

    #[test]
    fn union_5() {
        union(
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: true,
                }],
                initial: iter::once(0).collect(),
            },
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Wildcard(Transition {
                            dst: iter::once(0).collect(),
                            act: Action::Pop,
                            update: update!(|x, _| x),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: iter::once(0).collect(),
            },
            &[0],
        );
    }

    #[test]
    fn union_6() {
        union(
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: Some(CurryInput::Wildcard(Transition {
                            dst: iter::once(0).collect(),
                            act: Action::Local,
                            update: update!(|x, _| x),
                        })),
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: iter::once(0).collect(),
            },
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Wildcard(Transition {
                            dst: iter::once(0).collect(),
                            act: Action::Local,
                            update: update!(|x, _| x.saturating_add(1)),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: iter::once(0).collect(),
            },
            &[0],
        );
    }

    #[test]
    fn union_7() {
        union(
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Scrutinize(RangeMap {
                            entries: iter::once((
                                Range { first: 0, last: 0 },
                                Transition {
                                    dst: iter::once(0).collect(),
                                    act: Action::Local,
                                    update: update!(|x, _| x),
                                },
                            ))
                            .collect(),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: iter::once(0).collect(),
            },
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Scrutinize(RangeMap {
                            entries: iter::once((
                                Range { first: 0, last: 1 },
                                Transition {
                                    dst: iter::once(0).collect(),
                                    act: Action::Local,
                                    update: update!(|x: u8, _| x.saturating_add(1)),
                                },
                            ))
                            .collect(),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                }],
                initial: iter::once(0).collect(),
            },
            &[0],
        );
    }

    #[test]
    fn union_8() {
        union(
            &Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: iter::once((
                                0,
                                CurryInput::Wildcard(Transition {
                                    dst: iter::once(0).collect(),
                                    act: Action::Local,
                                    update: update!(|x, _| x),
                                }),
                            ))
                            .collect(),
                        },
                        accepting: false,
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(0).collect(),
                                act: Action::Local,
                                update: update!(|x, _| x),
                            })),
                            map_some: BTreeMap::new(),
                        },
                        accepting: false,
                    },
                ],
                initial: iter::once(2).collect(),
            },
            &Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Wildcard(Transition {
                            dst: iter::once(0).collect(),
                            act: Action::Local,
                            update: update!(|x, _| x.saturating_add(1)),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: true,
                }],
                initial: iter::once(0).collect(),
            },
            &[0],
        );
    }
    */
}

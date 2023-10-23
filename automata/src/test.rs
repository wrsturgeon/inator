/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::print_stdout,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::use_debug
)]

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

        fn procrustes_implies_check(nd: Nondeterministic<u8, u8, u8>) -> bool {
            let Some(graph) = nd.procrustes() else {
                return true;
            };
            graph.check() == Ok(())
        }

        // Important to claim that we're really testing *all possible* valid graphs.
        fn check_implies_procrustes_identity(nd: Nondeterministic<u8, u8, u8>) -> bool {
            if nd.check() == Ok(()) {
                nd.clone().procrustes() == Some(nd)
            } else {
                true
            }
        }

        // With discarding, this takes a ridiculously long time.
        fn check_implies_no_runtime_errors(
            nd: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            let Some(graph) = nd.procrustes() else {
                return true;
            };
            !matches!(graph.accept(input), Err(ParseError::BadParser(..)))
        }

        fn check_implies_determinize(nd: Nondeterministic<u8, u8, u8>) -> bool {
            let Some(graph) = nd.procrustes() else {
                return true;
            };
            graph.determinize().is_ok()
        }

        fn deterministic_implies_no_runtime_errors(
            d: Deterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            let Some(graph) = d.procrustes() else {
                return true;
            };
            !matches!(graph.accept(input), Err(ParseError::BadParser(..)))
        }

        fn determinize_identity(d: Deterministic<u8, u8, u8>) -> bool {
            let Some(graph) = d.procrustes() else {
                return true;
            };
            graph.determinize() == Ok(graph)
        }

        fn union(
            lhs: Nondeterministic<u8, u8, u8>,
            rhs: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            let Some(lhs) = lhs.procrustes() else {
                return true;
            };
            let Some(rhs) = rhs.procrustes() else {
                return true;
            };
            let union = lhs.clone() | rhs.clone();
            let union_accept = union.accept(input.iter().copied());
            lhs.accept(input.iter().copied())
                .or_else(|_| rhs.accept(input.iter().copied()))
                .map_or_else(
                    |e| match e {
                        reject @ ParseError::BadInput(..) => union_accept == Err(reject),
                        ParseError::BadParser(..) => false, // if all else holds, unreachable
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

        fn intersection(
            lhs: Nondeterministic<u8, u8, u8>,
            rhs: Nondeterministic<u8, u8, u8>,
            input: Vec<u8>
        ) -> bool {
            let Some(lhs) = lhs.procrustes() else {
                return true;
            };
            let Some(rhs) = rhs.procrustes() else {
                return true;
            };
            let intersection = lhs.clone() & rhs.clone();
            let intersection_accept = intersection.accept(input.iter().copied());
            lhs.accept(input.iter().copied())
                .and_then(|_| rhs.accept(input.iter().copied()))
                .map_or_else(
                    |e| match e {
                        reject @ ParseError::BadInput(..) => intersection_accept == Err(reject),
                        ParseError::BadParser(..) => false, // if all else holds, unreachable
                    },
                    |ok| intersection_accept == Ok(ok),
                )
        }

        fn intersection_preserves_ill_formedness(
            lhs: Nondeterministic<u8, u8, u8>,
            rhs: Nondeterministic<u8, u8, u8>
        ) -> bool {
            (lhs.check().is_err() || rhs.check().is_err()) == (lhs & rhs).check().is_err()
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

    fn determinize_identity(d: &Deterministic<u8, u8, u8>) {
        if d.check().is_err() {
            return;
        }
        assert_eq!(d.determinize().unwrap(), *d);
    }

    fn procrustes_implies_check(nd: Nondeterministic<u8, u8, u8>) {
        println!("{nd:?}");
        let Some(graph) = nd.procrustes() else {
            return;
        };
        println!("{graph:?}");
        graph.check().unwrap();
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
    fn determinize_identity_1() {
        determinize_identity(&Graph {
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
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
            ],
            initial: 0,
        });
    }

    #[test]
    fn procrustes_implies_check_01() {
        procrustes_implies_check(Graph {
            states: vec![],
            initial: iter::once(0).collect(),
        });
    }

    #[test]
    fn procrustes_implies_check_02() {
        procrustes_implies_check(Graph {
            states: vec![State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Wildcard(Transition {
                        dst: BTreeSet::new(),
                        act: Action::Local,
                        update: update!(|x, _| x),
                    })),
                    map_none: None,
                    map_some: BTreeMap::new(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_03() {
        procrustes_implies_check(Graph {
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
        });
    }

    #[test]
    fn procrustes_implies_check_04() {
        procrustes_implies_check(Graph {
            states: vec![State {
                transitions: CurryStack {
                    wildcard: None,
                    map_none: None,
                    map_some: iter::once((
                        0,
                        CurryInput::Scrutinize(RangeMap {
                            entries: iter::once(CmpFirst(
                                Range { first: 0, last: 0 },
                                Transition {
                                    dst: BTreeSet::new(),
                                    act: Action::Local,
                                    update: update!(|x, _| x),
                                },
                            ))
                            .collect(),
                        }),
                    ))
                    .collect(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_05() {
        procrustes_implies_check(Graph {
            states: vec![State {
                transitions: CurryStack {
                    wildcard: None,
                    map_none: None,
                    map_some: iter::once((
                        0,
                        CurryInput::Scrutinize(RangeMap {
                            entries: [
                                CmpFirst(
                                    Range { first: 0, last: 0 },
                                    Transition {
                                        dst: iter::once(0).collect(),
                                        act: Action::Local,
                                        update: update!(|x, _| x),
                                    },
                                ),
                                CmpFirst(
                                    Range { first: 0, last: 1 },
                                    Transition {
                                        dst: iter::once(0).collect(),
                                        act: Action::Local,
                                        update: update!(|x, _| x),
                                    },
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        }),
                    ))
                    .collect(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_06() {
        procrustes_implies_check(Graph {
            states: vec![State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Wildcard(Transition {
                        dst: iter::once(0).collect(),
                        act: Action::Local,
                        update: update!(|x, _| x),
                    })),
                    map_none: None,
                    map_some: iter::once((
                        0,
                        CurryInput::Scrutinize(RangeMap {
                            entries: iter::once(CmpFirst(
                                Range { first: 0, last: 0 },
                                Transition {
                                    dst: iter::once(0).collect(),
                                    act: Action::Local,
                                    update: update!(|x: u8, _| x.saturating_add(1)),
                                },
                            ))
                            .collect(),
                        }),
                    ))
                    .collect(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_07() {
        procrustes_implies_check(Graph {
            states: vec![State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Wildcard(Transition {
                        dst: iter::once(0).collect(),
                        act: Action::Local,
                        update: update!(|x, _| x),
                    })),
                    map_none: None,
                    map_some: iter::once((
                        0,
                        CurryInput::Scrutinize(RangeMap {
                            entries: [
                                CmpFirst(
                                    Range { first: 0, last: 0 },
                                    Transition {
                                        dst: iter::once(0).collect(),
                                        act: Action::Local,
                                        update: update!(|x, _| x),
                                    },
                                ),
                                CmpFirst(
                                    Range { first: 1, last: 1 },
                                    Transition {
                                        dst: iter::once(0).collect(),
                                        act: Action::Local,
                                        update: update!(|x, _| x),
                                    },
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        }),
                    ))
                    .collect(),
                },
                accepting: false,
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_08() {
        procrustes_implies_check(Graph {
            states: vec![
                State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Wildcard(Transition {
                            dst: BTreeSet::new(),
                            act: Action::Local,
                            update: update!(|x, _| x),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
                State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Wildcard(Transition {
                            dst: BTreeSet::new(),
                            act: Action::Local,
                            update: update!(|x, _| x),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
            ],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_09() {
        procrustes_implies_check(Graph {
            states: vec![State {
                transitions: CurryStack {
                    wildcard: Some(CurryInput::Scrutinize(RangeMap {
                        entries: iter::once(CmpFirst(
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
            }],
            initial: BTreeSet::new(),
        });
    }

    #[test]
    fn procrustes_implies_check_10() {
        procrustes_implies_check(Graph {
            states: vec![
                State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Scrutinize(RangeMap {
                            entries: BTreeSet::new(),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
                State {
                    transitions: CurryStack {
                        wildcard: Some(CurryInput::Wildcard(Transition {
                            dst: BTreeSet::new(),
                            act: Action::Local,
                            update: update!(|x, _| x),
                        })),
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
            ],
            initial: iter::once(0).collect(),
        });
    }

    #[test]
    fn procrustes_implies_check_11() {
        procrustes_implies_check(Graph {
            states: vec![
                State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: Some(CurryInput::Scrutinize(RangeMap {
                            entries: iter::once(CmpFirst(
                                Range { first: 0, last: 0 },
                                Transition {
                                    dst: BTreeSet::new(),
                                    act: Action::Local,
                                    update: update!(|x, _| x),
                                },
                            ))
                            .collect(),
                        })),
                        map_some: BTreeMap::new(),
                    },
                    accepting: false,
                },
                State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: Some(CurryInput::Scrutinize(RangeMap {
                            entries: BTreeSet::new(),
                        })),
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
                },
            ],
            initial: BTreeSet::new(),
        });
    }
}

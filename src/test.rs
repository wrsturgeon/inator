/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::absolute_paths,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use crate::*;

/// Check if we can split this input into a bunch of non-zero-sized slices
/// that are all individually accepted by a given parser.
#[inline]
fn sliceable<I: Input, S: Stack, C: Ctrl<I, S>>(parser: &Graph<I, S, C>, input: &[I]) -> bool {
    input.is_empty()
        || (1..=input.len()).rev().any(|i| {
            parser.accept(input[..i].iter().cloned()).is_ok() && sliceable(parser, &input[i..])
        })
}

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;
    use std::panic;

    quickcheck! {
        fn empty_works(input: Vec<u8>) -> bool {
            let parser = empty::<u8, u8>();
            if parser.check().is_err() { return false; }
            input.is_empty() == empty::<u8, u8>().accept(input).is_ok()
        }

        fn any_of_works(range: Range<u8>, input: Vec<u8>) -> bool {
            let parser = any_of::<_, ()>(range, update!(|(), _| {}));
            if parser.check().is_err() { return false; }
            parser.accept(input.iter().copied()).is_ok() == (input.len() == 1 && range.contains(&input[0]))
        }

        fn fixpoint_unused(parser: Deterministic<u8, u8>, input: Vec<u8>) -> bool {
            let pre = parser.accept(input.iter().copied());
            let post = (fixpoint("unused") >> parser).accept(input);
            pre == post
        }

        fn fixpoint_repeat(parser: Deterministic<u8, u8>, both: Vec<u8>) -> bool {
            if parser.check().is_err() {
                return true; // not our problem
            }
            if parser.accept(iter::empty()).is_err() {
                return true;
            }
            let sliceable = sliceable(&parser, &both);
            let Ok(repeated) = panic::catch_unwind(|| fixpoint("da capo") >> parser >> recurse("da capo")) else {
                return true;
            };
            if repeated.check().is_err() {
                return false;
            }
            let output = repeated.accept(both);
            if matches!(output, Err(ParseError::BadParser(_))) {
                return true;
            }
            output.is_ok() == sliceable
        }

        /*
        fn fixpoint_repeat_twice(lhs: Deterministic<u8, u8>, rhs: Deterministic<u8, u8>, both: Vec<u8>) -> bool {
            if lhs.accept(iter::empty()).is_err() || rhs.accept(iter::empty()).is_err() {
                return true;
            }
            let sliceable = {
                let Ok(parser) = panic::catch_unwind(|| lhs.clone() >> rhs.clone()) else {
                    return true;
                };
                sliceable(&parser, &both)
            };
            let repeated = fixpoint("da capo") >> lhs >> rhs >> recurse("da capo");
            if repeated.check().is_err() {
                return false;
            }
            let output = repeated.accept(both);
            if matches!(output, Err(ParseError::BadParser(_))) {
                return true;
            }
            output.is_ok() == sliceable
        }
        */
    }
}

mod reduced {
    #![allow(clippy::print_stdout, clippy::use_debug)]

    use super::*;
    use std::panic;

    fn fixpoint_unused(parser: Deterministic<u8, u8>, input: Vec<u8>) {
        let pre = parser.accept(input.iter().copied());
        let post = (fixpoint("unused") >> parser).accept(input);
        assert_eq!(pre, post);
    }

    fn fixpoint_repeat(parser: Deterministic<u8, u8>, both: Vec<u8>) {
        parser.check().unwrap();
        if parser.accept(iter::empty()).is_err() {
            return;
        }
        let sliceable = sliceable(&parser, &both);
        let Ok(repeated) =
            panic::catch_unwind(|| fixpoint("da capo") >> parser >> recurse("da capo"))
        else {
            return;
        };
        println!("Repeated: {repeated:#?}");
        repeated.check().unwrap();
        let mut run = both.iter().copied().run(&repeated);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            println!("{r:?} {run:?}");
        }
        let output = repeated.accept(both);
        if matches!(output, Err(ParseError::BadParser(_))) {
            return;
        }
        assert_eq!(output.is_ok(), sliceable, "{output:?}");
    }

    /*
    fn fixpoint_repeat_twice(
        lhs: Deterministic<u8, u8>,
        rhs: Deterministic<u8, u8>,
        both: Vec<u8>,
    ) {
        lhs.check().unwrap();
        rhs.check().unwrap();
        if lhs.accept(iter::empty()).is_err() || rhs.accept(iter::empty()).is_err() {
            return;
        }
        let sliceable = {
            let Ok(parser) = panic::catch_unwind(|| lhs.clone() >> rhs.clone()) else {
                return;
            };
            sliceable(&parser, &both)
        };
        let repeated = fixpoint("da capo") >> lhs >> rhs >> recurse("da capo");
        println!("Repeated: {repeated:#?}");
        repeated.check().unwrap();
        let mut run = both.iter().copied().run(&repeated);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            println!("{r:?} {run:?}");
        }
        let output = repeated.accept(both);
        if matches!(output, Err(ParseError::BadParser(_))) {
            return;
        }
        assert_eq!(output.is_ok(), sliceable, "{output:?}");
    }
    */

    #[test]
    #[allow(clippy::too_many_lines)]
    fn fixpoint_unused_1() {
        fixpoint_unused(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: [
                                (
                                    65,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: iter::once((
                                            Range {
                                                first: 61,
                                                last: 227,
                                            },
                                            Transition {
                                                dst: 2,
                                                act: Action::Local,
                                                update: update!(|(), _| {}),
                                            },
                                        ))
                                        .collect(),
                                    }),
                                ),
                                (
                                    140,
                                    CurryInput::Wildcard(Transition {
                                        dst: 0,
                                        act: Action::Local,
                                        update: update!(|(), _| {}),
                                    }),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        },
                        non_accepting: [
                            "\u{19}",
                            "%\u{9d}\u{f}#\u{fff8}",
                            "1\u{8771}%\u{17}",
                            "\u{a4}+\u{b372}",
                        ]
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: 0,
                                act: Action::Pop,
                                update: update!(|(), _| {}),
                            })),
                            map_none: Some(CurryInput::Scrutinize(RangeMap {
                                entries: BTreeMap::new(),
                            })),
                            map_some: [
                                (
                                    18,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    71,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    76,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    116,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    164,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        },
                        non_accepting: ["", "#", "-\u{80}0", "["]
                            .into_iter()
                            .map(str::to_owned)
                            .collect(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: 1,
                                act: Action::Pop,
                                update: update!(|(), _| {}),
                            })),
                            map_none: None,
                            map_some: [
                                (
                                    71,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    164,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    205,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    213,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                                (
                                    220,
                                    CurryInput::Scrutinize(RangeMap {
                                        entries: BTreeMap::new(),
                                    }),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: 0,
                tags: [
                    ("!/", 0),
                    ("`\u{602}\u{206d}", 0),
                    ("m\u{5920}\u{99}\u{61c}", 0),
                    ("u\u{8b}", 1),
                    ("\u{c4c8}\u{8a}\u{98}\u{e}", 4),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
            },
            vec![],
        );
    }

    #[test]
    fn fixpoint_repeat_1() {
        fixpoint_repeat(
            Nondeterministic {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(Ok(0)).collect(),
                                act: Action::Local,
                                update: update!(|(), _| {}),
                            })),
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: [Ok(0), Ok(1)].into_iter().collect(),
                tags: BTreeMap::new(),
            }
            .determinize()
            .unwrap(),
            vec![0],
        );
    }

    #[test]
    fn fixpoint_repeat_2() {
        fixpoint_repeat(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: 0,
                                act: Action::Local,
                                update: update!(|(), _| {}),
                            })),
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: 1,
                tags: BTreeMap::new(),
            },
            vec![0, 0, 0],
        );
    }

    #[test]
    fn fixpoint_repeat_3() {
        fixpoint_repeat(
            Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    non_accepting: BTreeSet::new(),
                }],
                initial: 0,
                tags: BTreeMap::new(),
            },
            vec![0],
        );
    }

    #[test]
    fn fixpoint_repeat_4() {
        fixpoint_repeat(
            Nondeterministic {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(Ok(1)).collect(),
                                act: Action::Local,
                                update: update!(|(), _| {}),
                            })),
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: [Ok(1), Ok(2)].into_iter().collect(),
                                act: Action::Local,
                                update: update!(|(), _| {}),
                            })),
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: iter::once(Ok(0)).collect(),
                tags: BTreeMap::new(),
            }
            .determinize()
            .unwrap(),
            vec![],
        );
    }

    #[test]
    fn fixpoint_repeat_6() {
        fixpoint_repeat(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: iter::once(String::new()).collect(),
                    },
                ],
                initial: 0,
                tags: iter::once((String::new(), 1)).collect(),
            },
            vec![],
        );
    }

    #[test]
    fn fixpoint_repeat_7() {
        fixpoint_repeat(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: Some(CurryInput::Wildcard(Transition {
                                dst: 0,
                                act: Action::Local,
                                update: update!(|(), _| {}),
                            })),
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: Some(CurryInput::Wildcard(Transition {
                                dst: 0,
                                act: Action::Pop,
                                update: update!(|(), _| {}),
                            })),
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                    },
                ],
                initial: 1,
                tags: BTreeMap::new(),
            },
            vec![],
        );
    }
}

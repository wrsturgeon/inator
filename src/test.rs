/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::use_debug
)]

use crate::*;

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;

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

        fn fixpoint_unused(parser: Nondeterministic<u8, u8>, input: Vec<u8>) -> bool {
            let pre = parser.accept(input.iter().copied());
            let post = (fixpoint("unused") >> parser).accept(input);
            pre == post
        }

        fn kleene_star(parser: Nondeterministic<u8, u8>, both: Vec<u8>) -> bool {
            parser.check().unwrap();
            if parser.accept(iter::empty()).is_err() {
                return true;
            }
            let sliceable = sliceable(&parser, &both);
            let repeated = fixpoint("da capo") >> parser >> recurse("da capo");
            if repeated.check().is_err() {
                return false;
            }
            let output = repeated.accept(both);
            if matches!(output, Err(ParseError::BadParser(_))) {
                return true;
            }
            output.is_ok() == sliceable
        }
    }
}

mod reduced {
    use super::*;

    fn kleene_star(parser: Nondeterministic<u8, u8>, both: Vec<u8>) {
        parser.check().unwrap();
        if parser.accept(iter::empty()).is_err() {
            return;
        }
        let sliceable = sliceable(&parser, &both);
        let repeated = fixpoint("da capo") >> parser >> recurse("da capo");
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

    #[test]
    fn kleene_star_1() {
        kleene_star(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
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
                        tags: BTreeSet::new(),
                    },
                ],
                initial: [Ok(0), Ok(1)].into_iter().collect(),
            },
            vec![0],
        );
    }

    #[test]
    fn kleene_star_2() {
        kleene_star(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: iter::once(String::new()).collect(),
                    },
                ],
                initial: iter::once(Ok(0)).collect(),
            },
            vec![],
        );
    }

    #[test]
    fn kleene_star_3() {
        kleene_star(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
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
                        tags: BTreeSet::new(),
                    },
                ],
                initial: iter::once(Ok(1)).collect(),
            },
            vec![0, 0, 0],
        );
    }

    #[test]
    fn kleene_star_4() {
        kleene_star(
            Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    non_accepting: BTreeSet::new(),
                    tags: BTreeSet::new(),
                }],
                initial: iter::once(Ok(0)).collect(),
            },
            vec![0],
        );
    }

    #[test]
    fn kleene_star_5() {
        kleene_star(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
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
                        tags: BTreeSet::new(),
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
                        tags: BTreeSet::new(),
                    },
                ],
                initial: iter::once(Ok(0)).collect(),
            },
            vec![],
        );
    }

    #[test]
    fn kleene_star_6() {
        kleene_star(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(Ok(2)).collect(),
                                act: Action::Local,
                                update: update!(|(), _| {}),
                            })),
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: Some(CurryInput::Scrutinize(RangeMap {
                                entries: BTreeMap::new(),
                            })),
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: Some(CurryInput::Wildcard(Transition {
                                dst: iter::once(Ok(0)).collect(),
                                act: Action::Pop,
                                update: update!(|(), _| {}),
                            })),
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: BTreeSet::new(),
                        tags: BTreeSet::new(),
                    },
                ],
                initial: iter::once(Ok(0)).collect(),
            },
            vec![0, 0],
        );
    }
}

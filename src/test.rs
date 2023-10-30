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

#[cfg(feature = "quickcheck")]
mod prop {
    use crate::*;
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

        fn fixpoint_repeat(parser: Nondeterministic<u8, u8>, both: Vec<u8>) -> bool {
            parser.check().unwrap();
            if parser.accept(iter::empty()).is_err() {
                return true;
            }
            // Find an index such that, if we look only at input before that index,
            // we can split that subset of input into two further subsets such that
            // the original parser accepts each individual subset.
            let splittable = (0..=both.len()).fold(None, |acc, i| {
                acc.or_else(|| {
                    (i..=both.len()).fold(None, |accc, k| {
                        accc.or_else(|| {
                            (parser.accept(both[..i].iter().copied()).is_ok()
                                && parser.accept(both[i..k].iter().copied()).is_ok())
                            .then_some(k)
                        })
                    })
                })
            });
            let repeated = fixpoint("da capo") >> parser >> recurse("da capo");
            if repeated.check().is_err() {
                return false;
            }
            if let Some(endpoint) = splittable {
                let output = repeated.accept(both[..endpoint].iter().copied());
                output.is_ok()
            } else {
                let output = repeated.accept(both);
                output.is_err()
            }
        }
    }
}

mod reduced {
    use crate::*;

    fn fixpoint_repeat(parser: Nondeterministic<u8, u8>, both: Vec<u8>) {
        parser.check().unwrap();
        if parser.accept(iter::empty()).is_err() {
            return;
        }
        // Find an index such that, if we look only at input before that index,
        // we can split that subset of input into two further subsets such that
        // the original parser accepts each individual subset.
        let splittable = (0..=both.len()).fold(None, |acc, i| {
            acc.or_else(|| {
                (i..=both.len()).fold(None, |accc, k| {
                    accc.or_else(|| {
                        (parser.accept(both[..i].iter().copied()).is_ok()
                            && parser.accept(both[i..k].iter().copied()).is_ok())
                        .then_some(k)
                    })
                })
            })
        });
        let repeated = fixpoint("da capo") >> parser >> recurse("da capo");
        println!("Repeated: {repeated:#?}");
        repeated.check().unwrap();
        let mut run = both.iter().copied().run(&repeated);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            println!("{r:?} {run:?}");
        }
        if let Some(endpoint) = splittable {
            let output = repeated.accept(both[..endpoint].iter().copied());
            assert!(output.is_ok(), "{output:?}");
        } else {
            let output = repeated.accept(both);
            assert!(output.is_err(), "{output:?}");
        }
    }

    #[test]
    fn fixpoint_repeat_1() {
        fixpoint_repeat(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: vec![],
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
                        non_accepting: vec![],
                        tags: BTreeSet::new(),
                    },
                ],
                initial: [Ok(0), Ok(1)].into_iter().collect(),
            },
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
                        non_accepting: vec![],
                        tags: BTreeSet::new(),
                    },
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: vec![],
                        tags: iter::once(String::new()).collect(),
                    },
                ],
                initial: iter::once(Ok(0)).collect(),
            },
            vec![],
        );
    }

    #[test]
    fn fixpoint_repeat_3() {
        fixpoint_repeat(
            Graph {
                states: vec![
                    State {
                        transitions: CurryStack {
                            wildcard: None,
                            map_none: None,
                            map_some: BTreeMap::new(),
                        },
                        non_accepting: vec![],
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
                        non_accepting: vec![],
                        tags: BTreeSet::new(),
                    },
                ],
                initial: iter::once(Ok(1)).collect(),
            },
            vec![0, 0, 0],
        );
    }

    #[test]
    fn fixpoint_repeat_4() {
        fixpoint_repeat(
            Graph {
                states: vec![State {
                    transitions: CurryStack {
                        wildcard: None,
                        map_none: None,
                        map_some: BTreeMap::new(),
                    },
                    non_accepting: vec![],
                    tags: BTreeSet::new(),
                }],
                initial: iter::once(Ok(0)).collect(),
            },
            vec![0],
        );
    }
}

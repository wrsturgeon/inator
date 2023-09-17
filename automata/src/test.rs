/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::print_stdout)]

use crate::*;
use std::collections::{BTreeMap, BTreeSet};

mod prop {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "quickcheck")]
    quickcheck::quickcheck! {
        fn nfa_dfa_equal(nfa: Nfa<u8>, input: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if input.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let dfa = Dfa::from(nfa.clone());
            quickcheck::TestResult::from_bool(
                input
                    .into_iter()
                    .all(|v| nfa.accept(v.iter().copied()) == dfa.accept(v)),
            )
        }

        fn dfa_nfa_equal(dfa: Dfa<u8>, input: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if input.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let nfa = Nfa::from(dfa.clone());
            quickcheck::TestResult::from_bool(
                input
                    .into_iter()
                    .all(|v| dfa.accept(v.iter().copied()) == nfa.accept(v)),
            )
        }

        fn nfa_dfa_one_and_a_half_roundtrip(nfa: Nfa<u8>) -> bool {
            let dfa = Dfa::from(nfa);
            Dfa::from(Nfa::from(dfa.clone())) == dfa
        }

        fn dfa_nfa_double_roundtrip(dfa: Dfa<u8>) -> bool {
            let once = Dfa::from(Nfa::from(dfa));
            Dfa::from(Nfa::from(once.clone())) == once
        }

        fn brzozowski(nfa: Nfa<u8>, input: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if input.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let dfa = nfa.clone().minimize();
            quickcheck::TestResult::from_bool(
                input
                    .into_iter()
                    .all(|v| nfa.accept(v.iter().copied()) == dfa.accept(v)),
            )
        }

        fn brzozowski_reduces_size(nfa: Nfa<u8>) -> quickcheck::TestResult {
            let nfa_size = nfa.size();
            match nfa.minimize().size().cmp(&nfa_size) {
                core::cmp::Ordering::Greater => quickcheck::TestResult::failed(),
                core::cmp::Ordering::Equal => quickcheck::TestResult::discard(),
                core::cmp::Ordering::Less => quickcheck::TestResult::passed(),
            }
        }

        fn unit(singleton: u8, reject: Vec<u8>) -> quickcheck::TestResult {
            let accept = vec![singleton];
            if reject == accept {
                return quickcheck::TestResult::discard();
            }
            let nfa = Nfa::unit(singleton);
            quickcheck::TestResult::from_bool(nfa.accept(accept) && !nfa.accept(reject))
        }
    }
}

mod prop_reduced {
    use super::*;

    fn nfa_dfa_equal(nfa: &Nfa<u8>, input: Vec<Vec<u8>>) {
        println!("NFA:");
        println!("{nfa}");
        let dfa = Dfa::from(nfa.clone());
        println!("DFA:");
        println!("{dfa}");
        for string in input {
            let nfa_accepted = nfa.accept(string.iter().copied());
            let dfa_accepted = dfa.accept(string.iter().copied());
            assert_eq!(
                nfa_accepted,
                dfa_accepted,
                "On input {string:?}, the NFA {} but the DFA {}",
                if nfa_accepted {
                    "accepted"
                } else {
                    "did not accept"
                },
                if dfa_accepted {
                    "accepted"
                } else {
                    "did not accept"
                },
            );
        }
    }

    fn brzozowski(nfa: &Nfa<u8>, input: Vec<Vec<u8>>) {
        if input.is_empty() {
            return;
        }
        println!("NFA:");
        println!("{nfa}");
        let dfa = nfa.clone().minimize();
        println!("DFA:");
        println!("{dfa}");
        for string in input {
            let nfa_accepted = nfa.accept(string.iter().copied());
            let dfa_accepted = dfa.accept(string.iter().copied());
            assert_eq!(
                nfa_accepted,
                dfa_accepted,
                "On input {string:?}, the NFA {} but the DFA {}",
                if nfa_accepted {
                    "accepted"
                } else {
                    "did not accept"
                },
                if dfa_accepted {
                    "accepted"
                } else {
                    "did not accept"
                },
            );
        }
    }

    #[test]
    fn nfa_dfa_equal_1() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((0, BTreeSet::new())).collect(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
            },
            vec![vec![]],
        );
    }

    #[test]
    fn nfa_dfa_equal_2() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: core::iter::once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once(0).collect(),
            },
            vec![vec![]],
        );
    }

    #[test]
    fn nfa_dfa_equal_3() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((255, BTreeSet::new())).collect(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
            },
            vec![vec![255]],
        );
    }

    #[test]
    fn nfa_dfa_equal_4() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: core::iter::once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((255, core::iter::once(0).collect()))
                            .collect(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once(0).collect(),
            },
            vec![vec![255]],
        );
    }

    #[test]
    fn nfa_dfa_equal_5() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once(1).collect(),
            },
            vec![vec![]],
        );
    }

    #[test]
    fn brzozowski_1() {
        brzozowski(
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            vec![vec![]],
        );
    }

    #[test]
    fn brzozowski_2() {
        brzozowski(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: false,
                }],
                initial: core::iter::once(0).collect(),
            },
            vec![vec![]],
        );
    }

    #[test]
    fn brzozowski_3() {
        brzozowski(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((0, core::iter::once(0).collect())).collect(),
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            vec![vec![]],
        );
    }
}

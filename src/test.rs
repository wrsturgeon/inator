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
        fn nfa_dfa_equal(nfa: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let dfa = nfa.clone().subsets();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| nfa.accept(input.iter().copied()) == dfa.accept(input)),
            )
        }

        fn dfa_nfa_equal(dfa: Dfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let nfa = Nfa::from(dfa.clone());
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| dfa.accept(input.iter().copied()) == nfa.accept(input)),
            )
        }

        fn nfa_dfa_one_and_a_half_roundtrip(nfa: Nfa<u8>) -> bool {
            let dfa = nfa.subsets();
            Nfa::from(dfa.clone()).subsets() == dfa
        }

        fn dfa_nfa_double_roundtrip(dfa: Dfa<u8>) -> bool {
            let once = Nfa::from(dfa).subsets();
            Nfa::from(once.clone()).subsets() == once
        }

        fn brzozowski(nfa: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let dfa = nfa.clone().compile();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| nfa.accept(input.iter().copied()) == dfa.accept(input)),
            )
        }

        fn brzozowski_reduces_size(nfa: Nfa<u8>) -> quickcheck::TestResult {
            let nfa_size = nfa.size();
            match nfa.compile().size().cmp(&nfa_size) {
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

        fn bitor(lhs: Nfa<u8>, rhs: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let fused = lhs.clone() | rhs.clone();
            quickcheck::TestResult::from_bool(inputs.into_iter().all(|input| {
                fused.accept(input.iter().copied())
                    == (lhs.accept(input.iter().copied()) || rhs.accept(input))
            }))
        }

        fn bitand(lhs: Nfa<u8>, rhs: Nfa<u8>, inputs: Vec<(Vec<u8>, Vec<u8>)>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let fused = lhs.clone() & rhs.clone();
            quickcheck::TestResult::from_bool(inputs.into_iter().all(|(input_l, input_r)| {
                fused.accept(input_l.iter().chain(&input_r).copied())
                    == (lhs.accept(input_l.iter().copied()) && rhs.accept(input_r))
            }))
        }
    }
}

mod prop_reduced {
    use super::*;

    fn nfa_dfa_equal(nfa: &Nfa<u8>, input: &Vec<u8>) {
        println!("NFA:");
        println!("{nfa}");
        let dfa = nfa.clone().subsets();
        println!("DFA:");
        println!("{dfa}");
        let nfa_accepted = nfa.accept(input.iter().copied());
        let dfa_accepted = dfa.accept(input.iter().copied());
        assert_eq!(
            nfa_accepted,
            dfa_accepted,
            "On input {input:?}, the NFA {} but the DFA {}",
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

    fn brzozowski(nfa: &Nfa<u8>, input: &Vec<u8>) {
        println!("NFA:");
        println!("{nfa}");
        let dfa = nfa.clone().compile();
        println!("DFA:");
        println!("{dfa}");
        let nfa_accepted = nfa.accept(input.iter().copied());
        let dfa_accepted = dfa.accept(input.iter().copied());
        assert_eq!(
            nfa_accepted,
            dfa_accepted,
            "On input {input:?}, the NFA {} but the DFA {}",
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

    fn bitor(lhs: &Nfa<u8>, rhs: &Nfa<u8>, input: &Vec<u8>) {
        println!("LHS:");
        println!("{lhs}");
        println!("RHS:");
        println!("{rhs}");
        let fused = lhs.clone() | rhs.clone();
        println!("Fused:");
        println!("{fused}");
        let lhs_accepted = lhs.accept(input.iter().copied());
        let rhs_accepted = rhs.accept(input.iter().copied());
        let fused_accepted = fused.accept(input.iter().copied());
        assert_eq!(
            fused_accepted,
            lhs_accepted || rhs_accepted,
            "On input {input:?}, the LHS {} and the RHS {} but the fused NFA {}",
            if lhs_accepted {
                "accepted"
            } else {
                "did not accept"
            },
            if rhs_accepted {
                "accepted"
            } else {
                "did not accept"
            },
            if fused_accepted {
                "accepted"
            } else {
                "did not accept"
            },
        );
    }

    fn bitand(lhs: &Nfa<u8>, rhs: &Nfa<u8>, input_l: &Vec<u8>, input_r: &Vec<u8>) {
        println!("LHS:");
        println!("{lhs}");
        println!("RHS:");
        println!("{rhs}");
        let fused = lhs.clone() & rhs.clone();
        println!("Fused:");
        println!("{fused}");
        let lhs_accepted = lhs.accept(input_l.iter().copied());
        let rhs_accepted = rhs.accept(input_r.iter().copied());
        let fused_accepted = fused.accept(input_l.iter().chain(input_r).copied());
        assert_eq!(
            fused_accepted,
            lhs_accepted && rhs_accepted,
            "On inputs {input_l:?} and {input_r:?}, \
            the LHS {} and the RHS {} but the fused NFA {}",
            if lhs_accepted {
                "accepted"
            } else {
                "did not accept"
            },
            if rhs_accepted {
                "accepted"
            } else {
                "did not accept"
            },
            if fused_accepted {
                "accepted"
            } else {
                "did not accept"
            },
        );
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
            &vec![],
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
            &vec![],
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
            &vec![255],
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
            &vec![255],
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
            &vec![],
        );
    }

    #[test]
    fn brzozowski_1() {
        brzozowski(
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &vec![],
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
            &vec![],
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
            &vec![],
        );
    }

    #[test]
    fn bitor_1() {
        bitor(
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
            },
            &vec![],
        );
    }

    #[test]
    fn bitor_2() {
        bitor(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: false,
                }],
                initial: core::iter::once(0).collect(),
            },
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &vec![],
        );
    }

    #[test]
    fn bitor_3() {
        bitor(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((72, core::iter::once(0).collect())).collect(),
                    accepting: false,
                }],
                initial: core::iter::once(0).collect(),
            },
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
            },
            &vec![72],
        );
    }

    #[test]
    fn bitand_1() {
        bitand(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
            },
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &vec![],
            &vec![],
        );
    }
}

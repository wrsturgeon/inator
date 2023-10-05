/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::use_debug)]

use crate::{Compiled as Dfa, Parser as Nfa, *};
use core::iter::once;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "quickcheck")]
use core::cmp::Ordering;

mod unit {
    use super::*;

    #[test]
    fn zero_state_nfa_subsets() {
        let nfa = Nfa::<()>::void();
        let dfa = nfa.subsets();
        assert_eq!(
            dfa,
            Dfa {
                states: vec![dfa::State {
                    transitions: BTreeMap::new(),
                    accepting: false
                }],
                initial: 0
            }
        );
    }

    #[test]
    fn zero_state_nfa_fuzz() {
        let _ = Nfa::<()>::void().fuzz().unwrap_err();
    }

    #[test]
    fn zero_initial_nfa_fuzz() {
        let _ = Nfa::<()> {
            states: vec![nfa::State {
                epsilon: BTreeSet::new(),
                non_epsilon: BTreeMap::new(),
                accepting: true,
            }],
            initial: BTreeSet::new(),
        }
        .fuzz()
        .unwrap_err();
    }

    #[test]
    fn isolated_state_nfa_fuzz() {
        let _ = Nfa::<()> {
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
            initial: once(0).collect(),
        }
        .fuzz()
        .unwrap_err();
    }

    #[test]
    #[should_panic]
    fn ambiguity() {
        let parser =
            (ignore('a') >> on('a', "aa")) | (ignore('a') >> ignore('a') >> on('b', "aab"));
        drop(parser.compile());
    }
}

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
            let nfa =dfa.generalize();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| dfa.accept(input.iter().copied()) == nfa.accept(input)),
            )
        }

        fn nfa_dfa_one_and_a_half_roundtrip(nfa: Nfa<u8>) -> bool {
            let dfa = nfa.subsets();
            dfa.generalize().subsets() == dfa
        }

        fn dfa_nfa_double_roundtrip(dfa: Dfa<u8>) -> bool {
            let once = dfa.generalize().subsets();
            once.generalize().subsets() == once
        }

        fn brzozowski(nfa: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let dfa = nfa.compile();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| nfa.accept(input.iter().copied()) == dfa.accept(input)),
            )
        }

        fn brzozowski_reduces_size(dfa: Dfa<u8>) -> quickcheck::TestResult {
            let orig_size = dfa.size();
            match dfa.generalize().compile().size().cmp(&orig_size) {
                Ordering::Greater => quickcheck::TestResult::failed(),
                Ordering::Equal => quickcheck::TestResult::discard(),
                Ordering::Less => quickcheck::TestResult::passed(),
            }
        }

        fn unit(singleton: u8, reject: Vec<u8>) -> quickcheck::TestResult {
            let accept = vec![singleton];
            if reject == accept {
                return quickcheck::TestResult::discard();
            }
            let nfa = Nfa::unit(singleton, None);
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

        #[allow(clippy::arithmetic_side_effects)]
        fn shr(lhs: Nfa<u8>, rhs: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let fused = lhs.clone() >> rhs.clone();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| fused.accept(&input) == nfa::chain(&lhs, &rhs, &input)),
            )
        }

        fn fuzz_roundtrip(nfa: Nfa<u8>) -> quickcheck::TestResult {
            nfa.fuzz()
                .map_or(quickcheck::TestResult::discard(), |fuzz| {
                    quickcheck::TestResult::from_bool(fuzz.take(100).all(|input| nfa.accept(input)))
                })
        }

        fn repeat_fuzz_chain(nfa: Nfa<u8>) -> quickcheck::TestResult {
            let Ok(mut fuzzer) = nfa.fuzz() else {
                return quickcheck::TestResult::discard();
            };
            let repeated = nfa.repeat();
            #[allow(clippy::default_numeric_fallback)]
            for _ in 0..100 {
                let fst = fuzzer.next().unwrap();
                let snd = fuzzer.next().unwrap();
                if !repeated.accept(fst.into_iter().chain(snd)) {
                    return quickcheck::TestResult::failed();
                }
            }
            quickcheck::TestResult::passed()
        }

        fn star_def_swap_eq(nfa: Nfa<u8>) -> bool {
            let lhs = nfa.repeat().optional();
            let rhs = nfa.optional().repeat();
            for (il, ir) in lhs.fuzz().unwrap().zip(rhs.fuzz().unwrap()).take(100) {
                if !rhs.accept(il) { return false; }
                if !lhs.accept(ir) { return false; }
            }
            true
        }

        fn sandwich(a: Nfa<u8>, b: Nfa<u8>, c: Nfa<u8>) -> quickcheck::TestResult {
            let Ok(af) = a.fuzz() else { return quickcheck::TestResult::discard(); };
            let Ok(bf) = b.fuzz() else { return quickcheck::TestResult::discard(); };
            let Ok(cf) = c.fuzz() else { return quickcheck::TestResult::discard(); };
            #[allow(clippy::arithmetic_side_effects)]
            let abc = a >> b.optional() >> c;
            #[allow(clippy::default_numeric_fallback)]
            for ((ai, bi), ci) in af.zip(bf).zip(cf).take(10) {
                assert!(
                    abc.accept(ai.iter().chain(bi.iter()).chain(ci.iter())),
                    "Sandwiched optional did not accept the concatenation of \
                    three valid inputs: {ai:?}, {bi:?}, & {ci:?}",
                );
                assert!(
                    abc.accept(ai.iter().chain(ci.iter())),
                    "Sandwiched optional did not accept the concatenation of \
                    two valid inputs: {ai:?} & {ci:?}",
                );
            }
            quickcheck::TestResult::passed()
        }
    }
}

mod reduced {
    use super::*;

    fn nfa_dfa_equal(nfa: &Nfa<u8>, input: &[u8]) {
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
            if dfa_accepted { "accepted" } else { "did not" },
        );
    }

    fn brzozowski(nfa: &Nfa<u8>, input: &[u8]) {
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
            if dfa_accepted { "accepted" } else { "did not" },
        );
    }

    fn bitor(lhs: &Nfa<u8>, rhs: &Nfa<u8>, input: &[u8]) {
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
                "did not"
            },
        );
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn shr(lhs: &Nfa<u8>, rhs: &Nfa<u8>, input: &[u8]) {
        println!("LHS:");
        println!("{lhs}");
        println!("RHS:");
        println!("{rhs}");
        let fused = lhs.clone() >> rhs.clone();
        println!("Fused:");
        println!("{fused}");
        let chain_accepted = nfa::chain(lhs, rhs, input);
        let fused_accepted = fused.accept(input);
        assert_eq!(
            fused_accepted,
            chain_accepted,
            "On input {input:?}, \
            the LHS chained to the RHS {} but the fused NFA {}",
            if chain_accepted {
                "accepted"
            } else {
                "did not accept"
            },
            if fused_accepted {
                "accepted"
            } else {
                "did not"
            },
        );
    }

    #[test]
    fn nfa_dfa_equal_1() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: once((0, nfa::Transition::default())).collect(),
                    accepting: true,
                }],
                initial: once(0).collect(),
            },
            &[],
        );
    }

    #[test]
    fn nfa_dfa_equal_2() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: once(0).collect(),
            },
            &[],
        );
    }

    #[test]
    fn nfa_dfa_equal_3() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: once((255, nfa::Transition::default())).collect(),
                    accepting: true,
                }],
                initial: once(0).collect(),
            },
            &[255],
        );
    }

    #[test]
    fn nfa_dfa_equal_4() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: once((
                            255,
                            nfa::Transition {
                                dsts: once(0).collect(),
                                call: None,
                            },
                        ))
                        .collect(),
                        accepting: true,
                    },
                ],
                initial: once(0).collect(),
            },
            &[255],
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
                initial: once(1).collect(),
            },
            &[],
        );
    }

    #[test]
    fn brzozowski_1() {
        brzozowski(
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[],
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
                initial: once(0).collect(),
            },
            &[],
        );
    }

    #[test]
    fn brzozowski_3() {
        brzozowski(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: once((
                        0,
                        nfa::Transition {
                            dsts: once(0).collect(),
                            call: None,
                        },
                    ))
                    .collect(),
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            &[],
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
                initial: once(0).collect(),
            },
            &[],
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
                initial: once(0).collect(),
            },
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn bitor_3() {
        bitor(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: once((
                        1,
                        nfa::Transition {
                            dsts: once(0).collect(),
                            call: None,
                        },
                    ))
                    .collect(),
                    accepting: false,
                }],
                initial: once(0).collect(),
            },
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: once(0).collect(),
            },
            &[1],
        );
    }

    #[test]
    fn shr_1() {
        shr(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: once(0).collect(),
            },
            &Nfa {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn shr_2() {
        shr(
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: once((
                            2,
                            nfa::Transition {
                                dsts: once(1).collect(),
                                call: None,
                            },
                        ))
                        .collect(),
                        accepting: true,
                    },
                ],
                initial: once(1).collect(),
            },
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: once((
                            1,
                            nfa::Transition {
                                dsts: once(1).collect(),
                                call: None,
                            },
                        ))
                        .collect(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: once(0).collect(),
            },
            &[2, 1],
        );
    }

    #[test]
    fn shr_3() {
        shr(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: once(0).collect(),
            },
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: once(0).collect(),
            },
            &[],
        );
    }
}

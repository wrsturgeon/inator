/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(
    clippy::arithmetic_side_effects,
    clippy::print_stdout,
    clippy::unwrap_used
)]

use super::{dfa, nfa, Compiled as Dfa, Parser as Nfa};
use std::collections::{BTreeMap, BTreeSet};

mod unit {
    use super::*;

    #[test]
    fn zero_state_nfa_subsets() {
        let nfa = Nfa::<()>::empty();
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
        let _ = Nfa::<()>::empty().fuzz().unwrap_err();
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
            initial: core::iter::once(0).collect(),
        }
        .fuzz()
        .unwrap_err();
    }
}

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;

    quickcheck::quickcheck! {
        fn nfa_dfa_equal(nfa: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let dfa = nfa.clone().subsets();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| nfa.format(input.iter().copied()) == dfa.format(input)),
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
                    .all(|input| dfa.format(input.iter().copied()) == nfa.format(input)),
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
                    .all(|input| nfa.format(input.iter().copied()) == dfa.format(input)),
            )
        }

        fn brzozowski_reduces_size(dfa: Dfa<u8>) -> quickcheck::TestResult {
            let orig_size = dfa.size();
            match dfa.generalize().compile().size().cmp(&orig_size) {
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
            let nfa = Nfa::unit(singleton, vec![]);
            quickcheck::TestResult::from_bool(nfa.format(accept).is_some() && nfa.format(reject).is_none())
        }

        fn bitor(lhs: Nfa<u8>, rhs: Nfa<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let fused = lhs.clone() | rhs.clone();
            quickcheck::TestResult::from_bool(inputs.into_iter().all(|input| {
                fused.format(input.iter().copied())
                    == lhs.format(input.iter().copied()).or_else(|| rhs.format(input))
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
                    .all(|input| fused.format(&input) == nfa::chain(&lhs, &rhs, &input)),
            )
        }

        fn fuzz_roundtrip(nfa: Nfa<u8>) -> quickcheck::TestResult {
            nfa.fuzz()
                .map_or(quickcheck::TestResult::discard(), |fuzz| {
                    quickcheck::TestResult::from_bool(fuzz.take(100).all(|input| nfa.format(input).is_some()))
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
                if repeated.format(fst.into_iter().chain(snd)).is_none() {
                    return quickcheck::TestResult::failed();
                }
            }
            quickcheck::TestResult::passed()
        }

        fn star_def_swap_eq(nfa: Nfa<u8>) -> bool {
            nfa.clone().repeat().optional().compile() == nfa.optional().repeat().compile()
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
                    abc.format(ai.iter().chain(bi.iter()).chain(ci.iter())).is_some(),
                    "Sandwiched optional did not accept the concatenation of \
                    three valid inputs: {ai:?}, {bi:?}, & {ci:?}",
                );
                assert!(
                    abc.format(ai.iter().chain(ci.iter())).is_some(),
                    "Sandwiched optional did not accept the concatenation of \
                    two valid inputs: {ai:?} & {ci:?}",
                );
            }
            quickcheck::TestResult::passed()
        }

        fn backtrack(nfa: Nfa<u8>, huge_index: usize) -> quickcheck::TestResult {
            nfa.backtrack(huge_index % nfa.size()).map_or_else(
                quickcheck::TestResult::discard,
                |minimal_input| quickcheck::TestResult::from_bool(
                    nfa.format(minimal_input).is_some(),
                ),
            )
            // TODO: use `step` to make sure it ends up in EXACTLY the right state
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
        let nfa_formatted = nfa.format(input.iter().copied());
        let dfa_formatted = dfa.format(input.iter().copied());
        assert_eq!(
            nfa_formatted, dfa_formatted,
            "On input {input:?}, the NFA formatted {nfa_formatted:?} \
            but the DFA formatted {dfa_formatted:?}",
        );
    }

    fn brzozowski(nfa: &Nfa<u8>, input: &[u8]) {
        println!("NFA:");
        println!("{nfa}");
        let dfa = nfa.clone().compile();
        println!("DFA:");
        println!("{dfa}");
        let nfa_formatted = nfa.format(input.iter().copied());
        let dfa_formatted = dfa.format(input.iter().copied());
        assert_eq!(
            nfa_formatted, dfa_formatted,
            "On input {input:?}, the NFA formatted {nfa_formatted:?} \
            but the DFA formatted {dfa_formatted:?}",
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
        let lhs_formatted = lhs.format(input.iter().copied());
        let rhs_formatted = rhs.format(input.iter().copied());
        let fused_formatted = fused.format(input.iter().copied());
        assert_eq!(
            fused_formatted.as_ref(),
            lhs_formatted.as_ref().or(rhs_formatted.as_ref()),
            "On input {input:?}, the LHS formatted {lhs_formatted:?} \
            and the RHS formatted {rhs_formatted:?} but the fused NFA formatted {fused_formatted:?}",
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
        let chain_formatted = nfa::chain(lhs, rhs, input);
        let fused_formatted = fused.format(input);
        assert_eq!(
            fused_formatted, chain_formatted,
            "On input {input:?}, \
            the LHS chained to the RHS formatted {chain_formatted:?} \
            but the fused NFA formatted {fused_formatted:?}",
        );
    }

    fn star_def_swap_eq(nfa: Nfa<u8>) {
        assert_eq!(
            nfa.clone().repeat().optional().compile(),
            nfa.optional().repeat().compile()
        );
    }

    #[test]
    fn nfa_dfa_equal_1() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((0, nfa::Recommendation::empty())).collect(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
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
            &[],
        );
    }

    #[test]
    fn nfa_dfa_equal_3() {
        nfa_dfa_equal(
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((255, nfa::Recommendation::empty())).collect(),
                    accepting: true,
                }],
                initial: core::iter::once(0).collect(),
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
                        epsilon: core::iter::once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((
                            255,
                            nfa::Recommendation {
                                set: core::iter::once(0).collect(),
                                append: vec![],
                            },
                        ))
                        .collect(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once(0).collect(),
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
                initial: core::iter::once(1).collect(),
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
                initial: core::iter::once(0).collect(),
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
                    non_epsilon: core::iter::once((
                        0,
                        nfa::Recommendation {
                            set: core::iter::once(0).collect(),
                            append: vec![],
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
                initial: core::iter::once(0).collect(),
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
                initial: core::iter::once(0).collect(),
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
                    non_epsilon: core::iter::once((
                        1,
                        nfa::Recommendation {
                            set: core::iter::once(0).collect(),
                            append: vec![],
                        },
                    ))
                    .collect(),
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
                initial: core::iter::once(0).collect(),
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
                        non_epsilon: core::iter::once((
                            2,
                            nfa::Recommendation {
                                set: core::iter::once(1).collect(),
                                append: vec![],
                            },
                        ))
                        .collect(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once(1).collect(),
            },
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((
                            1,
                            nfa::Recommendation {
                                set: core::iter::once(1).collect(),
                                append: vec![],
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
                initial: core::iter::once(0).collect(),
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
            &[],
        );
    }

    #[test]
    fn star_def_swap_eq_1() {
        star_def_swap_eq(Nfa {
            states: vec![],
            initial: BTreeSet::new(),
        });
    }
}

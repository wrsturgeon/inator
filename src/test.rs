/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::print_stdout, clippy::unwrap_used)]

use crate::*;
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

    // fn brzozowski_reduces_size(dfa: &Dfa<u8>) {
    //     let orig_size = dfa.size();
    //     println!("Original DFA (size {orig_size}):");
    //     println!("{dfa}");
    //     let dfa = Nfa::from(dfa.clone()).compile();
    //     let dfa_size = dfa.size();
    //     println!("Reduced DFA (size {dfa_size}):");
    //     println!("{dfa}");
    //     assert!(
    //         dfa_size < orig_size,
    //         "Reducing a {orig_size}-state DFA increased its size to {dfa_size}",
    //     );
    // }

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
                    non_epsilon: core::iter::once((0, BTreeSet::new())).collect(),
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
                    non_epsilon: core::iter::once((255, BTreeSet::new())).collect(),
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
                        non_epsilon: core::iter::once((255, core::iter::once(0).collect()))
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
                    non_epsilon: core::iter::once((0, core::iter::once(0).collect())).collect(),
                    accepting: false,
                }],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    // #[test]
    // fn brzozowski_reduces_size_1() {
    //     brzozowski_reduces_size(&Nfa {
    //         states: vec![
    //             nfa::State {
    //                 epsilon: BTreeSet::new(),
    //                 non_epsilon: BTreeMap::new(),
    //                 accepting: true,
    //             },
    //             nfa::State {
    //                 epsilon: BTreeSet::new(),
    //                 non_epsilon: [(0, 1), (1, 0)]
    //                     .map(|(a, b)| (a, core::iter::once(b).collect()))
    //                     .into_iter()
    //                     .collect(),
    //                 accepting: false,
    //             },
    //         ],
    //         initial: [0, 1].into_iter().collect(),
    //     });
    // }

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
                    non_epsilon: core::iter::once((1, core::iter::once(0).collect())).collect(),
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
                        non_epsilon: core::iter::once((2, core::iter::once(1).collect())).collect(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once(1).collect(),
            },
            &Nfa {
                states: vec![
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((1, core::iter::once(1).collect())).collect(),
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
}

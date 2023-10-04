/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::use_debug)]

use crate::{deterministic as d, nondeterministic as n, Compiled as D, Parser as N, *};
use std::collections::{BTreeMap, BTreeSet};

mod unit {
    use super::*;

    #[test]
    fn zero_state_nfa_subsets() {
        let nfa = N::<(), ()>::void();
        let dfa = nfa.subsets();
        assert_eq!(
            dfa,
            D {
                states: vec![d::State {
                    transitions: BTreeMap::new(),
                    accepting: false
                }],
                initial: (0, ())
            }
        );
    }

    #[test]
    fn zero_state_nfa_fuzz() {
        let _ = N::<(), ()>::void().fuzz().unwrap_err();
    }

    #[test]
    fn zero_initial_nfa_fuzz() {
        let _ = N::<(), ()> {
            states: vec![n::State {
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
        let _ = N::<(), ()> {
            states: vec![
                n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: false,
                },
                n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                },
            ],
            initial: core::iter::once((0, ())).collect(),
        }
        .fuzz()
        .unwrap_err();
    }

    #[test]
    #[should_panic]
    fn ambiguity() {
        let parser = (ignore::<_, ()>('a') >> on('a', "aa"))
            | (ignore('a') >> ignore('a') >> on('b', "aab"));
        drop(parser.compile());
    }
}

mod prop {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "quickcheck")]
    quickcheck::quickcheck! {
        fn nfa_dfa_equal(nfa: N<u8, ()>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
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

        fn dfa_nfa_equal(dfa: D<u8>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
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

        fn nfa_dfa_one_and_a_half_roundtrip(nfa: N<u8, ()>) -> bool {
            let dfa = nfa.subsets();
            dfa.generalize().subsets() == dfa
        }

        fn dfa_nfa_double_roundtrip(dfa: D<u8>) -> bool {
            let once = dfa.generalize().subsets();
            once.generalize().subsets() == once
        }

        fn brzozowski(nfa: N<u8, ()>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
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

        fn brzozowski_reduces_size(dfa: D<u8>) -> quickcheck::TestResult {
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
            let nfa = N::unit(singleton, None);
            quickcheck::TestResult::from_bool(nfa.accept(accept) && !nfa.accept(reject))
        }

        fn bitor(lhs: N<u8, ()>, rhs: N<u8, ()>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
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
        fn shr(lhs: N<u8, ()>, rhs: N<u8, ()>, inputs: Vec<Vec<u8>>) -> quickcheck::TestResult {
            if inputs.is_empty() {
                return quickcheck::TestResult::discard();
            }
            let fused = lhs.clone() >> rhs.clone();
            quickcheck::TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| fused.accept(&input) == n::chain(&lhs, &rhs, &input)),
            )
        }

        fn fuzz_roundtrip(nfa: N<u8, ()>) -> quickcheck::TestResult {
            nfa.fuzz()
                .map_or(quickcheck::TestResult::discard(), |fuzz| {
                    quickcheck::TestResult::from_bool(fuzz.take(100).all(|input| nfa.accept(input)))
                })
        }

        fn repeat_fuzz_chain(nfa: N<u8, ()>) -> quickcheck::TestResult {
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

        fn star_def_swap_eq(nfa: N<u8, ()>) -> bool {
            let lhs = nfa.repeat().optional();
            let rhs = nfa.optional().repeat();
            for (il, ir) in lhs.fuzz().unwrap().zip(rhs.fuzz().unwrap()).take(100) {
                if !rhs.accept(il) { return false; }
                if !lhs.accept(ir) { return false; }
            }
            true
        }

        fn sandwich(a: N<u8, ()>, b: N<u8, ()>, c: N<u8, ()>) -> quickcheck::TestResult {
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

    fn nfa_dfa_equal(nfa: &N<u8, ()>, input: &[u8]) {
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

    fn brzozowski(nfa: &N<u8, ()>, input: &[u8]) {
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

    fn bitor(lhs: &N<u8, ()>, rhs: &N<u8, ()>, input: &[u8]) {
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
    fn shr(lhs: &N<u8, ()>, rhs: &N<u8, ()>, input: &[u8]) {
        println!("LHS:");
        println!("{lhs}");
        println!("RHS:");
        println!("{rhs}");
        let fused = lhs.clone() >> rhs.clone();
        println!("Fused:");
        println!("{fused}");
        let chain_accepted = n::chain(lhs, rhs, input);
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
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((0, (BTreeSet::new(), (), None))).collect(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &[],
        );
    }

    #[test]
    fn nfa_dfa_equal_2() {
        nfa_dfa_equal(
            &N {
                states: vec![
                    n::State {
                        epsilon: core::iter::once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once((0, ())).collect(),
            },
            &[],
        );
    }

    #[test]
    fn nfa_dfa_equal_3() {
        nfa_dfa_equal(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((255, (BTreeSet::new(), (), None))).collect(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &[255],
        );
    }

    #[test]
    fn nfa_dfa_equal_4() {
        nfa_dfa_equal(
            &N {
                states: vec![
                    n::State {
                        epsilon: core::iter::once(1).collect(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((
                            255,
                            (core::iter::once(0).collect(), (), None),
                        ))
                        .collect(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once((0, ())).collect(),
            },
            &[255],
        );
    }

    #[test]
    fn nfa_dfa_equal_5() {
        nfa_dfa_equal(
            &N {
                states: vec![
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once((1, ())).collect(),
            },
            &[],
        );
    }

    #[test]
    fn brzozowski_1() {
        brzozowski(
            &N {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn brzozowski_2() {
        brzozowski(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: false,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &[],
        );
    }

    #[test]
    fn brzozowski_3() {
        brzozowski(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((0, (core::iter::once(0).collect(), (), None)))
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
            &N {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &[],
        );
    }

    #[test]
    fn bitor_2() {
        bitor(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: false,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &N {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn bitor_3() {
        bitor(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: core::iter::once((1, (core::iter::once(0).collect(), (), None)))
                        .collect(),
                    accepting: false,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &[1],
        );
    }

    #[test]
    fn shr_1() {
        shr(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &N {
                states: vec![],
                initial: BTreeSet::new(),
            },
            &[],
        );
    }

    #[test]
    fn shr_2() {
        shr(
            &N {
                states: vec![
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: false,
                    },
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((
                            2,
                            (core::iter::once(1).collect(), (), None),
                        ))
                        .collect(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once((1, ())).collect(),
            },
            &N {
                states: vec![
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: core::iter::once((
                            1,
                            (core::iter::once(1).collect(), (), None),
                        ))
                        .collect(),
                        accepting: false,
                    },
                    n::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: true,
                    },
                ],
                initial: core::iter::once((0, ())).collect(),
            },
            &[2, 1],
        );
    }

    #[test]
    fn shr_3() {
        shr(
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &N {
                states: vec![n::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: true,
                }],
                initial: core::iter::once((0, ())).collect(),
            },
            &[],
        );
    }
}

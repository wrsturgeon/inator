/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::use_debug)]

use crate::{Compiled as Dfa, Parser as Nfa, *};
use std::panic;

#[cfg(feature = "quickcheck")]
use {core::cmp::Ordering, quickcheck::*};

#[cfg(not(feature = "quickcheck"))]
use {
    crate::call::Call,
    core::{fmt::Debug, iter::once, panic::RefUnwindSafe},
    std::collections::{BTreeMap, BTreeSet},
};

#[cfg(not(feature = "quickcheck"))]
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
                    accepting: None
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
                accepting: Some(Call::Pass),
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
                    accepting: None,
                },
                nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: Some(Call::Pass),
                },
            ],
            initial: once(0).collect(),
        }
        .fuzz()
        .unwrap_err();
    }

    #[test]
    #[should_panic]
    fn ambiguity_simple() {
        let parser = on('a', "a") | on('a', "b");
        drop(parser.compile());
    }

    #[test]
    fn ambiguity_prefix_then_diverge() {
        let parser = (ignore('a') >> on('a', "aac") >> ignore('c'))
            | (ignore('a') >> ignore('a') >> on('b', "aab"));
        drop(parser.compile());
    }

    #[test]
    fn ambiguity_but_not_really_just_postpone_it() {
        let parser = (on('a', "aa") >> ignore('a')) | (on('a', "ab") >> ignore('b'));
        drop(parser.compile());
    }
}

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;

    // Tests that have worked in the past:
    quickcheck! {
        fn nfa_dfa_equal(nfa: Nfa<u8>, inputs: Vec<Vec<u8>>) -> TestResult {
            let nfa = nfa.remove_calls();
            if inputs.is_empty() {
                return TestResult::discard();
            }
            let dfa = nfa.clone().subsets();
            TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| nfa.accept(input.iter().copied()) == dfa.accept(input)),
            )
        }

        fn dfa_nfa_equal(dfa: Dfa<u8>, inputs: Vec<Vec<u8>>) -> TestResult {
            if inputs.is_empty() {
                return TestResult::discard();
            }
            let nfa =dfa.generalize();
            TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| dfa.accept(input.iter().copied()) == nfa.accept(input)),
            )
        }

        fn nfa_dfa_one_and_a_half_roundtrip(nfa: Nfa<u8>) -> bool {
            let dfa = nfa.remove_calls().subsets();
            dfa.generalize().subsets() == dfa
        }

        fn dfa_nfa_double_roundtrip(dfa: Dfa<u8>) -> bool {
            let once = dfa.generalize().subsets();
            once.generalize().subsets() == once
        }

        fn brzozowski(nfa: Nfa<u8>, inputs: Vec<Vec<u8>>) -> TestResult {
            let nfa = nfa.remove_calls();
            if inputs.is_empty() {
                return TestResult::discard();
            }
            let dfa = nfa.clone().compile();
            TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| nfa.accept(input.iter().copied()) == dfa.accept(input)),
            )
        }

        fn brzozowski_reduces_size(dfa: Dfa<u8>) -> TestResult {
            let dfa = dfa.remove_calls();
            let orig_size = dfa.size();
            match dfa.generalize().compile().size().cmp(&orig_size) {
                Ordering::Greater => TestResult::failed(),
                Ordering::Equal => TestResult::discard(),
                Ordering::Less => TestResult::passed(),
            }
        }

        fn unit(singleton: u8, reject: Vec<u8>) -> TestResult {
            let accept = vec![singleton];
            if reject == accept {
                return TestResult::discard();
            }
            let nfa = Nfa::unit(singleton, Call::Pass);
            TestResult::from_bool(nfa.accept(accept) && !nfa.accept(reject))
        }

        fn bitor(lhs: Nfa<u8>, rhs: Nfa<u8>, inputs: Vec<Vec<u8>>) -> TestResult {
            if inputs.is_empty() {
                return TestResult::discard();
            }
            let fused = lhs.clone() | rhs.clone();
            TestResult::from_bool(inputs.into_iter().all(|input| {
                fused.accept(input.iter().copied())
                    == (lhs.accept(input.iter().copied()) || rhs.accept(input))
            }))
        }

        #[allow(clippy::arithmetic_side_effects)]
        fn shr(lhs: Nfa<u8>, rhs: Nfa<u8>, inputs: Vec<Vec<u8>>) -> TestResult {
            if inputs.is_empty() {
                return TestResult::discard();
            }
            let fused = lhs.clone() >> rhs.clone();
            TestResult::from_bool(
                inputs
                    .into_iter()
                    .all(|input| fused.accept(&input) == nfa::chain(&lhs, &rhs, &input)),
            )
        }

        fn fuzz_roundtrip(nfa: Nfa<u8>) -> TestResult {
            let nfa = nfa.remove_calls();
            nfa.fuzz()
                .map_or(TestResult::discard(), |fuzz| {
                    TestResult::from_bool(fuzz.take(100).all(|input| nfa.accept(input)))
                })
        }

        fn repeat_fuzz_chain(nfa: Nfa<u8>) -> TestResult {
            let nfa = nfa.remove_calls();
            let Ok(mut fuzzer) = nfa.fuzz() else {
                return TestResult::discard();
            };
            let repeated = nfa.repeat();
            #[allow(clippy::default_numeric_fallback)]
            for _ in 0..100 {
                let fst = fuzzer.next().unwrap();
                let snd = fuzzer.next().unwrap();
                if !repeated.accept(fst.into_iter().chain(snd)) {
                    return TestResult::failed();
                }
            }
            TestResult::passed()
        }

        fn star_def_swap_eq(nfa: Nfa<u8>) -> bool {
            let nfa = nfa.remove_calls();
            let lhs = nfa.repeat().optional();
            let rhs = nfa.optional().repeat();
            for (il, ir) in lhs.fuzz().unwrap().zip(rhs.fuzz().unwrap()).take(100) {
                if !rhs.accept(il) { return false; }
                if !lhs.accept(ir) { return false; }
            }
            true
        }

        fn sandwich(a: Nfa<u8>, b: Nfa<u8>, c: Nfa<u8>) -> TestResult {
            let a = a.remove_calls();
            let b = b.remove_calls();
            let c = c.remove_calls();
            let Ok(af) = a.fuzz() else { return TestResult::discard(); };
            let Ok(bf) = b.fuzz() else { return TestResult::discard(); };
            let Ok(cf) = c.fuzz() else { return TestResult::discard(); };
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
            TestResult::passed()
        }

        fn unambiguous_forward_means_survive_reversal(nfa: Nfa<u8>) -> TestResult {
            let Ok(dfa) = panic::catch_unwind(|| nfa.subsets()) else {
                return TestResult::discard();
            };
            panic::catch_unwind(|| {
                dfa.generalize().reverse().subsets()
            }).map_or_else(|_| TestResult::failed(), |_| TestResult::passed())
        }

        #[allow(clippy::arithmetic_side_effects)]
        fn postpone_ambiguity(x0: u8, tl: Vec<u8>, tr: Vec<u8>) -> bool {
            let parser =
                (on(x0, "lf") >> seq(tl.iter().copied().map(ignore))) |
                (on(x0, "rt") >> seq(tr.iter().copied().map(ignore)));
            panic::catch_unwind(|| parser.subsets()).is_err() == (tl == tr)
        }
    }
}

#[cfg(not(feature = "quickcheck"))]
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

    /// For inputs like ("aa" | "ba"), where forward is fine but backward ("aa" | "ab") is temporarily ambiguous.
    fn unambiguous_forward_means_survive_reversal<
        I: Clone + Debug + Expression + Ord + RefUnwindSafe,
    >(
        nfa: &Nfa<I>,
    ) {
        let Ok(dfa) = panic::catch_unwind(|| nfa.clone().subsets()) else {
            return; // but be warned: this is just totally outside the realm of what we're testing
        };
        let reversed = dfa.generalize().reverse();
        println!("{reversed}");
        drop(reversed.subsets()); // shouldn't panic
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn postpone_ambiguity(x0: u8, tl: &[u8], tr: &[u8]) {
        let parser = (on(x0, "lf") >> seq(tl.iter().copied().map(ignore)))
            | (on(x0, "rt") >> seq(tr.iter().copied().map(ignore)));
        assert_ne!(tl, tr);
        drop(parser.subsets()); // shouldn't panic
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
                    accepting: Some(Call::Pass),
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
                        accepting: None,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: Some(Call::Pass),
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
                    accepting: Some(Call::Pass),
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
                        accepting: None,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: once((
                            255,
                            nfa::Transition {
                                dsts: once(0).collect(),
                                call: Call::Pass,
                            },
                        ))
                        .collect(),
                        accepting: Some(Call::Pass),
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
                        accepting: None,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: Some(Call::Pass),
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
                    accepting: None,
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
                            call: Call::Pass,
                        },
                    ))
                    .collect(),
                    accepting: None,
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
                    accepting: Some(Call::Pass),
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
                    accepting: None,
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
                            call: Call::Pass,
                        },
                    ))
                    .collect(),
                    accepting: None,
                }],
                initial: once(0).collect(),
            },
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: Some(Call::Pass),
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
                    accepting: Some(Call::Pass),
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
                        accepting: None,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: once((
                            2,
                            nfa::Transition {
                                dsts: once(1).collect(),
                                call: Call::Pass,
                            },
                        ))
                        .collect(),
                        accepting: Some(Call::Pass),
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
                                call: Call::Pass,
                            },
                        ))
                        .collect(),
                        accepting: None,
                    },
                    nfa::State {
                        epsilon: BTreeSet::new(),
                        non_epsilon: BTreeMap::new(),
                        accepting: Some(Call::Pass),
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
                    accepting: Some(Call::Pass),
                }],
                initial: once(0).collect(),
            },
            &Nfa {
                states: vec![nfa::State {
                    epsilon: BTreeSet::new(),
                    non_epsilon: BTreeMap::new(),
                    accepting: Some(Call::Pass),
                }],
                initial: once(0).collect(),
            },
            &[],
        );
    }

    #[test]
    fn unambiguous_forward_means_survive_reversal_actually_ambiguous_is_fine() {
        unambiguous_forward_means_survive_reversal(
            &(on_seq("a".chars(), "a") | on_seq("a".chars(), "b")),
        );
    }

    #[test]
    fn unambiguous_forward_means_survive_reversal_1() {
        unambiguous_forward_means_survive_reversal(
            &(on_seq("aa".chars(), "a") | on_seq("ba".chars(), "b")),
        );
    }

    #[test]
    fn postpone_ambiguity_1() {
        postpone_ambiguity(0, &[], &[0]);
    }
}

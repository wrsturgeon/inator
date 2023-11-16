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

/*
/// Check if we can split this input into a bunch of non-zero-sized slices
/// that are all individually accepted by a given parser.
#[inline]
#[cfg(feature = "quickcheck")] // <-- TODO: disable for reduced tests
fn sliceable<I: Input, C: Ctrl<I>>(parser: &Graph<I, C>, input: &[I]) -> bool {
    input.is_empty()
        || (1..=input.len()).rev().any(|i| {
            parser.accept(input[..i].iter().cloned()).is_ok() && sliceable(parser, &input[i..])
        })
}
*/

mod unit {
    use super::*;

    #[test]
    fn star_in_paren_structural() {
        // Same as `automata/examples/paren_a_star.rs`
        let handcrafted = Graph {
            states: vec![
                State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(
                            iter::once((
                                Range::unit('('),
                                Transition::Call {
                                    region: "parentheses",
                                    detour: 1,
                                    dst: Box::new(Transition::Lateral {
                                        dst: 2,
                                        update: None,
                                    }),
                                    combine: ff!(|(), ()| ()),
                                },
                            ))
                            .collect(),
                        ),
                        fallback: None,
                    },
                    non_accepting: iter::once("No input".to_owned()).collect(),
                },
                State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(
                            [
                                (
                                    Range::unit('a'),
                                    Transition::Lateral {
                                        dst: 1,
                                        update: None,
                                    },
                                ),
                                (
                                    Range::unit(')'),
                                    Transition::Return {
                                        region: "parentheses",
                                    },
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        fallback: None,
                    },
                    non_accepting: iter::once("Unclosed parentheses".to_owned()).collect(),
                },
                State {
                    transitions: Curry::Scrutinize {
                        filter: RangeMap(BTreeMap::new()),
                        fallback: None,
                    },
                    non_accepting: BTreeSet::new(),
                },
            ],
            initial: 0,
        };
        assert_eq!(
            region(
                "parentheses",
                toss('('),
                toss('a').star(),
                toss(')'),
                ff!(|(), ()| ()),
            ),
            handcrafted,
        );
    }
}

#[cfg(feature = "quickcheck")]
mod prop {
    use super::*;
    use quickcheck::*;

    quickcheck! {
        fn empty_works(input: Vec<u8>) -> bool {
            let parser = empty::<u8>();
            if parser.check().is_err() { return false; }
            input.is_empty() == empty().accept(input).is_ok()
        }

        fn on_any_of_works(range: Range<u8>, input: Vec<u8>) -> bool {
            let parser = on_any_of(range, update!(|(), _| {}));
            if parser.check().is_err() { return false; }
            parser.accept(input.iter().copied()).is_ok() == (input.len() == 1 && range.contains(&input[0]))
        }

        #[allow(clippy::diverging_sub_expression, clippy::todo, unreachable_code, unused_variables)] // <-- FIXME
        fn star_in_paren(count: u8) -> bool {
            let input = iter::once('(').chain(iter::repeat('a').take(usize::from(count))).chain(iter::once(')'));
            let parser = region(
                "parentheses",
                toss('('),
                toss('a').star(),
                toss(')'),
                ff!(|(), ()| ()),
            );
            parser.accept(input).is_ok()
        }

        fn region_depth_one(open: Parser<u8>, contents: Parser<u8>, close: Parser<u8>, input: Vec<u8>) -> bool {
            let seq = open.clone() >> contents.clone() >> close.clone();
            let reg = region("region", open, contents, close, ff!(|(), ()| ()));
            seq.accept(input.iter().copied()) == reg.accept(input)
        }
    }
}

mod reduced {
    use super::*;
    use core::iter;

    fn star_in_paren(count: u8) {
        let input = iter::once('(')
            .chain(iter::repeat('a').take(usize::from(count)))
            .chain(iter::once(')'));
        let parser = region(
            "parentheses",
            toss('('),
            toss('a').star(),
            toss(')'),
            ff!(|(), ()| ()),
        );
        assert_eq!(parser.accept(input), Ok("()".to_owned()));
    }

    #[test]
    fn star_in_paren_1() {
        star_in_paren(0);
    }
}

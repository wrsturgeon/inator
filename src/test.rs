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
            let splittable = (0..=both.len()).any(|i| {
                parser.accept(both[..i].iter().copied()).is_ok() && parser.accept(both[i..].iter().copied()).is_ok()
            });
            let repeated = fixpoint("da capo") >> parser >> recurse("da capo");
            if repeated.check().is_err() {
                return false;
            }
            repeated.accept(both).is_ok() == splittable
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
        let splittable = (0..=both.len()).any(|i| {
            parser.accept(both[..i].iter().copied()).is_ok()
                && parser.accept(both[i..].iter().copied()).is_ok()
        });
        let repeated = fixpoint("da capo") >> parser >> recurse("da capo");
        println!("Repeated: {repeated:#?}");
        repeated.check().unwrap();
        let mut run = both.iter().copied().run(&repeated);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            println!("{:?} {run:?}", r.unwrap());
        }
        assert_eq!(repeated.accept(both).is_ok(), splittable);
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
}

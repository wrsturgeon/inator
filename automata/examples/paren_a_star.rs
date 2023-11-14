use core::iter;
use inator_automata::*;
use rand::{thread_rng, RngCore};
use std::collections::{BTreeMap, BTreeSet};

/// Check if this string consists of _n_ `a`s in parentheses.
fn accept<I: Iterator<Item = char>>(mut iter: I) -> bool {
    if !matches!(iter.next(), Some('(')) {
        return false;
    }
    loop {
        match iter.next() {
            Some('a') => continue,
            Some(')') => return true,
            _ => return false,
        }
    }
}

/// Output a jumble of characters with a very low chance of being valid.
fn shitpost<R: RngCore>(rng: &mut R) -> String {
    let mut s = String::new();
    loop {
        let i = rng.next_u32();
        if (i & 9) == 0 {
            return s;
        }
        s.push(char::from(i as u8));
    }
}

pub fn main() {
    let parser = Graph {
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
    parser.check().unwrap();

    // Print the Rust source representation of this parser
    println!("{}", parser.to_src().unwrap());

    let mut rng = thread_rng();

    // Accept all valid strings
    for _ in 0..10 {
        let n = (rng.next_u32() & 15) as usize;
        let s: String = iter::once('(')
            .chain(iter::repeat('a').take(n))
            .chain(iter::once(')'))
            .collect();
        println!();
        println!("{s:?}");
        let mut run = s.chars().run(&parser);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            let Ok(c) = r else { panic!("{r:?}") };
            println!("{c:?} {run:?}");
        }
    }

    // Reject all invalid strings
    'examples: for _ in 0..10 {
        let s = shitpost(&mut rng);
        println!();
        println!("{s:?}");
        let mut run = s.chars().run(&parser);
        println!("    {run:?}");
        while let Some(r) = run.next() {
            let Ok(c) = r else {
                assert!(!accept(s.chars()));
                continue 'examples;
            };
            println!("{c:?} {run:?}");
        }
        assert!(accept(s.chars()));
    }
}

use core::iter;
use inator_automata::{
    update, Action, CurryInput, CurryStack, Deterministic, Graph, Range, RangeMap, Run, State,
    ToSrc, Transition,
};
use rand::{thread_rng, RngCore};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Symbol {
    Paren, // Just one value, but e.g. if we had parens and brackets, we would use two.
}

impl ToSrc for Symbol {
    #[inline]
    fn to_src(&self) -> String {
        match *self {
            Self::Paren => "Symbol::Paren".to_owned(),
        }
    }
    #[inline]
    fn src_type() -> String {
        "Symbol".to_owned()
    }
}

/// Very manually constructed parser recognizing only valid parentheses.
fn parser() -> Deterministic<char, Symbol> {
    Graph {
        states: vec![State {
            transitions: CurryStack {
                wildcard: Some(CurryInput::Scrutinize(RangeMap {
                    entries: iter::once((
                        Range::unit('('),
                        Transition {
                            dst: 0,
                            update: update!(|(), _| ()),
                            act: Action::Push(Symbol::Paren),
                        },
                    ))
                    .collect(),
                })),
                map_none: None,
                map_some: iter::once((
                    Symbol::Paren,
                    CurryInput::Scrutinize(RangeMap {
                        entries: iter::once((
                            Range::unit(')'),
                            Transition {
                                dst: 0,
                                update: update!(|(), _| ()),
                                act: Action::Pop,
                            },
                        ))
                        .collect(),
                    }),
                ))
                .collect(),
            },
            non_accepting: BTreeSet::new(),
            tags: BTreeSet::new(),
        }],
        initial: 0,
    }
}

/// Generate test cases (has nothing to do with automata!).
fn generate<R: RngCore>(rng: &mut R, fuel: u8) -> String {
    let Some(depleted) = fuel.checked_sub(1) else {
        return String::new();
    };
    let f: [fn(&mut R, u8) -> String; 3] = [
        |_, _| String::new(),
        |r, d| "(".to_owned() + &generate(r, d) + ")",
        |r, d| generate(r, d >> 1) + &generate(r, d >> 1),
    ];
    f[(rng.next_u32() % 3) as usize](rng, depleted)
}

/// Check if this string consists of matched parentheses.
fn accept<I: Iterator<Item = char>>(iter: I) -> bool {
    let mut i: usize = 0;
    for c in iter {
        i = match c {
            '(' => i + 1,
            ')' => {
                if let Some(pred) = i.checked_sub(1) {
                    pred
                } else {
                    return false;
                }
            }
            _ => unreachable!(),
        }
    }
    i == 0
}

/// Output a jumble of parentheses with a very low chance of being valid.
fn shitpost<R: RngCore>(rng: &mut R) -> String {
    let mut s = String::new();
    loop {
        let i = rng.next_u32();
        if i & 2 == 0 {
            return s;
        }
        s.push(if i & 1 == 0 { '(' } else { ')' });
    }
}

pub fn main() {
    let parser = parser();

    let mut rng = thread_rng();

    // Accept all valid strings
    for _ in 0..10 {
        let s = generate(&mut rng, 32);
        println!();
        println!("\"{s}\"");
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
        println!("\"{s}\"");
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

    // Print the Rust source representation of this parser
    println!("{}", parser.to_src().unwrap());
}

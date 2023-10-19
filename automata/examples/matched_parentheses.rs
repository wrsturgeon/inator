use core::iter;
use inator_automata::{
    update, Action, CurryInput, CurryStack, Deterministic, Graph, Range, RangeMap, Run, State,
    Transition,
};
use rand::{thread_rng, RngCore};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Symbol {
    Paren, // Just one value, but e.g. if we had parens and brackets, we would use two.
}

/// Very manually constructed parser recognizing only valid parentheses.
fn parser() -> Deterministic<char, Symbol, ()> {
    Graph {
        states: vec![State {
            transitions: CurryStack {
                wildcard: Some(CurryInput::Scrutinize(RangeMap {
                    entries: vec![(
                        Range::unit('('),
                        Transition {
                            dst: 0,
                            update: update!(|(), _| ()),
                            act: Action::Push(Symbol::Paren),
                        },
                    )],
                })),
                map_none: None,
                map_some: iter::once((
                    Symbol::Paren,
                    CurryInput::Scrutinize(RangeMap {
                        entries: vec![(
                            Range::unit(')'),
                            Transition {
                                dst: 0,
                                update: update!(|(), _| ()),
                                act: Action::Pop,
                            },
                        )],
                    }),
                ))
                .collect(),
            },
            accepting: true,
        }],
        initial: 0,
    }
}

/// Generate test cases (has nothing to do with automata!).
fn generate<R: RngCore>(rng: &mut R) -> String {
    let f: [fn(&mut R) -> String; 3] = [
        |_| String::new(),
        |r| "(".to_owned() + &generate(r) + ")",
        |r| generate(r) + &generate(r),
    ];
    f[(rng.next_u32() % 3) as usize](rng)
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
    for _ in 0..5 {
        let s = generate(&mut rng);
        println!();
        println!("\"{s}\"");
        let mut run = s.chars().run(&parser);
        println!("{run:?}");
        while run.next().is_some() {
            println!("{run:?}");
        }
        assert_eq!(run.ctrl, Err(true));
    }

    // Reject all invalid strings
    for _ in 0..5 {
        let s = shitpost(&mut rng);
        println!();
        println!("\"{s}\"");
        let mut run = s.chars().run(&parser);
        println!("{run:?}");
        while run.next().is_some() {
            println!("{run:?}");
        }
        assert_eq!(run.ctrl, Err(accept(s.chars())));
    }
}

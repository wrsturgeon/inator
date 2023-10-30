use core::iter;
use inator_automata::{
    update, Action, CurryInput, CurryStack, Deterministic, IllFormed, Range, RangeMap, State,
    Transition,
};
use std::{collections::BTreeSet, io};
use symbols::Symbol;

pub fn main() -> Result<io::Result<()>, IllFormed<char, Symbol, usize>> {
    // Very manually constructed parser recognizing only valid parentheses.
    let parser = Deterministic {
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
            non_accepting: vec![],
            tags: BTreeSet::new(),
        }],
        initial: 0,
    };

    parser.to_file("src/parser.rs")
}

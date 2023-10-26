use core::iter;
use inator_automata::{
    update, Action, CurryInput, CurryStack, Deterministic, Range, RangeMap, State, Transition,
};
use std::{collections::BTreeSet, io};
use symbols::Symbol;

pub fn main() -> io::Result<()> {
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
            accepting: true,
            tag: BTreeSet::new(),
            input_t: "()".to_owned(),
        }],
        initial: 0,
        output_t: "()".to_owned(),
    };

    parser.to_file("src/parser.rs")
}

//! Matches the specification from <https://www.json.org/json-en.html> almost word-for-word.

use inator::*;
use std::io;

fn main() -> Result<io::Result<()>, IllFormed<char, types::Stack, usize>> {
    /*
    let ws = fixpoint("ws")
        >> (empty()
            | ((toss('\u{0020}') | toss('\u{000A}') | toss('\u{000D}') | toss('\u{0009}'))
                >> recurse("ws")));

    let sign = empty() | call('+', "positive") | call('-', "negative");

    let nonzero = call_range('1'..='9', "nonzero");
    let digit = call('0', "zero") | nonzero;
    let digits = fixpoint("digits") >> (digit | (digit >> recurse("digits")));

    let fraction = empty() | (toss('.') >> digits);

    let exponent =
        empty() | ((call('E', "start_exponent") | call('e', "start_exponent")) >> sign >> digits);
    */

    let parser = empty::<char, types::Stack>();

    parser.determinize().unwrap().to_file("src/parser.rs")
}

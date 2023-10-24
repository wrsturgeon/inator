//! Matches the specification from <https://www.json.org/json-en.html> almost word-for-word.

use std::io;

fn main() -> io::Result<()> {
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

    Ok(())
}

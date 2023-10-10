//! Literal transcription of <https://www.json.org/json-en.html>

use inator::{any, ignore, on, on_range, on_seq, space, Parser};

fn parser() -> Parser<char> {
    // Define a bunch of parsers, starting small, and keep combining until we have a full JSON parser.

    // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    let digit = on_range('0'..='9', "digit");

    // e.g. -1.000e-10
    //            ^^^^
    let exp = (on('e', "exponential") | on('E', "exponential"))
        + (ignore('+') | on('-', "negative")).optional()
        + digit.repeat();

    // e.g. -1.000e-10
    //        ^^^^
    let fractional = on('.', "fraction_dot") >> digit.repeat();

    // e.g. -1.000e-10
    //      ^^^^^^^^^^
    let number =
        on('-', "negative").optional() >> digit.repeat() >> fractional.optional() >> exp.optional();

    // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F
    let hex = on_range('0'..='9', "hex_digit_digit")
        | on_range('A'..='F', "hex_digit_capitalized")
        | on_range('a'..='f', "hex_digit_lowercase");

    // e.g. \u47FD
    //        ^^^^
    let unicode_hex = hex.clone() >> hex.clone() >> hex.clone() >> hex; // four of 'em

    // \n, \t, \u47FD, \\, \", etc.: anything that would otherwise be parsed with a special meaning
    let escaped_character = ignore('\\')
        >> ((ignore('u') >> unicode_hex)
            | on('"', "esc_quote")
            | on('\\', "esc_backslash")
            | on('/', "esc_slash")
            | on('b', "esc_b")
            | on('f', "esc_f")
            | on('n', "esc_n")
            | on('r', "esc_r")
            | on('t', "esc_t"));

    // Any usual character in a string (not the above, not control characters like delete).
    let non_escaped_character = any((0..=127)
        .filter(|&c| c != b'"' && c != b'\\' && !c.is_ascii_control())
        .map(|c| on(c.into(), "character")));

    // Any character, escaped or not.
    let character = non_escaped_character | escaped_character;

    // Literally writing `null`.
    let lit_null = on_seq("null".chars(), "lit_null");

    // Literally writing `true`.
    let lit_true = on_seq("true".chars(), "lit_true");

    // Literally writing `false`.
    let lit_false = on_seq("false".chars(), "lit_false");

    // A set of characters (in the specific sense defined above) between quotes.
    let string = on('"', "begin_string") >> character.star() >> on('"', "end_string");

    // Here's where the limitations of automata come in.
    //
    // We want to write this:
    //
    // ```rust
    // let member = string + on(':', "key_value_sep") + value;
    // let members = comma_separated(member);
    // let object = on('{', "begin_object") + members + ignore('}');
    //
    // let element = space() >> value >> space();
    // let elements = comma_separated(element);
    // let array = on('[', "begin_array") + elements + ignore(']');
    //
    // // Look how beautiful this definition is!
    // let value = string | number | object | array | lit_true | lit_false | lit_null;
    // ```
    //
    // ...but that would require `value` to contain itself. :(
    //
    // So, instead, we deal with nesting _outside_ automata theory, in good ol' Rust.
    // What we can do is define _how to start a thing_ and _how to end a thing_,
    // then we just have to push and pop in Rust to make sure it comes out even.

    // Arrays are easier:
    let begin_array = on('[', "open_array") + space();
    let end_array = space() + on(']', "end_array");

    // Objects are a bit trickier:
    let begin_object = on('{', "open_object") + string.clone() + on(':', "key_value_sep") + space(); // Stop exactly when we might circle back to the beginning.
    let end_object = space() + on('}', "close_object");

    let literal = string | number | lit_true | lit_false | lit_null;

    // We can't _a priori_ reason about infintely nested data structures, so we have to deal with them outside an automaton:
    let value = literal | begin_array | end_array | begin_object | end_object;

    // At last,
    space() >> value >> space()
}

fn main() -> std::io::Result<()> {
    std::fs::write("src/parser.rs", parser().compile().into_source("json"))
}

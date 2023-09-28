//! Literal transcription of <https://www.json.org/json-en.html>

use inator::{any, empty, ignore, on, on_seq, opt, postpone, space, Compiled, Parser};

fn comma_separated(p: Parser<'_, char>) -> Parser<'_, char> {
    empty() // no elements
        | ( // or...
            p.clone() // first element
                + (on(',', "list_sep") + p).star() // rest of the elements
                + opt(',') // optional trailing comma
        )
}

fn parser() -> Parser<char> {
    // Define a bunch of parsers, starting small, and keep combining until we have a full JSON parser.

    let nzero = any(('1'..='9').map(|c| on(c, "nzero")));
    let digit = any(('0'..='9').map(|c| on(c, "digit")));
    let exp = (on('e', "exponential") | on('E', "exponential"))
        + (ignore('+') | on('-', "negative")).optional()
        + digit.clone().repeat();
    let fractional = on('.', "fraction_dot") >> digit.clone().repeat();
    let number = on('-', "negative").optional()
        >> nzero
        >> digit.star()
        >> fractional.optional()
        >> exp.optional();

    let hex = any(('0'..='9')
        .chain('A'..'F')
        .chain('a'..'f')
        .map(|c| on(c, "hex_digit")));
    let unicode_hex = hex >> hex >> hex >> hex; // four of 'em
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
    let non_escaped_character = any((0..u8::MAX) // all 8-bit characters...
        .filter(|&c| c != b'"' && c != b'\\' && !c.is_ascii_control()) // except ", \, and control characters...
        .map(|c| on(c.into(), "character"))); // ...or'd together
    let character = non_escaped_character | escaped_character;

    let lit_null = on_seq("null".chars(), "lit_null");
    let lit_true = on_seq("true".chars(), "lit_true");
    let lit_false = on_seq("false".chars(), "lit_false");

    let string = on('"', "begin_string") >> character.star() >> ignore('"');

    // Declare (but not define) a value which can nest itself infinitely many times...
    let postponed = None;
    let value = postpone(&postponed);

    let member = string + on(':', "key_value_sep") + value;
    let members = comma_separated(member);
    let object = on('{', "begin_object") + members + ignore('}');

    let element = space() >> value >> space();
    let elements = comma_separated(element.clone());
    let array = on('[', "begin_array") + elements + ignore(']');

    // Look how beautiful this definition is!
    let final_value = string | number | object | array | lit_true | lit_false | lit_null;
    postponed = Some(&final_value); // <-- Mark completed

    element
}

fn main() -> std::io::Result<()> {
    std::fs::write("src/parser.rs", parser().compile().into_source("json"))
}

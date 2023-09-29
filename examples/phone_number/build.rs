//! US-style phone numbers.

use inator::{any, empty, ignore, on};

fn main() -> std::io::Result<()> {
    // Define the basics:
    let digit = any(('0'..='9').map(|c| on(c, "digit")));
    let triple_digit = digit.clone() >> digit.clone() >> digit.clone();
    let quadruple_digit = triple_digit.clone() >> digit.clone();
    let area_code = triple_digit.clone() | (ignore('(') >> triple_digit.clone() >> ignore(')'));

    // Iterate over separators to make sure they're consistent:
    let parser = any([' ', '.', '-'] // Possible valid separators
        .into_iter()
        .map(ignore) // As single-character parsers that skip the separator
        .chain(core::iter::once(empty())) // Plus an option not to use separators
        .map(|sep| {
            // Treat each separator individually:
            area_code.clone()
                >> sep.clone()
                >> triple_digit.clone()
                >> sep
                >> quadruple_digit.clone()
        }));

    // Compile to Rust source code
    std::fs::write(
        "src/autogen.rs",
        parser.compile().into_source("phone_number"),
    ) // function namd `phone_number` ^^^^^^^^^^^^
}

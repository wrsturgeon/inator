//! No tricks under the hood--here are some definitions from our "standard library":
//! ```
//!
//! /// Accept any of these tokens here.
//! #[inline]
//! pub fn any<I: Clone + Ord, II: IntoIterator<Item = I>>(tokens: II) -> Parser<I> {
//!     tokens
//!         .into_iter()
//!         .fold(Parser::void(), |acc, token| acc | c(token))
//! }
//!
//! /// Any amount of whitespace.
//! #[inline]
//! pub fn space() -> Parser<char> {
//!     any((0..u8::MAX)
//!         .filter(|c| c.is_ascii_whitespace())
//!         .map(char::from))
//! }
//!
//! /// Surround this language in parentheses.
//! /// Note that whitespace around the language--e.g. "( A )"--is fine.
//! #[inline]
//! pub fn parenthesized(p: Parser<char>) -> Parser<char> {
//!     c('(') >> space() >> p >> space() >> c(')')
//! }
//!
//! ```

use inator::{base::*, *};

#[inline]
fn definitely_comma(p: Parser<char>) -> Parser<char> {
    p >> space() >> c(',') >> space()
}

#[inline]
fn maybe_comma(p: Parser<char>) -> Parser<char> {
    p >> space() >> opt(',') >> space()
}

#[inline]
fn tuple(p: Parser<char>) -> Parser<char> {
    parenthesized(definitely_comma(p.clone()).repeat() >> maybe_comma(p))
}

fn main() {
    // Specify what we want in parentheses
    let def = tuple(any(['A', 'B', 'C']));

    // Compile it to a provably optimal implementation
    let parser = def.compile();

    // Some unit tests:
    assert!(parser.accept("()".chars())); // Empty tuple
    assert!(parser.accept("(,)".chars())); // Still an empty tuple
    assert!(parser.reject("(,,)".chars())); // Too many commas
    assert!(parser.reject("(A)".chars())); // Just parenthesized, not a tuple
    assert!(parser.accept("(A,)".chars())); // Singleton
    assert!(parser.reject("(A,,)".chars())); // Too many commas
    assert!(parser.accept("(A, B)".chars())); // 2-tuple, no extra comma
    assert!(parser.accept("(A, B,)".chars())); // 2-tuple, extra comma
    assert!(parser.accept("(  A  ,  B  ,  )".chars())); // Whitespace doesn't matter

    // Generate random input:
    for fuzz in parser.fuzz().unwrap().take(32) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }
}

//! No tricks under the hood--here are some definitions from our "standard library":
//! ```
//!
//! /// Surround this language in parentheses.
//! pub fn parenthesized(p: Parser<char>) -> Parser<char> {
//!     c('(') >> p >> c(')')
//! }
//!
//! /// Accept any of these tokens here.
//! pub fn any<I: Clone + Ord, II: IntoIterator<Item = I>>(tokens: II) -> Parser<I> {
//!     tokens
//!         .into_iter()
//!         .fold(Parser::void(), |acc, token| acc | c(token))
//! }
//!
//! /// Any amount of whitespace.
//! pub fn space() -> Parser<char> {
//!     any((0..u8::MAX).filter(u8::is_ascii_whitespace).map(char::from))
//! }
//!
//! ```

use inator::{any, c, s, Parser};

fn parenthesized(p: Parser<char>) -> Parser<char> {
    c('(') >> p >> c(')')
}

fn empty_tuple() -> Parser<char> {
    parenthesized(Parser::empty())
}

fn singleton(p: Parser<char>) -> Parser<char> {
    parenthesized(p >> c(','))
}

fn separator() -> Parser<char> {
    s([',', ' '])
}

fn pair_or_more(p: Parser<char>) -> Parser<char> {
    parenthesized(p.clone() >> separator() >> p.clone() >> (separator() >> p).star())
}

#[inline]
fn tuple(p: Parser<char>) -> Parser<char> {
    empty_tuple() | singleton(p.clone()) | pair_or_more(p)
}

fn main() {
    // Specify what we want in parentheses
    let spec = tuple(any(['A', 'B', 'C']));

    // Compile it to a provably optimal implementation
    let parser = spec.compile();

    // Pretty-print the compiled version as a graph
    println!("{parser}");

    // Some unit tests
    assert!(parser.accept("()".chars())); // Empty tuple
    assert!(parser.reject("(,)".chars())); // Unnecessary comma
    assert!(parser.reject("(A)".chars())); // Just parenthesized, not a tuple
    assert!(parser.accept("(A,)".chars())); // Singleton
    assert!(parser.reject("(A,,)".chars())); // Too many commas
    assert!(parser.accept("(A, B)".chars())); // 2-tuple, no extra comma
    assert!(parser.reject("(A, B,)".chars())); // 2-tuple, extra comma
    assert!(parser.reject("(A, B, )".chars())); // 2-tuple, extra comma & space
    assert!(parser.accept("(A, B, C)".chars())); // 3-tuple

    // Randomly generate guaranteed valid input
    for fuzz in parser.fuzz().unwrap().take(32) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }

    // Compile to Rust source code (e.g. in `build.rs` dumping contents to a file in `src/`)
    println!("{}", parser.into_source("abc_tuple"));
}

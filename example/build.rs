use inator::{empty, ignore, on, Parser};

fn append(c: char) -> Parser<char> {
    on(c, "append") // We define `append` in `src/inator_config.rs`!
}

fn parenthesized(p: Parser<char>) -> Parser<char> {
    ignore('(') + p + ignore(')')
}

fn empty_tuple() -> Parser<char> {
    parenthesized(empty())
}

fn singleton(p: Parser<char>) -> Parser<char> {
    parenthesized(p + ignore(','))
}

fn pair_or_more(p: Parser<char>) -> Parser<char> {
    parenthesized(p.clone() + ignore(',') + p.clone() + (ignore(',') + p).star())
}

#[inline]
fn tuple(p: Parser<char>) -> Parser<char> {
    empty_tuple() | singleton(p.clone()) | pair_or_more(p)
}

fn main() -> std::io::Result<()> {
    // Specify what we want in parentheses
    let spec = tuple(append('A') | append('B') | append('C'));

    // Compile it to a provably optimal implementation
    // SAFETY: No postponed terms.
    let parser = unsafe { spec.compile() }.unwrap();

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
    let formatted = parser.into_source("abc_tuple"); // <-- `abc_tuple` is the function name
    println!("{formatted}");
    std::fs::write("src/autogen.rs", formatted)
}

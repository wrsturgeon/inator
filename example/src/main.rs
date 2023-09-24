//! See `build.rs` first!

mod autogen; // <-- Automatically generated with `inator` in `build.rs`.
mod inator_config;

use autogen::abc_tuple as parse;

fn main() {
    // Same unit tests as in `build.rs`,
    // this time "on the metal" with compiled Rust:
    assert!(parse("()".chars()).is_some()); // Empty tuple
    assert!(parse("(,)".chars()).is_none()); // Unnecessary comma
    assert!(parse("(A)".chars()).is_none()); // Just parenthesized, not a tuple
    assert!(parse("(A,)".chars()).is_some()); // Singleton
    assert!(parse("(A,,)".chars()).is_none()); // Too many commas
    assert!(parse("(A, B)".chars()).is_some()); // 2-tuple, no extra comma
    assert!(parse("(A, B,)".chars()).is_none()); // 2-tuple, extra comma
    assert!(parse("(A, B, )".chars()).is_none()); // 2-tuple, extra comma & space
    assert!(parse("(A, B, C)".chars()).is_some()); // 3-tuple
}

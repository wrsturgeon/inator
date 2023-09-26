//! See `build.rs` first!

mod autogen; // <-- Automatically generated with `inator` in `build.rs`.
mod inator_config;
mod test;

use autogen::{abc_tuple as parse, abc_tuple_fuzz as fuzz};

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

    // Property-testing:
    let mut g = quickcheck::Gen::new(10);
    for _ in 0..10 {
        test::roundtrip(&mut g);
    }

    // Print some inputs (guaranteed to be valid and cover the whole range of valid sequences):
    // for input in autogen::fuzz().take(100) {
    //     println!("{input}");
    // }
}

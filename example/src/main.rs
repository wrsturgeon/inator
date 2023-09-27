//! See `build.rs` first!

mod autogen; // <-- Automatically generated in `build.rs`! Should be `.gitignore`d.
mod inator_config;
mod test;

use autogen::{abc_tuple as parse, abc_tuple_fuzz as fuzz};

fn main() {
    // Same unit tests as in `build.rs`,
    // this time "on the metal" with compiled Rust:
    println!("Unit tests...");
    assert!(parse("()".chars()).is_some()); // Empty tuple
    assert!(parse("(,)".chars()).is_none()); // Unnecessary comma
    assert!(parse("(A)".chars()).is_none()); // Just parenthesized, not a tuple
    assert!(parse("(A,)".chars()).is_some()); // Singleton
    assert!(parse("(A,,)".chars()).is_none()); // Too many commas
    assert!(parse("(A, B)".chars()).is_some()); // 2-tuple, no extra comma
    assert!(parse("(A, B,)".chars()).is_none()); // 2-tuple, extra comma
    assert!(parse("(A, B, )".chars()).is_none()); // 2-tuple, extra comma & space
    assert!(parse("(A, B, C)".chars()).is_some()); // 3-tuple
    println!();

    // Property-testing:
    println!("Property tests...");
    let mut g = quickcheck::Gen::new(10);
    for _ in 0..10 {
        test::roundtrip(&mut g);
    }
    println!();

    // Print some inputs (guaranteed to be valid and cover the whole range of valid sequences):
    println!("Fuzzing inputs...");
    let mut rng = rand::thread_rng();
    for input in core::iter::from_fn(|| Some(fuzz(&mut rng))).take(10) {
        println!(
            "\"{}\" => {:?}",
            input.iter().copied().collect::<String>(),
            parse(input.into_iter()).unwrap()
        );
    }
    println!();
}

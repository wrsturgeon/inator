#[cfg(feature = "quickcheck")]
fn main() {
    use inator_automata::*;
    use quickcheck::*;
    use std::env;
    // SAFETY: Well-tested crate.
    unsafe {
        backtrace_on_stack_overflow::enable();
    }
    let tests = env::var("QUICKCHECK_TESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let mut g = Gen::new(
        env::var("QUICKCHECK_GENERATOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100),
    );
    for _ in 0..tests {
        let parser = Nondeterministic::<u8, u8>::arbitrary(&mut g);
        drop(parser.sort());
    }
}

#[cfg(not(feature = "quickcheck"))]
fn main() {
    println!("Feature `quickcheck` not enabled; doing nothing.");
}

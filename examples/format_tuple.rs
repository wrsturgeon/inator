use inator::{f, format::*, no_space, opt, space};

#[inline]
fn parenthesized(p: Parser<char>) -> Parser<char> {
    f('(') >> no_space() >> p >> no_space() >> f(')')
}

#[inline]
fn tuple(p: Parser<char>) -> Parser<char> {
    parenthesized(p.clone() >> no_space() >> f(','))
        | parenthesized(
            p.clone()
                >> no_space()
                >> f(',')
                >> space()
                >> p.clone() // Require two elements in this (non-singleton) variant
                >> (no_space() >> f(',') >> space() >> p).star()
                >> opt(','),
        )
}

fn main() {
    let slow = tuple(f('A') | f('B') | f('C'));
    println!("Slow:");
    println!("{slow}");

    // Find the provably minimal DFA with a custom extension of Brzozowski's algorithm
    let fast = slow.compile();
    println!();
    println!("Fast:");
    println!("{fast}");

    // Unit test: just parenthesized, not a tuple
    assert_eq!(slow.format("(A)".chars()), None);

    // Unit test: singleton tuple with unnecessary space
    assert_eq!(
        slow.format("(  A  ,  )".chars()),
        Some("(A,)".chars().collect())
    );

    // Unit test: too many commas
    assert_eq!(slow.format("(A,,)".chars()), None);

    // Unit test: two elements with an extra comma
    assert_eq!(
        slow.format("(A, B,)".chars()),
        Some("(A, B)".chars().collect())
    );

    // Generate random input
    for fuzz in fast.fuzz().unwrap().take(32) {
        println!(
            "Fuzz: \"{}\" -> \"{}\" =?= \"{}\"",
            fuzz.iter().copied().collect::<String>(),
            slow.format(fuzz.iter().copied())
                .unwrap()
                .into_iter()
                .collect::<String>(),
            fast.format(fuzz).unwrap().into_iter().collect::<String>(),
        );
    }
}

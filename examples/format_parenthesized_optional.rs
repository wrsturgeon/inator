use inator::{f, format::*};

#[inline]
fn parenthesized(automaton: Parser<char>) -> Parser<char> {
    f('(') >> automaton >> f(')')
}

fn main() {
    let abc = f('A') | f('B') | f('C');
    println!("abc:");
    println!("{abc}");

    let abc_optional = abc.optional();
    println!("abc_optional:");
    println!("{abc_optional}");

    let in_parentheses = parenthesized(abc_optional);
    println!("in_parentheses:");
    println!("{in_parentheses}");

    let compiled = in_parentheses.compile();
    println!("compiled:");
    println!("{compiled}");

    assert_eq!(compiled.format("()".chars()), Some("()".chars().collect()));

    for fuzz in compiled.fuzz().unwrap().take(10) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }

    assert_eq!(
        compiled.format("(A)".chars()),
        Some("(A)".chars().collect())
    );
}

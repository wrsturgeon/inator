use inator::*;

#[inline]
fn parenthesized(automaton: Parser<char>) -> Parser<char> {
    d('(') >> automaton >> d(')')
}

fn main() {
    let abc = d('A') | d('B') | d('C');
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

    // assert!(compiled.accept("()".chars()));

    for fuzz in compiled.fuzz().unwrap().take(10) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }

    assert!(compiled.accept("(A)".chars()));
}

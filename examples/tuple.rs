use inator::*;

#[inline]
fn parenthesized(p: Parser<char>) -> Parser<char> {
    d('(') >> p >> d(')')
}

#[inline]
fn definitely_comma(p: Parser<char>) -> Parser<char> {
    p >> d(',') >> d(' ').star()
}

#[inline]
fn maybe_comma(p: Parser<char>) -> Parser<char> {
    p >> d(',').optional() >> d(' ').star()
}

#[inline]
fn tuple(p: Parser<char>) -> Parser<char> {
    parenthesized(definitely_comma(p.clone()).repeat() >> maybe_comma(p))
}

fn main() {
    let parser = tuple(d('A')).compile();
    assert!(!parser.accept("(A,,)".chars()));
    for fuzz in parser.fuzz().unwrap().take(32) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }
}

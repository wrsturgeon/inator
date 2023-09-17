use inator::prelude::*;

#[inator]
fn just_star() -> impl Parse(char) {
    p!('*')
}

#[inator]
fn parenthesized<O>(inside: impl Parse(char) -> O) -> impl Parse(char) -> O {
    p!('(') >> inside << p!(')')
}

fn main() {
    parenthesized().parse("(*)").unwrap();
}

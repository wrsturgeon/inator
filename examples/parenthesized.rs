use inator::prelude::*;

#[inator]
fn parenthesized<O>(inside: impl Parse(char) -> O) -> impl Parse(char) -> O {
    // p!('(') >> inside << p!(')')
    p!('A' | 'B') // binary (bitor)
                  // p!(_) // infer
}

fn main() {
    parenthesized().parse("(*)");
}

use inator::*;

fn main() {
    let abc = c('A') | c('B') | c('C');
    println!("abc:");
    println!("{abc}");

    let left_paren = c('(');
    println!("left_paren:");
    println!("{left_paren}");

    let right_paren = c(')');
    println!("right_paren:");
    println!("{right_paren}");

    let left_paren_abc = c('(') >> abc;
    println!("left_paren_abc:");
    println!("{left_paren_abc}");

    // let abc_in_parentheses = left_paren_abc << c(')');
    let abc_in_parentheses = left_paren_abc >> c(')');
    println!("abc_in_parentheses:");
    println!("{abc_in_parentheses}");

    let compiled = abc_in_parentheses.compile();
    println!("compiled:");
    println!("{compiled}");

    for fuzz in compiled.fuzz().unwrap().take(10) {
        println!("Fuzz: {}", fuzz.into_iter().collect::<String>());
    }

    assert!(compiled.accept("(A)".chars()));
}
